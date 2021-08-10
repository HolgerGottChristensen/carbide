//! This example behaves the same as the `all_winit_glium` example while demonstrating how to run
//! the `carbide` loop on a separate thread.

extern crate carbide_core;
extern crate carbide_example_shared;
extern crate carbide_glium;
#[macro_use]
extern crate carbide_winit;
extern crate find_folder;
extern crate glium;
extern crate image;

use glium::Surface;

use carbide_example_shared::{WIN_H, WIN_W};
use carbide_glium::Renderer;

mod support;

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

    // Load the Rust logo from our assets folder to use as an example image.
    fn load_rust_logo(display: &glium::Display) -> glium::texture::Texture2d {
        let assets = find_folder::Search::ParentsThenKids(5, 3)
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

    let mut image_map = carbide_core::image_map::ImageMap::new();
    let rust_logo = image_map.insert(load_rust_logo(&display.0));

    // A channel to send events from the main `winit` thread to the carbide thread.
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    // A channel to send `render::Primitive`s from the carbide thread to the `winit thread.
    let (render_tx, render_rx) = std::sync::mpsc::channel();
    // Clone the handle to the events loop so that we can interrupt it when ready to draw.
    let events_loop_proxy = events_loop.create_proxy();

    // A function that runs the carbide loop.
    fn run_carbide(
        rust_logo: carbide_core::image_map::Id,
        event_rx: std::sync::mpsc::Receiver<carbide_core::event::Input>,
        render_tx: std::sync::mpsc::Sender<carbide_core::render::OwnedPrimitives>,
        events_loop_proxy: glium::glutin::EventsLoopProxy,
    ) {
        // Construct our `Ui`.
        let mut ui = carbide_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
            .theme(carbide_example_shared::theme())
            .build();

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // A demonstration of some app state that we want to control with the carbide GUI.
        let mut app = carbide_example_shared::DemoApp::new(rust_logo);

        // The `widget::Id` of each widget instantiated in `carbide_example_shared::gui`.
        let ids = carbide_example_shared::Ids::new(ui.widget_id_generator());

        // Many widgets require another frame to finish drawing after clicks or hovers, so we
        // insert an update into the carbide loop using this `bool` after each event.
        let mut needs_update = true;
        'carbide: loop {
            // Collect any pending events.
            let mut events = Vec::new();
            while let Ok(event) = event_rx.try_recv() {
                events.push(event);
            }

            // If there are no events pending, wait for them.
            if events.is_empty() || !needs_update {
                match event_rx.recv() {
                    Ok(event) => events.push(event),
                    Err(_) => break 'carbide,
                };
            }

            needs_update = false;

            // Input each event into the `Ui`.
            for event in events {
                ui.handle_event(event);
                needs_update = true;
            }

            // Instantiate a GUI demonstrating every widget type provided by carbide.
            carbide_example_shared::gui(&mut ui.set_widgets(), &ids, &mut app);

            // Render the `Ui` to a list of primitives that we can send to the main thread for
            // display. Wakeup `winit` for rendering.
            if let Some((primitives, cprims)) = ui.draw_if_changed() {
                if render_tx.send(primitives.owned()).is_err()
                    || events_loop_proxy.wakeup().is_err()
                {
                    break 'carbide;
                }
            }
        }
    }

    // Draws the given `primitives` to the given `Display`.
    fn draw(
        display: &glium::Display,
        renderer: &mut Renderer,
        image_map: &carbide_core::image_map::ImageMap<glium::Texture2d>,
        primitives: &carbide_core::render::OwnedPrimitives,
    ) {
        renderer.fill(display, primitives.walk(), &image_map);
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        renderer.draw(display, &mut target, &image_map).unwrap();
        target.finish().unwrap();
    }

    // Spawn the carbide loop on its own thread.
    std::thread::spawn(move || run_carbide(rust_logo, event_rx, render_tx, events_loop_proxy));

    // Run the `winit` loop.
    let mut last_update = std::time::Instant::now();
    let mut closed = false;
    while !closed {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least
        // 16ms since the last yield.
        let sixteen_ms = std::time::Duration::from_millis(16);
        let now = std::time::Instant::now();
        let duration_since_last_update = now.duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        events_loop.run_forever(|event| {
            // Use the `winit` backend feature to convert the winit event to a carbide one.
            if let Some(event) = support::convert_event(event.clone(), &display) {
                event_tx.send(event).unwrap();
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
                    } => {
                        closed = true;
                        return glium::glutin::ControlFlow::Break;
                    }
                    // We must re-draw on `Resized`, as the event loops become blocked during
                    // resize on macOS.
                    glium::glutin::WindowEvent::Resized(..) => {
                        if let Some(primitives) = render_rx.iter().next() {
                            draw(&display.0, &mut renderer, &image_map, &primitives);
                        }
                    }
                    _ => {}
                },
                glium::glutin::Event::Awakened => return glium::glutin::ControlFlow::Break,
                _ => (),
            }

            glium::glutin::ControlFlow::Continue
        });

        // Draw the most recently received `carbide_core::render::Primitives` sent from the `Ui`.
        if let Some(primitives) = render_rx.try_iter().last() {
            draw(&display.0, &mut renderer, &image_map, &primitives);
        }

        last_update = std::time::Instant::now();
    }
}
