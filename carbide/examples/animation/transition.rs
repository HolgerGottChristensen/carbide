use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::ControlsExt;
use carbide::closure;
use carbide::draw::Dimension;
use carbide::state::{LocalState, ReadStateExtTransition};
use carbide::widget::*;
use carbide::{Application, Window};
use carbide::time::*;

fn main() {
    let mut application = Application::new();

    let offset = LocalState::new(-120.0);

    let transition = offset.transition().duration(Duration::new(2, 0));

    application.set_scene(Window::new(
        "Transition - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Rectangle::new()
                .frame(60.0, 60.0)
                .offset(transition, 0.0),
            HStack::new((
                Button::new("Left", closure!(|_| {
                    *$offset = -120.0;
                }))
                    .frame(96.0, 22.0),
                Button::new("Right", closure!(|_| {
                    *$offset = 120.0;
                }))
                    .frame(96.0, 22.0),
            )).spacing(10.0),
        )).spacing(10.0)
            .button_style(BorderedProminentStyle)
    ));


    application.launch();
}