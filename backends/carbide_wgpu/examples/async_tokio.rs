use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;

use carbide_core::state::{LocalState, State};
use carbide_core::task;
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn main() {
    env_logger::init();

    let mut application = Application::new()
        .with_asset_fonts();

    let image_id = LocalState::new(None);
    let image_id_for_async = image_id.clone();

    async fn hello() -> f64 {
        100.0
    }

    let block_width = LocalState::new(50.0);
    let text = LocalState::new("Hello World!".to_string());

    let env = application.environment_mut();

    task!(env, block_width := {
        sleep(Duration::new(2, 0)).await;
        hello().await
    });

    let ipsum_path = Application::assets().join("ipsum.txt");

    task!(env, text := {
        sleep(Duration::new(1, 0)).await;
        tokio::fs::read_to_string(ipsum_path).await.unwrap()
    });

    task!(
        env,
        {
            let client = reqwest::Client::builder()
                .user_agent(APP_USER_AGENT)
                .build().unwrap();

            //println!("{:?}", client);

            let response = client.get("https://picsum.photos/300")
                .send()
                .await
                .unwrap();

            //.text().await;

            //println!("{:#?}", response);

            let data = response.bytes()
                .await
                .expect("Could not get bytes");
            let image = carbide_core::image::load_from_memory(&data).unwrap();
            image
        },
        move |res, env: &mut Environment| {
            image_id_for_async.clone().set_value(env.queue_image(PathBuf::new().join("ThisIsNotValid"), res))
        }
    );

    let widgets = VStack::new(vec![
        Text::new(text).padding(20.0),
        Image::new(image_id),
        Rectangle::new()
            .fill(EnvironmentColor::Accent)
            .frame(block_width, 50),
    ])
        .accent_color(EnvironmentColor::Red);

    application.set_scene(
        Window::new(
            "Async using tokio example",
            Dimension::new(400.0, 600.0),
            widgets
        ).close_application_on_window_close()
    );

    application.launch();
}
