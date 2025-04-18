use carbide_core::time::*;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::AnimatedState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let progress = AnimatedState::linear()
        .repeat()
        .duration(Duration::from_secs(5))
        .range(0.0, 1.0);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Progress bar example - Carbide",
        Dimension::new(400.0, 400.0),
        ProgressBar::new(progress)
            .padding(20.0)
            .accent_color(EnvironmentColor::Yellow),
    ));

    application.launch();
}
