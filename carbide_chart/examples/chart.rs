use std::iter;

use carbide_chart::{Chart, DatasetExt, ScatterController, Stepped};
use carbide_controls::{ControlsExt, PopUpButton};
use carbide_core::draw::{Dimension, Position, Scalar};
use carbide_core::state::{LocalState, ValueState};
use carbide_core::widget::{VStack, WidgetExt};
use carbide_fluent::{locale, LocaleExt};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let stepped = LocalState::new(Stepped::None);

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

    let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0].stepped(stepped.clone());
    let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

    application.set_scene(
        Window::new(
            "Chart example - Carbide",
            Dimension::new(700.0, 700.0),
            VStack::new((
                //Chart::new(ScatterController::new(data2))
                Chart::new(ScatterController::new((data7, data8)))
                    //Chart::new(ScatterController::new((data3, data6)))
                    .border()
                    .locale(ValueState::new(locale!("da"))),
                PopUpButton::new(stepped.clone(), vec![Stepped::None, Stepped::Before, Stepped::After, Stepped::Middle, Stepped::MiddleVertical])
                    .label("Stepped: ")
            )).padding(50.0)
        ));

    application.launch();
}