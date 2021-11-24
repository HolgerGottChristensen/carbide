use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, StateExt, TState};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "VSplit example".to_string(),
        1200,
        800,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    let height1 = LocalState::new(0.1);
    let percent = LocalState::new(0.1);
    let height2 = LocalState::new(0.1);

    window.set_widgets(
        HStack::new(vec![
            v_split(&height1).relative_to_start(height1),
            v_split(&percent).percent(percent),
            v_split(&height2).relative_to_end(height2),
        ]),
    );

    window.launch();
}

fn v_split(size: &TState<f64>) -> Box<VSplit> {
    VSplit::new(
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