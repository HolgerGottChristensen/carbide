use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_controls::{ControlsExt, List};
use carbide_core::closure;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let model = LocalState::new(
        (1..20).map(|i| format!("Number {}", i)).collect::<Vec<_>>()
    );

    let add_element = Button::new("Add element", closure!(|_| {
            let len = ($model).len();
            model.push(format!("New element: {}", len + 1));
        }))
        .frame(150.0, 22.0);

    let remove_element = Button::new("Remove element", closure!(|_| {
            ($model).pop();
        }))
        .frame(150.0, 22.0)
        .accent_color(EnvironmentColor::Red);

    let add_to_start = Button::new("Add element to start", closure!(|_| {
            let len = ($model).len();
            model.insert(0, format!("New element start: {}", len + 1));
        }))
        .frame(150.0, 22.0);

    let remove_first = Button::new("Remove first element", closure!(|_| {
            if ($model).len() > 0 {
                model.remove(0);
            }
        }))
        .frame(150.0, 22.0)
        .accent_color(EnvironmentColor::Red);

    application.set_scene(Window::new(
        "List Add/Remove Example - Carbide",
        Dimension::new(400.0, 400.0),
        VStack::new((
            List::new(model.clone(), |item, _| {
                ZStack::new((
                    RoundedRectangle::new(CornerRadii::all(3.0)).fill(EnvironmentColor::SystemFill),
                    Text::new(item),
                )).frame_fixed_height(30.0)
            })
                .padding(1.0)
                .background(Rectangle::new().stroke(EnvironmentColor::Teal).stroke_style(1.0))
                .custom_flexibility(0),
            HStack::new((add_element, remove_element)).spacing(10.0),
            HStack::new((add_to_start, remove_first)).spacing(10.0),
        ))
            .spacing(10.0)
            .padding(30.0)
            .button_style(BorderedProminentStyle)
    ));

    application.launch();
}
