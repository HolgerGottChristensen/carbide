use carbide_core::color::{BLACK, WHITE};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{AnyReadState, AnyState, ReadState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let colors1 = vec![
        EnvironmentColor::Accent,
        EnvironmentColor::Blue,
        EnvironmentColor::Green,
        EnvironmentColor::Indigo,
        EnvironmentColor::Orange,
        EnvironmentColor::Pink,
        EnvironmentColor::Purple,
        EnvironmentColor::Red,
        EnvironmentColor::Teal,
        EnvironmentColor::Yellow,
        EnvironmentColor::Gray,
        EnvironmentColor::Gray2,
        EnvironmentColor::Gray3,
        EnvironmentColor::Gray4,
        EnvironmentColor::Gray5,
    ];

    let colors2 = vec![
        EnvironmentColor::Gray6,
        EnvironmentColor::Label,
        EnvironmentColor::SecondaryLabel,
        EnvironmentColor::TertiaryLabel,
        EnvironmentColor::QuaternaryLabel,
        EnvironmentColor::SystemFill,
        EnvironmentColor::SecondarySystemFill,
        EnvironmentColor::TertiarySystemFill,
        EnvironmentColor::QuaternarySystemFill,
        EnvironmentColor::PlaceholderText,
        EnvironmentColor::SystemBackground,
        EnvironmentColor::SecondarySystemBackground,
        EnvironmentColor::TertiarySystemBackground,
        EnvironmentColor::Separator,
        EnvironmentColor::OpaqueSeparator,
    ];

    let colors3 = vec![
        EnvironmentColor::Link,
        EnvironmentColor::DarkText,
        EnvironmentColor::LightText,
    ];

    application.set_scene(Window::new(
        "Color preview example",
        Dimension::new(800.0, 700.0),
        ZStack::new((
            Rectangle::new().fill(BLACK),
            HStack::new(ForEach::new(vec![colors1, colors2, colors3], |item, index| {
                VStack::new(ForEach::new(item, |item: Box<dyn AnyState<T=EnvironmentColor>>, index| {
                    HStack::new((
                        Text::new(format!("{:?}", *item.value())),
                        Rectangle::new()
                            .fill(item)
                            .stroke(WHITE)
                            .stroke_style(1.0)
                            .frame(100.0, 30.0)
                    ))
                })).cross_axis_alignment(CrossAxisAlignment::End)
            })).cross_axis_alignment(CrossAxisAlignment::Start)
        ))
    ).close_application_on_window_close());

    application.launch();
}
