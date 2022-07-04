use carbide_core::environment::*;
use carbide_core::state::{ReadState, State, TState, UsizeState};
use carbide_core::text::{FontFamily, FontStyle, FontWeight};
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Foreach example".to_string(),
        600,
        450,
        Some(icon_path.clone()),
    );

    let mut family = FontFamily::new("NotoSans");
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Italic.ttf",
        FontWeight::Normal,
        FontStyle::Italic,
    );
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Bold.ttf",
        FontWeight::Bold,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    fn delegate(item: TState<EnvironmentColor>, index: UsizeState) -> Box<dyn Widget> {
        ZStack::new(vec![
            Rectangle::new().fill(item.value().clone()),
            Text::new(index).font_size(EnvironmentFontSize::LargeTitle),
        ])
        .frame(100.0, 50.0)
    }

    window.set_widgets(
        VStack::new(vec![ForEach::new(
            vec![
                EnvironmentColor::Red,
                EnvironmentColor::Orange,
                EnvironmentColor::Yellow,
                EnvironmentColor::Green,
                EnvironmentColor::Accent,
                EnvironmentColor::Purple,
            ],
            delegate,
        )])
        .spacing(10.0),
    );

    window.launch();
}
