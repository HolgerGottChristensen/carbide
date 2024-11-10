use carbide_core::color::ColorExt;
use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::{ReadState, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new().with_asset_fonts();

    fn delegate(item: impl State<T=EnvironmentColor>, index: impl ReadState<T=usize>) -> impl Widget {
        ZStack::new((
            Rectangle::new().fill(item.clone()),
            Text::new(index)
                .color(item.color().plain_contrast())
                .font_size(EnvironmentFontSize::Title),
        ))
        .frame(100.0, 50.0)
    }

    application.set_scene(Window::new(
        "ForEach example - Carbide",
        Dimension::new(600.0, 450.0),
        VStack::new(
            ForEach::new(
                vec![
                    EnvironmentColor::Red,
                    EnvironmentColor::Orange,
                    EnvironmentColor::Yellow,
                    EnvironmentColor::Green,
                    EnvironmentColor::Accent,
                    EnvironmentColor::Purple,
                ],
                delegate,
            )
        ).spacing(10.0),
    ));

    application.launch();
}
