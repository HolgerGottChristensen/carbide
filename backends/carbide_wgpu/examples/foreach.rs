use carbide_core::environment::*;
use carbide_core::state::{State, TState};
use carbide_core::text::{FontFamily, FontStyle, FontWeight};
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Foreach example".to_string(),
        1200,
        900,
        Some(icon_path.clone()),
    );

    let mut family = FontFamily::new("NotoSans");
    family.add_font(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    family.add_font(
        "fonts/NotoSans/NotoSans-Italic.ttf",
        FontWeight::Normal,
        FontStyle::Italic,
    );
    family.add_font(
        "fonts/NotoSans/NotoSans-Bold.ttf",
        FontWeight::Bold,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    window.set_widgets(
        VStack::new(vec![
            ForEach::new(vec![
                EnvironmentColor::Red,
                EnvironmentColor::Orange,
                EnvironmentColor::Yellow,
                EnvironmentColor::Green,
                EnvironmentColor::Accent,
                EnvironmentColor::Purple,
            ], |item: TState<EnvironmentColor>, index| {
                *Rectangle::new(vec![
                    Text::new(index)
                        .font_size(EnvironmentFontSize::LargeTitle)
                ])
                    .fill(item.value().clone())
                    .frame(100.0, 50.0)
            })
        ]).spacing(10.0)
    );

    window.launch();
}
