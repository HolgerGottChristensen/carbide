use std::thread::sleep;
use std::time::Duration;

use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::{LocalState, Map1, State, StateExt};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::{task, Color};
use carbide_core::draw::Dimension;
use carbide_wgpu::{Application, Window};

fn main() {
    env_logger::init();

    let mut application = Application::new();

    fn window(child: Box<dyn Widget>) -> Box<Window> {
        Window::new(
            "Async example",
            Dimension::new(400.0, 600.0),
            child
        ).close_application_on_window_close()
    }

    let family =
        FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);

    application.add_font_family(family);

    //let image_id = LocalState::new(None);
    //let image_id_for_async = image_id.clone();

    async fn hello() -> f64 {
        100.0
    }

    let block_width = LocalState::new(50.0);

    let new_state = Map1::map(
        block_width.clone(),
        |x: &f64| *x * 2.0,
        |x: f64, _: &f64| Some(x / 2.0),
    );

    let new_state1 = Map1::read_map(block_width.clone(), |x: &f64| *x * 3.0);
    let new_state2 = Map1::read_map(new_state1.clone(), |x: &f64| *x * 1.2);
    //let text = LocalState::new("Hello World!".to_string());

    let env = application.environment_mut();

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

    let random_color = 10.mapped(|_: &i32| Color::random());

    let widgets = VStack::new(vec![
        //Text::new(text)
        //    .padding(20.0),
        //Image::new(image_id),
        Rectangle::new().fill(random_color).frame(block_width, 50),
        Rectangle::new()
            .fill(EnvironmentColor::Accent)
            .frame(new_state, 50),
        Rectangle::new()
            .fill(EnvironmentColor::Accent)
            .frame(new_state1.ignore_writes(), 50),
        Rectangle::new()
            .fill(EnvironmentColor::Accent)
            .frame(new_state2.ignore_writes(), 50),
    ])
        .accent_color(EnvironmentColor::Red);

    application.set_scene(
        window(widgets)
    );

    application.launch()
}
