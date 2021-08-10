#[macro_use]
extern crate carbide_core;
extern crate carbide_glium;
extern crate carbide_winit;
extern crate find_folder;
extern crate glium;

use glium::Surface;

mod support;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 300;

fn main() {
    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("carbide with glium!")
        .with_dimensions((WIDTH, HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = support::GliumDisplayWinitWrapper(display);

    // Construct our `Ui`.
    let mut ui = carbide_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    widget_ids!(struct Ids { canvas, file_navigator });
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `carbide_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = carbide_glium::Renderer::new(&display.0).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = carbide_core::image_map::ImageMap::<glium::texture::Texture2d>::new();

    let directory = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("carbide")
        .unwrap();

    // Poll events from the window.
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

        // Instantiate the carbide widgets.
        {
            use carbide_core::{widget, Colorable, OldWidget, Positionable, Sizeable};
            let ui = &mut ui.set_widgets();

            widget::Canvas::new()
                .color(carbide_core::color::DARK_CHARCOAL)
                .set(ids.canvas, ui);

            // Navigate the carbide directory only showing `.rs` and `.toml` files.
            for event in widget::FileNavigator::with_extension(&directory, &["rs", "toml"])
                .color(carbide_core::color::LIGHT_BLUE)
                .font_size(16)
                .wh_of(ids.canvas)
                .middle_of(ids.canvas)
                //.show_hidden_files(true)  // Use this to show hidden files
                .set(ids.file_navigator, ui)
            {
                println!("{:?}", event);
            }
        }

        // Render the `Ui` and then display it on the screen.
        if let Some((primitives, cprims)) = ui.draw_if_changed() {
            renderer.fill(&display.0, primitives, &image_map);
            let mut target = display.0.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display.0, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
