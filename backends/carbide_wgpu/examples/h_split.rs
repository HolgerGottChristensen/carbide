use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, StateExt, TState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};
use carbide_core::state::ReadStateExtNew;

fn main() {
    let width1 = LocalState::new(0.1);
    let percent = LocalState::new(0.1);
    let width2 = LocalState::new(0.1);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "HSplit example",
        Dimension::new(400.0, 600.0),
        *VStack::new(vec![
            h_split(&width1).relative_to_start(width1),
            h_split(&percent).percent(percent),
            h_split(&width2).relative_to_end(width2),
        ])
    ).close_application_on_window_close());

    application.launch();
}

fn h_split(size: &TState<f64>) -> Box<HSplit<f64>> {
    HSplit::new(
        ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::Green),
            Text::new(size.map(|t: &f64| format!("{:.2}", t))).wrap_mode(Wrap::None),
        ]),
        ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::Accent),
            Rectangle::new()
                .fill(EnvironmentColor::Yellow)
                .frame(100.0, 100.0)
                .boxed(),
        ]),
    )
}
