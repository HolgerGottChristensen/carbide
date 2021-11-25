use carbide_core::environment::EnvironmentColor;
use carbide_core::prelude::TState;
use carbide_core::state::{LocalState, StateExt};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "HSplit example".to_string(),
        400,
        800,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    let width1 = LocalState::new(0.1);
    let percent = LocalState::new(0.1);
    let width2 = LocalState::new(0.1);

    window.set_widgets(
        VStack::new(vec![
            h_split(&width1).relative_to_start(width1),
            h_split(&percent).percent(percent),
            h_split(&width2).relative_to_end(width2),
        ]),

    );

    window.launch();
}

fn h_split(size: &TState<f64>) -> Box<HSplit> {
    HSplit::new(
        ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::Green),
            Text::new(size.mapped(|t: &f64| { format!("{:.2}", t) })).wrap_mode(Wrap::None),
        ]),
        ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::Accent),
            Rectangle::new().fill(EnvironmentColor::Yellow)
                .frame(100.0, 100.0),
        ]),
    )
}
