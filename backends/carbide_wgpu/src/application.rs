use std::borrow::Borrow;
use std::collections::HashMap;
use winit::dpi::{LogicalSize, Size};
use winit::event::{Event, VirtualKeyCode, WindowEvent as WinitWindowEvent};
use winit::event_loop::{EventLoopWindowTarget, EventLoop as WinitEventLoop, ControlFlow};
use winit::window::{WindowBuilder, WindowId as WinitWindowId};
use carbide_core::environment::Environment;
use carbide_core::event::{CustomEvent, EventHandler, Input, WindowEvent};
use carbide_core::widget::{Empty, Rectangle, Widget};
use carbide_winit::{convert_mouse_cursor, convert_window_event};
use carbide_winit::EventLoop;
use crate::wgpu_window::WGPUWindow;
use std::cell::RefCell;
use std::mem::transmute;
use std::path::{Path, PathBuf};
use carbide_core::draw::Dimension;
use carbide_core::render::Render;
use carbide_core::{locate_folder, Scene};
use carbide_core::draw::image::ImageId;
use carbide_core::text::{FontFamily, FontId};
use crate::proxy_event_loop::ProxyEventLoop;
use carbide_core::window::WindowId;

thread_local!(pub static EVENT_LOOP: RefCell<EventLoop<CustomEvent>> = RefCell::new(EventLoop::Owned(WinitEventLoop::<CustomEvent>::with_user_event())));
thread_local!(pub static WINDOW_IDS: RefCell<HashMap<WinitWindowId, WindowId>> = RefCell::new(HashMap::new()));

pub struct Application {
    // /// This contains the whole widget tree. This includes windows and other widgets.
    root: Box<dyn Scene>,
    event_handler: EventHandler,
    environment: Environment,
    //any_focus: bool,

    //windows: HashMap<WindowId, WGPUWindow>,
}

impl Application {
    pub fn new() -> Self {

        //let window = WGPUWindow::new(Box::new(WGPUWindow::new(Rectangle::new())));
        //let window = WGPUWindow::new(Rectangle::new());

        let window_pixel_dimensions = Dimension::new(400.0, 400.0);
        let scale_factor = 2.0;

        let proxy = EVENT_LOOP.with(|a| {
            match &*a.borrow() {
                EventLoop::Owned(e) => {e.create_proxy()}
                EventLoop::StaticBorrow(_) => unreachable!(),
                EventLoop::None => unreachable!(),
            }
        });

        let event_sink = Box::new(ProxyEventLoop(proxy));

        let environment = Environment::new(
            window_pixel_dimensions,
            scale_factor,
            None,
            event_sink,
        );

        Application {
            root: Empty::new(),
            event_handler: EventHandler::new(),
            environment
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

    fn input(&mut self, event: &WinitWindowEvent, window_id: WinitWindowId) {
        let input = convert_window_event(event);
        if let Some(input) = input {
            let id = WINDOW_IDS.with(|a| *a.borrow().get(&window_id).unwrap());
            self.event_handler.compound_and_add_event(input, Some(id));
        }
    }

    fn update(&mut self) -> bool {
        // Capture the current time and update the animations in the environment.
        self.environment.capture_time();
        self.environment.update_animation();
        self.environment.clear_animation_frame();

        self.environment.check_tasks();
        self.environment.add_queued_images();

        self.event_handler.delegate_events(&mut self.root, &mut self.environment)
    }

    pub fn add_font_family(&mut self, family: FontFamily) -> String {
        let family_name = family.name.clone();
        self.environment.add_font_family(family);
        family_name
    }

    fn add_font<P: AsRef<Path>>(&mut self, path: P) -> FontId {
        let assets = locate_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let font_path = assets.join(path.as_ref());

        self.environment.insert_font_from_file(font_path).0
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    /*fn add_image_from_path(&mut self, path: &str) -> Option<ImageId> {
        let assets = locate_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let image = Image::new(assets.join(path), &self.device, &self.queue);

        let information = image.image_information();

        let id = self.image_map.insert(image);

        self.ui.environment.insert_image(id, information);

        Some(id)
    }

    fn add_image(
        &mut self,
        id: ImageId,
        image: carbide_core::image::DynamicImage,
    ) -> Option<ImageId> {
        let image = Image::new_from_dynamic(image, &self.device, &self.queue);

        let information = image.image_information();

        let id = self.image_map.insert_with_id(id, image);

        self.ui.environment.insert_image(id, information);

        Some(id)
    }*/

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

        // Make the state sync on event loop run
        //self.input(&WinitWindowEvent::Focused(true));

        event_loop.run(
            move |event, event_loop, control_flow| {
                EVENT_LOOP.with(|a| {
                    // SAFETY: We make the reference static such that we can store it in our thread_local
                    // We make sure to drop it after the reference is no longer valid, at the end of
                    // the closure after the event was propagated.
                    let event_loop: &'static EventLoopWindowTarget<CustomEvent> = unsafe {
                        transmute(event_loop)
                    };
                    *a.borrow_mut() = EventLoop::StaticBorrow(event_loop)
                });

                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id,
                    } if WINDOW_IDS.with(|a| a.borrow().contains_key(&window_id)) => {
                        self.input(event, window_id);

                        /*if !self.input(event) {
                            match event {
                                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                                WindowEvent::KeyboardInput { input, .. } => {
                                    match input {
                                        KeyboardInput {
                                            state: ElementState::Pressed,
                                            virtual_keycode: Some(VirtualKeyCode::F1),
                                            ..
                                        } => {
                                            // This is only for debugging purposes.
                                            use carbide_core::image::GrayImage;
                                            use std::fs::*;
                                            let image_folder =
                                                String::from("/tmp/carbide_img_dump_")
                                                    + &Uuid::new_v4().to_string();
                                            create_dir_all(&image_folder).unwrap();
                                            self.mesh
                                                .texture_atlas_image()
                                                .save(image_folder.clone() + "/glyph_atlas0.png")
                                                .unwrap();
                                            let atlas1 = DynamicImage::ImageLuma8(
                                                GrayImage::from_raw(
                                                    DEFAULT_GLYPH_CACHE_DIMS[0],
                                                    DEFAULT_GLYPH_CACHE_DIMS[1],
                                                    self.mesh.glyph_cache_pixel_buffer().to_vec(),
                                                )
                                                    .unwrap(),
                                            );
                                            atlas1
                                                .save(image_folder.clone() + "/glyph_atlas1.png")
                                                .unwrap();
                                            println!("Images dumped to: {}", image_folder);
                                        }
                                        KeyboardInput {
                                            state: ElementState::Pressed,
                                            virtual_keycode: Some(VirtualKeyCode::Escape),
                                            ..
                                        } => *control_flow = ControlFlow::Exit,
                                        _ => {}
                                    }
                                }
                                WindowEvent::Resized(physical_size) => {
                                    self.resize(*physical_size);
                                    self.inner_window.request_redraw();
                                }
                                WindowEvent::ScaleFactorChanged {
                                    new_inner_size,
                                    scale_factor,
                                } => {
                                    self.ui.set_scale_factor(*scale_factor);
                                    self.resize(**new_inner_size);
                                    self.inner_window.request_redraw();
                                }
                                WindowEvent::Focused(true) => {
                                    self.ui.refresh_application_menu();
                                }
                                _ => {}
                            }
                        }*/
                    }

                    // Gets called whenever we receive carbide sent events
                    Event::UserEvent(event) => {
                        println!("{:?}", event);
                        self.event_handler.compound_and_add_event(Input::Custom(event), None);
                        self.request_redraw();
                    }

                    // Gets called when all window and user events are delivered
                    Event::MainEventsCleared => {
                        // If we have any events queued up and update the UI
                        if self.event_handler.has_queued_events() || self.environment.has_animations() {
                            // If the ui should redraw because of the update
                            if self.update() || self.environment.has_animations() {
                                self.request_redraw();
                            }

                            /*self.inner_window
                                .set_cursor_icon(convert_mouse_cursor(self.ui.mouse_cursor()));*/
                        }
                    }

                    // Gets called if redrawing is requested.
                    Event::RedrawRequested(_) => {
                        self.root.process_get_primitives(&mut vec![], &mut self.environment);

                        // Wait for the next event to be received
                        *control_flow = ControlFlow::Wait;
                    }

                    // This is called after the rendering
                    Event::RedrawEventsCleared => {
                        // If we have any animations running we should draw as soon as possible next frame
                        if self.environment.has_animations() {
                            self.request_redraw();
                        }
                    }
                    _ => {}
                }

                if self.environment.should_close_application() {
                    *control_flow = ControlFlow::Exit;
                }

                EVENT_LOOP.with(|a| {
                    a.take()
                });
            },
        );
    }
}