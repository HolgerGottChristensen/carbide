use std::iter;
use carbide_chart::Chart;
use carbide_core::draw::{Dimension, Position, Scalar};
use carbide_core::widget::WidgetExt;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(
        Window::new(
            "Chart example - Carbide",
            Dimension::new(700.0, 700.0),
            Chart::scatter(iter::repeat_with(|| {
                Position::new((rand::random::<Scalar>() * 200.0).round() / 10.0, (rand::random::<Scalar>() * 1000.0).round() / 10.0)
            }).take(20).collect())
                .border()
                .padding(50.0)
        ).close_application_on_window_close());

    application.launch();
}