use std::f64::consts::PI;
use std::iter;
use carbide_chart::{Chart, ScatterController};
use carbide_core::draw::{Dimension, Position, Scalar};
use carbide_core::state::{Functor, ValueState};
use carbide_core::widget::WidgetExt;
use carbide_fluent::{locale, LocaleExt};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let data1 = iter::repeat_with(|| {
        Position::new(rand::random::<Scalar>() * 100.0, rand::random::<Scalar>())
    }).take(40).collect::<Vec<_>>();

    let data4 = iter::repeat_with(|| {
        Position::new(rand::random::<Scalar>(), rand::random::<Scalar>())
    }).take(40).collect::<Vec<_>>();

    let data5 = iter::repeat_with(|| {
        Position::new(rand::random::<Scalar>(), rand::random::<Scalar>())
    }).take(40).collect::<Vec<_>>();

    let data2 = iter::repeat_with(|| {
        rand::random::<Scalar>()
    }).take(40).collect::<Vec<_>>();

    let data3 = (0..100)
        .into_iter()
        .map(|a| a as Scalar / 10.0)
        .map(|a| f64::sin(a))
        .collect::<Vec<_>>();

    let data6 = (0..100)
        .into_iter()
        .map(|a| a as Scalar / 10.0)
        .map(|a| f64::cos(a))
        .collect::<Vec<_>>();

    application.set_scene(
        Window::new(
            "Chart example - Carbide",
            Dimension::new(700.0, 700.0),
            Chart::new(ScatterController::new((data1, data3, data6)))
                .border()
                .padding(50.0)
                .locale(ValueState::new(locale!("da")))
        ).close_application_on_window_close());

    application.launch();
}