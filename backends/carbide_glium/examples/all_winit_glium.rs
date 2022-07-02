//! A demonstration using winit to provide events and glium for drawing the Ui.
extern crate carbide_core;
extern crate carbide_example_shared;
extern crate carbide_glium;
extern crate carbide_winit;
extern crate find_folder;
extern crate glium;
extern crate image;

mod support;

use carbide_example_shared::{WIN_H, WIN_W};
use carbide_glium::Renderer;
use glium::Surface;

fn main() {
    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("carbide with glium!")
        .with_dimensions((WIN_W, WIN_H).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = support::GliumDisplayWinitWrapper(display);

    // Construct our `Ui`.
    let mut ui = carbide_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
        .theme(carbide_example_shared::theme())
        .build();

    // The `widget::Id` of each widget instantiated in `carbide_example_shared::gui`.
    let ids = carbide_example_shared::Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Load the Rust logo from our assets folder to use as an example image.
    fn load_rust_logo(display: &glium::Display) -> glium::texture::Texture2d {
        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets")
            .unwrap();
        let path = assets.join("images/rust.png");
        let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
        let image_dimensions = rgba_image.dimensions();
        let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(
            &rgba_image.into_raw(),
            image_dimensions,
        );
        let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
        texture
    }

    let mut image_map = carbide_core::draw::image::image_map::ImageMap::new();
    let rust_logo = image_map.insert(load_rust_logo(&display.0));

    // A demonstration of some app state that we want to control with the carbide GUI.
    let mut app = carbide_example_shared::DemoApp::new(rust_logo);

    // A type used for converting `carbide_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    //
    // Internally, the `Renderer` maintains:
    // - a `backend::glium::GlyphCache` for caching text onto a `glium::texture::Texture2d`.
    // - a `glium::Program` to use as the shader program when drawing to the `glium::Surface`.
    // - a `Vec` for collecting `backend::glium::Vertex`s generated when translating the
    // `carbide_core::render::Primitive`s.
    // - a `Vec` of commands that describe how to draw the vertices.
    let mut renderer = Renderer::new(&display.0).unwrap();

    // Start the loop:
    //
    // - Poll the window for available events.
    // - Update the widgets via the `carbide_example_shared::gui` fn.
    // - Render the current state of the `Ui`.
    // - Repeat.
    let mut event_loop = support::EventLoop::new();
    'main: loop {
        // Handle all events.
        for event in event_loop.next(&mut events_loop) {
            // Use the `winit` backend feature to convert the winit event to a carbide one.
            if let Some(event) = support::convert_event(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested
                    | glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        // Instantiate a GUI demonstrating every widget type provided by carbide.
        carbide_example_shared::gui(&mut ui.set_widgets(), &ids, &mut app);

        // Draw the `Ui`.
        if let Some((primitives, cprims)) = ui.draw_if_changed() {
            renderer.fill(&display.0, primitives, &image_map);
            let mut target = display.0.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display.0, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
