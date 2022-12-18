use carbide_controls::List;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, TState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let list_model = (1..100)
        .map(|i| format!("Number {}", i))
        .collect::<Vec<_>>();

    let list_model_state = LocalState::new(list_model);

    fn delegate(item: TState<String>, _: TState<usize>) -> Box<dyn Widget> {
        ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::Green),
            Text::new(item),
        ])
        .frame_fixed_height(80.0)
    }

    application.set_scene(Window::new(
        "List Example - Carbide",
        Dimension::new(400.0, 600.0),
        List::new(list_model_state, delegate).clip().padding(50.0)
    ).close_application_on_window_close());

    application.launch();
}
