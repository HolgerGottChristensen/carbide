extern crate find_folder;
extern crate glium;
extern crate image;

use std;

use glium::backend::glutin::Display;
use glium::glutin::WindowBuilder;
use glium::Surface;

use carbide_core::{Colorable, Positionable, Ui, UiBuilder, UiCell, widget};
use carbide_core::text_old::font::{Error, Id};
use carbide_core::widget::primitive::Widget;
use carbide_winit::WinitWindow;

use crate::Renderer;

pub struct Window<S: 'static + Clone> {
    title: String,
    width: u32,
    height: u32,
    vsync: bool,
    multisampling: u32,
    event_loop: glium::glutin::EventsLoop,
    display: GliumDisplayWinitWrapper,
    ui: Ui<S>,
    renderer: Renderer,
    image_map: carbide_core::draw::image::image_map::ImageMap<glium::texture::Texture2d>,
    pub widgets: Option<Box<dyn Fn(&mut UiCell<S>) -> ()>>,
    pub state: S,
}

impl<S: 'static + Clone> Window<S> {
    pub fn new(title: String, width: u32, height: u32, state: S) -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        let window = WindowBuilder::new()
            .with_title(title.clone())
            .with_dimensions((width, height).into());
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = Display::new(window, context, &events_loop).unwrap();
        let display = GliumDisplayWinitWrapper(display);

        let ui = carbide_core::UiBuilder::new([width as f64, height as f64]).build();

        // A type used for converting `carbide_core::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let renderer = Renderer::new(&display.0).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = carbide_core::draw::image::image_map::ImageMap::<glium::texture::Texture2d>::new();

        Window {
            title,
            width,
            height,
            vsync: true,
            multisampling: 4,
            event_loop: events_loop,
            display,
            ui,
            renderer,
            image_map,
            widgets: None,
            state,
        }
    }

    pub fn add_font(&mut self, path: &str) -> Result<Id, Error> {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let font_path = assets.join(path);
        self.ui.environment.insert_font_from_file(font_path)
    }

    pub fn add_image(&mut self, path: &str) -> Result<carbide_core::draw::image::image_map::ImageId, Error> {
        let assets = find_folder::Search::ParentsThenKids(5, 3)
            .for_folder("assets")
            .unwrap();
        let path = assets.join(path);
        let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
        let image_dimensions = rgba_image.dimensions();
        let raw_image =
            glium::texture::RawImage2d::from_raw_rgba(rgba_image.into_raw(), image_dimensions);
        let texture = glium::texture::Texture2d::new(&self.display.0, raw_image).unwrap();
        Ok(self.image_map.insert(texture))
    }

    pub fn set_widgets(&mut self, w: Box<dyn Widget<S>>) {
        self.ui.widgets = w
    }

    pub fn draw(&mut self) {
        let mut events = Vec::new();

        'render: loop {
            events.clear();

            // Get all the new events since the last frame.
            self.event_loop.poll_events(|event| {
                events.push(event);
            });

            // If there are no new events, wait for one.
            if events.is_empty() {
                self.event_loop.run_forever(|event| {
                    events.push(event);
                    glium::glutin::ControlFlow::Break
                });
            }

            // Process the events.
            for event in events.drain(..) {
                // Break from the loop upon `Escape` or closed window.
                match event.clone() {
                    glium::glutin::Event::WindowEvent { event, .. } => match event {
                        glium::glutin::WindowEvent::CloseRequested
                        | glium::glutin::WindowEvent::KeyboardInput {
                            input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => break 'render,
                        _ => (),
                    },
                    _ => (),
                };

                // Use the `winit` backend feature to convert the winit event to a carbide input.
                let input = match convert_event(event, &self.display) {
                    None => continue,
                    Some(input) => input,
                };

                // Handle the input with the `Ui`.
                self.ui.compound_and_add_event(input, &mut self.state);

                // Set the widgets.
                let ui = &mut self.ui.set_widgets();

                match &self.widgets {
                    None => (),
                    Some(n) => {
                        n.as_ref()(ui);
                    }
                }
            }

            // Draw the `Ui` if it has changed.
            if let Some((_primitives, cprims)) = self.ui.draw_if_changed() {
                self.renderer.fill(&self.display.0, cprims, &self.image_map);
                let mut target = self.display.0.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                self.renderer
                    .draw(&self.display.0, &mut target, &self.image_map)
                    .unwrap();
                target.finish().unwrap();
            }
        }
    }
}

pub struct GliumDisplayWinitWrapper(pub glium::Display);

impl WinitWindow for GliumDisplayWinitWrapper {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        self.0.gl_window().get_inner_size().map(Into::into)
    }
    fn hidpi_factor(&self) -> f32 {
        self.0.gl_window().get_hidpi_factor() as _
    }
}

/// In most of the examples the `glutin` crate is used for providing the window context and
/// events while the `glium` crate is used for displaying `carbide_core::render::Primitives` to the
/// screen.
///
/// This `Iterator`-like type simplifies some of the boilerplate involved in setting up a
/// glutin+glium event loop that works efficiently with carbide.
pub struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(
        &mut self,
        events_loop: &mut glium::glutin::EventsLoop,
    ) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events_loop.poll_events(|event| events.push(event));

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events_loop.run_forever(|event| {
                events.push(event);
                glium::glutin::ControlFlow::Break
            });
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
}

// Conversion functions for converting between render from glium's version of `winit` and
// `carbide_core`.
carbide_winit::conversion_fns!();
