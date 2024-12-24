use carbide_controls::{ControlsExt, List};
use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_core::closure;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, ReadState, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let list_model = (1..10).map(|i| format!("Number {}", i)).collect::<Vec<_>>();

    let list_model_state = LocalState::new(list_model);

    fn delegate(item: impl State<T=String>, _: impl ReadState<T=usize>) -> impl Widget {
        ZStack::new((
            RoundedRectangle::new(CornerRadii::all(10.0)).fill(EnvironmentColor::Green),
            Text::new(item),
        ))
        .frame(0.0, 40.0)
        .expand_width()
    }

    let add_element = Button::new("Add element", closure!(|_| {
            let len = ($list_model_state).len();
            list_model_state.push(format!("New element: {}", len + 1));
        }))
        .frame(150.0, 22.0);

    let remove_element = Button::new("Remove element", closure!(|_| {
            ($list_model_state).pop();
        }))
        .frame(150.0, 22.0)
        .accent_color(EnvironmentColor::Red);

    let add_to_start = Button::new("Add element to start", closure!(|_| {
            let len = ($list_model_state).len();
            list_model_state.insert(0, format!("New element start: {}", len + 1));
        }))
        .frame(150.0, 22.0);

    let remove_first = Button::new("Remove first element", closure!(|_| {
            if ($list_model_state).len() > 0 {
                list_model_state.remove(0);
            }
        }))
        .frame(150.0, 22.0)
        .accent_color(EnvironmentColor::Red);

    application.set_scene(Window::new(
        "List Add/Remove Example - Carbide",
        Dimension::new(400.0, 400.0),
        VStack::new((
            List::new(list_model_state.clone(), delegate)
                .clip()
                .padding(1.0)
                .background(Rectangle::new().stroke(EnvironmentColor::Teal).stroke_style(1.0))
                .frame(350.0, 200.0),
            HStack::new((add_element, remove_element)).spacing(10.0),
            HStack::new((add_to_start, remove_first)).spacing(10.0),
        ))
            .spacing(10.0)
            .button_style(BorderedProminentStyle)
    ));

    application.launch();
}
