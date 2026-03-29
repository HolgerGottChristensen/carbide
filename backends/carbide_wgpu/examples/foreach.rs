use carbide_core::color::{ColorExt, RED};
use carbide_core::draw::{Color, Dimension};
use carbide_core::environment::*;
use carbide_core::state::{AnyReadState, AnyState, IndexState, LocalState, ReadState, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new().with_asset_fonts();

    let model = vec![
        EnvironmentColor::Red,
        EnvironmentColor::Orange,
        EnvironmentColor::Yellow,
        EnvironmentColor::Green,
        EnvironmentColor::Accent,
        EnvironmentColor::Purple,
    ];

    application.set_scene(Window::new(
        "ForEach example - Carbide",
        Dimension::new(600.0, 450.0),
        VStack::new(
            /*ForEach::new(LocalState::new(model), |item: Box<dyn AnyState<T=EnvironmentColor>>, index| {
                ZStack::new((
                    Rectangle::new().fill(item.clone()),
                    Text::new(index)
                        .color(item.color().plain_contrast())
                        .font_size(EnvironmentFontSize::Title),
                )).frame(100.0, 50.0)
            })*/
            ForEach::new(0..3, |a, b| {
                ForEach::new(0..2, |c, d| {
                    Rectangle::new()
                        .fill(Color::random())
                        .frame(100.0, 50.0)
                        .accent_color(RED)
                })
            })
        ),
    ));

    application.launch();
}
