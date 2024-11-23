use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, Map1, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let width1 = LocalState::new(0.1);
    let percent = LocalState::new(0.1);
    let width2 = LocalState::new(0.1);

    let mut application = Application::new()
        .with_asset_fonts();


    application.set_scene(Window::new(
        "HSplit example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            h_split(width1.clone()).relative_to_start(width1),
            h_split(percent.clone()).percent(percent),
            h_split(width2.clone()).relative_to_end(width2),
        ))
    ));

    application.launch();
}

fn h_split(size: impl State<T=f64>) -> HSplit<f64, impl Widget, impl Widget> {
    HSplit::new(
        ZStack::new((
            Rectangle::new().fill(EnvironmentColor::Green),
            Text::new(Map1::read_map(size, |t: &f64| format!("{:.2}", t))).wrap(Wrap::None),
        )),
        ZStack::new((
            Rectangle::new().fill(EnvironmentColor::Accent),
            Rectangle::new()
                .fill(EnvironmentColor::Yellow)
                .frame(100.0, 100.0),
        )),
    )
}
