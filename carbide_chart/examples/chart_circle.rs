use std::f64::consts::PI;
use carbide_chart::{Chart, DataColor, DatasetExt, ScatterController};
use carbide_core::color::BLUE;
use carbide_core::draw::{Dimension, Position};
use carbide_core::state::ValueState;
use carbide_core::widget::WidgetExt;
use carbide_fluent::{locale, LocaleExt};
use carbide_wgpu::{Application, Window};


fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let data = circle(100).color(DataColor::Color(BLUE));

    application.set_scene(
        Window::new(
            "Chart circle example - Carbide",
            Dimension::new(700.0, 700.0),
            Chart::new(ScatterController::new(data))
                .border()
                .padding(50.0)
                .locale(ValueState::new(locale!("da")))
        ));

    application.launch();
}

/// Create a circle of n * 2 + 1 points.
fn circle(n: usize) -> Vec<Position> {
    (0..n * 2 + 1)
        .into_iter()
        .map(|a| {
            Position::new(
                f64::sin(a as f64 / n as f64 * PI),
                f64::cos(a as f64 / n as f64 * PI)
            )
        })
        .collect::<Vec<_>>()
}