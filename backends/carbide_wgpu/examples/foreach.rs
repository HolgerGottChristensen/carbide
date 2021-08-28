use std::time::Duration;

use carbide_core::Color;
use carbide_core::environment::*;
use carbide_core::state::{AnimatedState, ColorState, ease_in_out, State, TState};
use carbide_core::text::*;
use carbide_core::widget::*;
use carbide_core::widget::canvas::*;
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
                *Rectangle::new(vec![])
                    .fill(item.value().clone())
                    .frame(100.0, 50.0)
            })
        ]).spacing(10.0)
    );

    window.launch();
}
