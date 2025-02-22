use std::time::Duration;

use carbide_core::asynchronous::sleep;
use carbide_core::color::WHITE;
use carbide_core::draw::{Color, Dimension};
use carbide_core::environment::EnvironmentColor;
use carbide_core::render::Style;
use carbide_core::state::ReadStateExtNew;
use carbide_core::state::{LocalState, Map1, State};
use carbide_core::task;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    async fn hello() -> f64 {
        100.0
    }

    let block_width = LocalState::new(50.0);

    let new_state = Map1::map(
        block_width.clone(),
        |x: &f64| *x * 2.0,
        |x: f64, mut val| {
            *val = x / 2.0;
        },
    );

    let new_state1 = Map1::read_map(block_width.clone(), |x: &f64| *x * 3.0).ignore_writes();
    let new_state2 = Map1::read_map(new_state1.clone(), |x: &f64| *x * 1.2).ignore_writes();

    task!(block_width := {
        sleep(Duration::new(1, 0)).await;
        hello().await
    });

    task!(new_state := {
        sleep(Duration::new(3, 0)).await;
        hello().await
    });

    let random_color = WHITE.map(|_| Style::Color(Color::random()));

    let widgets = VStack::new((
        Rectangle::new()
            .fill(random_color)
            .frame(block_width, 50.0),
        Rectangle::new()
            .fill(EnvironmentColor::Accent)
            .frame(new_state, 50.0),
        Rectangle::new()
            .fill(EnvironmentColor::Accent)
            .frame(new_state1, 50.0),
        Rectangle::new()
            .fill(EnvironmentColor::Accent)
            .frame(new_state2, 50.0),
    ))
        .accent_color(EnvironmentColor::Red);

    application.set_scene(
        Window::new(
            "Async example",
            Dimension::new(400.0, 600.0),
            widgets
        )
    );

    application.launch()
}
