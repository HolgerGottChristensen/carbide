use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::{Debug, Formatter};
use std::mem::transmute;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Once, OnceLock};
use futures::executor::block_on;
use once_cell::sync::Lazy;

use walkdir::WalkDir;
use wgpu::{Adapter, Device, Instance, Queue};

use carbide_core::{locate_folder, Scene};
use carbide_core::animation::AnimationManager;
use carbide_core::asynchronous::set_event_sink;
use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, EnvironmentStack};
use carbide_core::lifecycle::InitializationContext;
use carbide_core::text::InnerTextContext;
use carbide_core::widget::{Empty, WidgetId};
use carbide_core::window::WindowId;
use carbide_text::text_context::TextContext;
use carbide_winit::application::ApplicationHandler;
use carbide_winit::{NewEventHandler, RequestRedraw};
use carbide_winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use carbide_winit::window::WindowId as WinitWindowId;
use carbide_winit::custom_event::CustomEvent;
use carbide_winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use crate::image_context::WGPUImageContext;
use crate::proxy_event_loop::ProxyEventLoop;

pub(crate) static INSTANCE: Lazy<Arc<Instance>> = Lazy::new(|| {
    Arc::new(Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    }))
});

pub(crate) static ADAPTER: Lazy<Arc<Adapter>> = Lazy::new(|| {
    Arc::new(block_on(INSTANCE.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    })).unwrap())
});

static DEVICE_QUEUE: Lazy<(Arc<Device>, Arc<Queue>)> = Lazy::new(|| {
    let mut limits = wgpu::Limits::default();
    limits.max_bind_groups = 5;

    let (device, queue) = block_on(ADAPTER.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::CLEAR_TEXTURE,
            limits,
        },
        None, // Trace path
    )).unwrap();

    (Arc::new(device), Arc::new(queue))
});

pub static DEVICE: Lazy<Arc<Device>> = Lazy::new(|| DEVICE_QUEUE.0.clone());
pub static QUEUE: Lazy<Arc<Queue>> = Lazy::new(|| DEVICE_QUEUE.1.clone());

pub static EVENT_LOOP_PROXY: OnceLock<EventLoopProxy<CustomEvent>> = OnceLock::new();

pub struct Application {
    id: WidgetId,
    /// This contains the whole widget tree. This includes windows and other widgets.
    root: Box<dyn Scene>,
    event_handler: NewEventHandler,
    environment: Environment,
    environment_stack: EnvironmentStack<'static>,
    text_context: TextContext,
    event_loop: EventLoop<CustomEvent>,
}

impl Application {
    pub fn new() -> Self {
        let window_pixel_dimensions = Dimension::new(400.0, 400.0);

        let event_loop = EventLoop::<CustomEvent>::with_user_event().build().unwrap();

        EVENT_LOOP_PROXY.get_or_init(|| event_loop.create_proxy());

        set_event_sink(ProxyEventLoop(event_loop.create_proxy()));

        let environment = Environment::new(
            window_pixel_dimensions,
            Box::new(ProxyEventLoop(event_loop.create_proxy())),
        );

        Application {
            id: WidgetId::new(),
            root: Box::new(Empty::new()),
            event_handler: NewEventHandler::new(),
            environment,
            environment_stack: EnvironmentStack::new(),
            text_context: TextContext::new(),
            event_loop,
        }
    }

    pub fn assets() -> PathBuf {
        locate_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap()
    }

    pub fn set_scene(&mut self, scene: impl Scene) {
        self.root = Box::new(scene);
    }

    /// Locates the default asset folder and tries to load fonts from a subfolder called /fonts.
    /// For each sub folder in the fonts folder will create a new family with the name of that folder
    /// and load in any fonts within it.
    pub fn with_asset_fonts(mut self) -> Self {
        let assets = locate_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();

        for entry in WalkDir::new(assets) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("ttf")) {
                self.add_font(path.to_path_buf());
            }
        }

        self
    }

    pub fn add_font<P: AsRef<Path>>(&mut self, path: P) {
        self.text_context.add_font(path.as_ref().to_path_buf());
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    /// Request the window to redraw next frame
    fn request_redraw(&self) {
        self.root.request_redraw();
    }

    pub fn launch(mut self) {
        let Application {
            id,
            root,
            event_handler,
            environment,
            environment_stack: type_map,
            text_context,
            event_loop
        } = self;

        let mut running = RunningApplication {
            id,
            root,
            event_handler,
            environment,
            environment_stack: type_map,
            text_context,
            animation_manager: AnimationManager::new(),
        };

        event_loop.run_app(&mut running).unwrap();
    }
}

impl Debug for Application {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Application")
            .field("root", &self.root)
            .finish()
    }
}

pub struct RunningApplication {
    id: WidgetId,
    root: Box<dyn Scene>,
    event_handler: NewEventHandler,
    environment: Environment,
    environment_stack: EnvironmentStack<'static>,
    text_context: TextContext,
    animation_manager: AnimationManager,
}

impl ApplicationHandler<CustomEvent> for RunningApplication {
    fn resumed<'a>(&'a mut self, event_loop: &'a ActiveEventLoop) {
        let mut ctx = InitializationContext::<'a, '_> {
            env: &mut self.environment,
            env_stack: &mut self.environment_stack,
            lifecycle_manager: event_loop as &'a dyn Any
        };

        self.root.process_initialization(&mut ctx);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: CustomEvent) {
        let mut request = RequestRedraw::False;
        self.environment_stack.with_mut::<AnimationManager>(&mut self.animation_manager, |env_stack| {
            request = self.event_handler.user_event(event, &mut self.root, &mut self.text_context, &mut WGPUImageContext, &mut self.environment, env_stack, self.id);

            if matches!(request, RequestRedraw::True) {
                NewEventHandler::handle_refocus(&mut self.root, &mut self.environment, env_stack);
            }

            if self.environment.should_close_application() {
                event_loop.exit();
            }
        });

        match request {
            RequestRedraw::False => {}
            RequestRedraw::True => {
                self.root.request_redraw();
            }
            RequestRedraw::IfAnimationsRequested => {
                if self.animation_manager.take_frame() {
                    self.root.request_redraw();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WinitWindowId, event: WindowEvent) {
        let mut request = RequestRedraw::False;

        self.environment_stack.with_mut::<AnimationManager>(&mut self.animation_manager, |env_stack| {
            request = self.event_handler.window_event(event, window_id, &mut self.root, &mut self.text_context, &mut WGPUImageContext, &mut self.environment, env_stack, self.id);

            if matches!(request, RequestRedraw::True) {
                NewEventHandler::handle_refocus(&mut self.root, &mut self.environment, env_stack);
            }

            if self.environment.should_close_application() {
                event_loop.exit();
            }
        });

        match request {
            RequestRedraw::False => {}
            RequestRedraw::True => {
                self.root.request_redraw();
            }
            RequestRedraw::IfAnimationsRequested => {
                if self.animation_manager.take_frame() {
                    self.root.request_redraw();
                }
            }
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {}

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {}
}