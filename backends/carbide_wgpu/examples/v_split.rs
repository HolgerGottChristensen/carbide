use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, StateExt};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "VSplit example".to_string(),
        800,
        1200,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    let percent = LocalState::new(0.1);

    window.set_widgets(
        VSplit::new(
            ZStack::new(vec![
                Rectangle::new().fill(EnvironmentColor::Green),
                Text::new(percent.mapped(|t: &f64| { format!("{:.2}", t) })).wrap_mode(Wrap::None),
            ]),
            ZStack::new(vec![
                Rectangle::new().fill(EnvironmentColor::Accent),
                Rectangle::new().fill(EnvironmentColor::Yellow)
                    .frame(100.0, 100.0),
            ]),
        ).percent(percent)
    );

    window.launch();
}
