use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, StateExt, TState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};
use carbide_core::state::ReadStateExtNew;

fn main() {
    let height1 = LocalState::new(0.1);
    let percent = LocalState::new(0.1);
    let height2 = LocalState::new(0.1);

    let mut application = Application::new()
        .with_asset_fonts();


    application.set_scene(Window::new(
        "VSplit example",
        Dimension::new(600.0, 400.0),
        *HStack::new(vec![
            v_split(&height1).relative_to_start(height1),
            v_split(&percent).percent(percent),
            v_split(&height2).relative_to_end(height2),
        ])
    ).close_application_on_window_close());

    application.launch();
}

fn v_split(size: &TState<f64>) -> Box<VSplit<f64>> {
    VSplit::new(
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
