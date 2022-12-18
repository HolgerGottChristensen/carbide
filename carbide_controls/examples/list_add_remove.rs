use carbide_controls::{Button, List};
use carbide_controls::capture;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::environment::Environment;
use carbide_core::state::{LocalState, State, TState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let list_model = (1..10).map(|i| format!("Number {}", i)).collect::<Vec<_>>();

    let list_model_state = LocalState::new(list_model);

    fn delegate(item: TState<String>, _: TState<usize>) -> Box<dyn Widget> {
        ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(10.0)).fill(EnvironmentColor::Green),
            Text::new(item),
        ])
        .frame(0.0, 40.0)
        .expand_width()
    }

    let add_element = Button::new("Add element")
        .on_click(capture!([list_model_state], |env: &mut Environment| {
            let len = list_model_state.len();
            list_model_state.push(format!("New element: {}", len + 1));
        }))
        .frame(150.0, 22.0);

    let remove_element = Button::new("Remove element")
        .on_click(capture!([list_model_state], |env: &mut Environment| {
            list_model_state.pop();
        }))
        .frame(150.0, 22.0)
        .accent_color(EnvironmentColor::Red);

    let add_to_start = Button::new("Add element to start")
        .on_click(capture!([list_model_state], |env: &mut Environment| {
            let len = list_model_state.len();
            list_model_state.insert(0, format!("New element start: {}", len + 1));
        }))
        .frame(150.0, 22.0);

    let remove_first = Button::new("Remove first element")
        .on_click(capture!([list_model_state], |env: &mut Environment| {
            if list_model_state.len() > 0 {
                list_model_state.remove(0);
            }
        }))
        .frame(150.0, 22.0)
        .accent_color(EnvironmentColor::Red);

    application.set_scene(Window::new(
        "List Add/Remove Example - Carbide",
        Dimension::new(400.0, 400.0),
        VStack::new(vec![
            List::new(list_model_state.clone(), delegate)
                .clip()
                .border()
                .frame(350.0, 200.0),
            HStack::new(vec![add_element, remove_element]).spacing(10.0),
            HStack::new(vec![add_to_start, remove_first]).spacing(10.0),
        ])
            .spacing(10.0)
    ).close_application_on_window_close());

    application.launch();
}
