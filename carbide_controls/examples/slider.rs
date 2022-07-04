use carbide_controls::{PlainSlider, Slider};
use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::{AnimatedState, F64State, LocalState, StateExt};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;
use std::time::Duration;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Slider example".to_string(),
        400,
        400,
        Some(icon_path.clone()),
    );

    let family =
        FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let progress = LocalState::new(80.0);
    let progress2 = LocalState::new(80.0);

    window.set_widgets(VStack::new(vec![
        Text::new(
            progress
                .clone()
                .map(|a: &f64| format!("Slider value: {:.2}", a))
                .ignore_writes(),
        ),
        Slider::new(progress, 20.0, 100.0).padding(20.0),
        Empty::new().frame(20.0, 20.0),
        Text::new(
            progress2
                .clone()
                .map(|a: &f64| format!("Slider step value: {:.2}", a))
                .ignore_writes(),
        ),
        Slider::new(progress2, 20.0, 100.0)
            .step(5.0)
            .accent_color(EnvironmentColor::Orange)
            .padding(20.0),
    ]));

    window.launch();
}
