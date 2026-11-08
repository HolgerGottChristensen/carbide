use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::ControlsExt;
use carbide::closure;
use carbide::draw::Dimension;
use carbide::environment::EnvironmentFontSize;
use carbide::state::LocalState;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    let switch = LocalState::new(false);

    application.set_scene(Window::new(
        "OnChange - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Text::new(switch.clone())
                .font_size(EnvironmentFontSize::Title),
            Button::new("Change", closure!(|_| { *$switch = !*$switch; }))
                .button_style(BorderedProminentStyle)
                .frame(96.0, 22.0),
        )).spacing(10.0)
            .on_change(switch.clone(), closure!(|old, new| {
                println!("old: {:?}, new: {:?}", old, new);
            }))
    ));

    application.launch();
}