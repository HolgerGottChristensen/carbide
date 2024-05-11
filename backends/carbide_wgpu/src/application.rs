use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Debug, Formatter};
use std::mem::transmute;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use futures::executor::block_on;
use once_cell::sync::Lazy;

use walkdir::WalkDir;
use wgpu::{Adapter, Device, Instance, Queue};

use carbide_core::{locate_folder, Scene};
use carbide_core::asynchronous::set_event_sink;
use carbide_core::draw::Dimension;
use carbide_core::environment::Environment;
use carbide_core::event::CustomEvent;
use carbide_core::text::InnerTextContext;
use carbide_core::widget::Empty;
use carbide_core::window::WindowId;
use carbide_text::text_context::TextContext;
use carbide_winit::NewEventHandler;
use carbide_winit::event_loop::EventLoopWindowTarget;
use carbide_winit::event_loop::EventLoopBuilder;
use carbide_winit::EventLoop;
use carbide_winit::window::WindowId as WinitWindowId;

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

pub(crate) static DEVICE: Lazy<Arc<Device>> = Lazy::new(|| DEVICE_QUEUE.0.clone());
pub(crate) static QUEUE: Lazy<Arc<Queue>> = Lazy::new(|| DEVICE_QUEUE.1.clone());


thread_local!(pub static EVENT_LOOP: RefCell<EventLoop<CustomEvent>> = RefCell::new(EventLoop::Owned(EventLoopBuilder::<CustomEvent>::with_user_event().build().expect("Expected the event loop creation was successful"))));
thread_local!(pub static WINDOW_IDS: RefCell<HashMap<WinitWindowId, WindowId>> = RefCell::new(HashMap::new()));


pub struct Application {
    /// This contains the whole widget tree. This includes windows and other widgets.
    root: Box<dyn Scene>,
    event_handler: NewEventHandler,
    environment: Environment,
    text_context: TextContext,
    //any_focus: bool,

    //windows: HashMap<WindowId, WGPUWindow>,
}

impl Application {
    pub fn new() -> Self {
        let window_pixel_dimensions = Dimension::new(400.0, 400.0);

        let proxy = EVENT_LOOP.with(|a| {
            match &*a.borrow() {
                EventLoop::Owned(e) => {e.create_proxy()}
                EventLoop::StaticBorrow(_) => unreachable!(),
                EventLoop::None => unreachable!(),
            }
        });

        set_event_sink(ProxyEventLoop(proxy.clone()));

        let environment = Environment::new(
            window_pixel_dimensions,
            Box::new(ProxyEventLoop(proxy)),
        );

        Application {
            root: Box::new(Empty::new()),
            event_handler: NewEventHandler::new(),
            environment,
            text_context: TextContext::new(),
        }
    }

    pub fn assets() -> PathBuf {
        locate_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap()
    }

    pub fn set_scene(&mut self, scene: Box<dyn Scene>) {
        self.root = scene;
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
        let event_loop = EVENT_LOOP.with(|a| {
            match a.take() {
                EventLoop::Owned(e) => e,
                EventLoop::StaticBorrow(_)
                | EventLoop::None => panic!("Can only launch application once"),
            }
        });

        event_loop.run(
            move |event, event_loop| {
                EVENT_LOOP.with(|a| {
                    // SAFETY: We make the reference static such that we can store it in our thread_local
                    // We make sure to drop it after the reference is no longer valid, at the end of
                    // the closure after the event was propagated.
                    let event_loop: &'static EventLoopWindowTarget<CustomEvent> = unsafe {
                        transmute(event_loop)
                    };
                    *a.borrow_mut() = EventLoop::StaticBorrow(event_loop)
                });

                if self.event_handler.event(event, &mut self.root, &mut self.text_context, &mut WGPUImageContext, &mut self.environment) {
                    self.request_redraw()
                }

                if self.environment.should_close_application() {
                    event_loop.exit();
                }

                EVENT_LOOP.with(|a| {
                    a.take()
                });
            },
        ).expect("TODO: panic message");
    }
}

impl Debug for Application {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Application")
            .field("root", &self.root)
            .finish()
    }
}