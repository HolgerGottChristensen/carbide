use tokio::time::{sleep, Duration};

use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::{LocalState, State, ValueState};
use carbide_core::task;
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Async example".to_string(),
        400,
        600,
        Some(icon_path.clone()),
    );

    let family =
        FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let image_id = LocalState::new(None);
    let image_id_for_async = image_id.clone();

    async fn hello() -> f64 {
        100.0
    }

    let block_width = LocalState::new(50.0);
    let text = LocalState::new("Hello World!".to_string());

    let env = window.environment_mut();

    task!(env, block_width := {
        sleep(Duration::new(2, 0)).await;
        hello().await
    });

    let ipsum_path = Window::relative_path_to_assets("ipsum.txt");

    task!(env, text := {
        sleep(Duration::new(1, 0)).await;
        tokio::fs::read_to_string(ipsum_path).await.unwrap()
    });

    task!(
        env,
        {
            let response = reqwest::get("https://picsum.photos/300")
                .await
                .unwrap()
                .bytes()
                .await
                .expect("Could not get bytes");
            let image = carbide_core::image::load_from_memory(&response).unwrap();
            image
        },
        move |res, env: &mut Environment| {
            image_id_for_async.clone().set_value(env.queue_image(res))
        }
    );

    window.set_widgets(
        VStack::new(vec![
            Text::new(text).padding(20.0),
            Image::new(image_id),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(block_width, 50),
        ])
        .accent_color(EnvironmentColor::Red),
    );

    window.launch();
}
