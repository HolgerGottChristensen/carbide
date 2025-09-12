use futures::executor::block_on;
use once_cell::sync::Lazy;
use smallvec::SmallVec;
use std::ffi::OsStr;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use log::info;
use walkdir::WalkDir;
use wgpu::{Adapter, Device, Instance, Queue};

use crate::image_context::WGPUImageContext;
use crate::proxy_event_loop::ProxyEventLoop;
use carbide_core::animation::AnimationManager;
use carbide_core::application::ApplicationManager;
use carbide_core::asynchronous::set_event_sink;
use carbide_core::environment::{Environment, EnvironmentKey};
use carbide_core::event::EventSink;
use carbide_core::focus::FocusManager;
use carbide_core::lifecycle::InitializationContext;
use carbide_core::locate_folder;
use carbide_core::mouse_position::MousePositionKey;
use carbide_core::scene::{AnyScene, Scene, SceneSequence};
use carbide_core::text::TextContext as _;
use carbide_core::widget::WidgetId;
use carbide_cosmic_text::text_context::TextContext;
use carbide_winit::application::ApplicationHandler;
use carbide_winit::custom_event::CustomEvent;
use carbide_winit::event::WindowEvent;
use carbide_winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use carbide_winit::window::WindowId as WinitWindowId;
use carbide_winit::{NewEventHandler, RequestRedraw};
use crate::bind_group_layouts::texture_bind_group_layout;
use crate::samplers::main_sampler;
use crate::wgpu_context::WgpuContext;

pub static EVENT_LOOP_PROXY: OnceLock<EventLoopProxy<CustomEvent>> = OnceLock::new();

pub type Scenes = SmallVec<[Box<dyn AnyScene>; 4]>;

#[derive(Debug)]
pub(crate) struct ActiveEventLoopKey;
impl EnvironmentKey for ActiveEventLoopKey {
    type Value = ActiveEventLoop;
}

pub struct Application {
    id: WidgetId,
    /// This contains the whole widget tree. This includes windows and other widgets.
    scenes: Scenes,

    event_handler: NewEventHandler,
    environment: Environment<'static>,
    text_context: TextContext,
    event_loop: EventLoop<CustomEvent>,
    event_sink: Arc<dyn EventSink>,
}

impl Application {
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("could not initialize logger");
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            env_logger::init();
        }

        let event_loop = EventLoop::<CustomEvent>::with_user_event().build().unwrap();

        EVENT_LOOP_PROXY.get_or_init(|| event_loop.create_proxy());

        set_event_sink(ProxyEventLoop(event_loop.create_proxy()));

        let event_sink = Arc::new(ProxyEventLoop(event_loop.create_proxy()));

        Application {
            id: WidgetId::new(),
            scenes: Default::default(),
            event_handler: NewEventHandler::new(),
            environment: Environment::new(),
            text_context: TextContext::new(),
            event_loop,
            event_sink,
        }
    }

    pub fn add_environment<K: EnvironmentKey + ?Sized>(&mut self, value: &'static K::Value) {
        self.environment.insert::<K>(value)
    }

    pub fn add_environment_owned<K: EnvironmentKey + ?Sized>(&mut self, value: K::Value) {
        self.environment.insert::<K>(Box::leak(Box::new(value)))
    }

    pub fn assets() -> PathBuf {
        locate_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap()
    }

    pub fn set_scene(&mut self, scene: impl Scene) {
        self.scenes.push(Box::new(scene))
    }

    pub fn set_scenes(&mut self, scenes: impl SceneSequence) {
        for scene in scenes.to_vec() {
            self.scenes.push(scene);
        }
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

    pub fn text_context_mut(&mut self) -> &mut dyn carbide_core::text::TextContext {
        &mut self.text_context
    }

    pub fn launch(mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(Application::launch_inner(self))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            block_on(Application::launch_inner(self))
        }

    }

    async fn launch_inner(mut application: Application) {
        let Application {
            id,
            scenes,
            event_handler,
            environment: type_map,
            text_context,
            event_loop,
            event_sink
        } = application;

        let wgpu_context = WgpuContext::new().await;

        let mut running = RunningApplication {
            id,
            scenes,
            event_handler,
            environment: type_map,
            text_context,
            animation_manager: AnimationManager::new(),
            focus_manager: FocusManager::new(),
            event_sink,
            wgpu_context,
        };

        #[cfg(target_arch = "wasm32")]
        {
            use carbide_winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(running)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            event_loop.run_app(&mut running).unwrap();
        }
    }
}

impl Debug for Application {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Application")
            .field("root", &self.scenes)
            .finish()
    }
}

pub struct RunningApplication {
    id: WidgetId,
    scenes: Scenes,
    event_handler: NewEventHandler,
    environment: Environment<'static>,
    text_context: TextContext,
    animation_manager: AnimationManager,
    focus_manager: FocusManager,
    event_sink: Arc<dyn EventSink>,

    wgpu_context: WgpuContext
}

impl RunningApplication {
    /// Request the window to redraw next frame
    fn request_redraw(&self) {
        for scene in &self.scenes {
            if scene.request_redraw() {
                return;
            }
        }

        println!("Redraw was requested, but no root scenes have the ability to request redraw.");
    }

    fn handle_post_event(&mut self, event_loop: &ActiveEventLoop, request: &mut RequestRedraw, application_manager: &mut ApplicationManager) {
        if application_manager.close_requested() {
            event_loop.exit();
        }

        self.environment.with::<ActiveEventLoopKey>(event_loop, |env| {
            env.with::<dyn EventSink>(&self.event_sink, |env| {
                env.with_mut::<WgpuContext>(&mut self.wgpu_context, |env| {
                    let mut ctx = InitializationContext {
                        env,
                    };

                    for mut scene in application_manager.scenes_to_add().drain(..) {
                        scene.process_initialization(&mut ctx);
                        self.scenes.push(scene);
                    }
                })
            })
        });

        for scene in application_manager.scenes_to_dismiss() {
            self.scenes.retain(|a| a.id() != *scene);
        }

        if self.scenes.iter().filter(|a| !a.is_daemon()).count() == 0 {
            event_loop.exit();
        }

        match request {
            RequestRedraw::False => {}
            RequestRedraw::True => {
                NewEventHandler::handle_refocus(&mut self.scenes, &mut self.focus_manager, &mut self.environment);
                self.request_redraw();
            }
            RequestRedraw::IfAnimationsRequested => {
                if self.animation_manager.take_frame() {
                    self.request_redraw();
                }
            }
        }
    }
}

impl ApplicationHandler<CustomEvent> for RunningApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.environment.with::<ActiveEventLoopKey>(event_loop, |env| {
            env.with::<dyn EventSink>(&self.event_sink, |env| {
                env.with_mut::<WgpuContext>(&mut self.wgpu_context, |env| {
                    for scene in &mut self.scenes {
                        scene.process_initialization(&mut InitializationContext {
                            env,
                        });
                    }
                })
            })
        })
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: CustomEvent) {
        let mut request = RequestRedraw::False;

        self.animation_manager.update_frame_time();

        let mut application_manager = ApplicationManager::new();

        let mouse_position = self.event_handler.mouse_position();

        self.environment.with_mut::<AnimationManager>(&mut self.animation_manager, |env| {
            env.with_mut::<WgpuContext>(&mut self.wgpu_context, |env| {
                env.with_mut::<ApplicationManager>(&mut application_manager, |env| {
                    env.with::<ActiveEventLoopKey>(event_loop, |env| {
                        env.with_mut::<FocusManager>(&mut self.focus_manager, |env| {
                            env.with::<MousePositionKey>(&mouse_position, |env| {
                                env.with::<dyn EventSink>(&self.event_sink, |env| {
                                    for scene in &mut self.scenes {
                                        request += self.event_handler.user_event(&event, scene, &mut self.text_context, &mut WGPUImageContext, env, self.id);
                                    }
                                })
                            })
                        })
                    })
                })
            })
        });

        self.handle_post_event(event_loop, &mut request, &mut application_manager);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WinitWindowId, event: WindowEvent) {
        let mut request = RequestRedraw::False;

        self.animation_manager.update_frame_time();

        let mut application_manager = ApplicationManager::new();

        let mouse_position = self.event_handler.mouse_position();

        self.environment.with_mut::<AnimationManager>(&mut self.animation_manager, |env| {
            env.with_mut::<WgpuContext>(&mut self.wgpu_context, |env| {
                env.with_mut::<ApplicationManager>(&mut application_manager, |env| {
                    env.with::<ActiveEventLoopKey>(event_loop, |env| {
                        env.with_mut::<FocusManager>(&mut self.focus_manager, |env| {
                            env.with::<MousePositionKey>(&mouse_position, |env| {
                                env.with::<dyn EventSink>(&self.event_sink, |env| {
                                    request = self.event_handler.window_event(&event, window_id, &mut self.scenes, &mut self.text_context, &mut WGPUImageContext, env, self.id);
                                })
                            })
                        })
                    })
                })
            })
        });

        self.handle_post_event(event_loop, &mut request, &mut application_manager);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        info!("Suspended");
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        info!("Exiting");
    }
}