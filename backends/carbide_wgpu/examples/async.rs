use std::thread::sleep;
use std::time::Duration;

use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::{Listenable, LocalState, NewMapState, ReadState, State, StateExt, ValueState};
use carbide_core::{Color, task};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Async example".to_string(),
        800,
        1200,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    //let image_id = LocalState::new(None);
    //let image_id_for_async = image_id.clone();

    async fn hello() -> f64 {
        100.0
    }

    let block_width = LocalState::new(50.0);

    let new_state = NewMapState::new(
        block_width.clone(),
        |x: &f64| {*x * 2.0},
        |x: f64, y: &f64| {x / 2.0});
    block_width.subscribe(Box::new(|a: &f64| {
        println!("Value changed: {}", a)
    }));

    let new_state1 = block_width.read_map(|x: &f64| {*x * 3.0});
    let new_state2 = new_state1.read_map(|x: &f64| {*x * 1.2});
    //let text = LocalState::new("Hello World!".to_string());

    let env = window.environment_mut();

    task!(env, block_width := {
        sleep(Duration::new(1, 0));
        hello().await
    });

    task!(env, new_state := {
        sleep(Duration::new(3, 0));
        hello().await
    });

    //let ipsum_path = Window::relative_path_to_assets("ipsum.txt");

    /*task!(env, text := {
        async_std::task::sleep(Duration::new(1, 0)).await;
        async_std::fs::read_to_string(ipsum_path).await.unwrap()
    });

    task!(env, {
        let client = surf::Client::new().with(surf::middleware::Redirect::new(5));
        let response = client.recv_bytes(surf::get("https://picsum.photos/300")).await.unwrap();
        let image = image::load_from_memory(&response).unwrap();
        image
    }, move |res, env: &mut Environment| {
        image_id_for_async.clone().set_value(env.queue_image(res))
    });*/

    let random_color = 10.mapped(|_: &i32| {
        Color::random()
    });

    window.set_widgets(
        VStack::new(vec![
            //Text::new(text)
            //    .padding(20.0),
            //Image::new(image_id),
            Rectangle::new()
                .fill(random_color)
                .frame(block_width, 50),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(new_state, 50),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(new_state1, 50),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(new_state2, 50),
        ]).accent_color(EnvironmentColor::Red)
    );

    window.launch();
}
