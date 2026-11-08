use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::ControlsExt;
use carbide::closure;
use carbide::draw::Dimension;
use carbide::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide::state::LocalState;
use carbide::widget::*;
use carbide::ui;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    let condition = LocalState::new(false);

    let widget = ui!(if condition {
        ZStack::new((
            Rectangle::new().fill(EnvironmentColor::Green),
            Text::new("True").font_size(EnvironmentFontSize::LargeTitle)
        ))
    } else {
        ZStack::new((
            Rectangle::new().fill(EnvironmentColor::Red),
            Text::new("False").font_size(EnvironmentFontSize::LargeTitle)
        ))
    });

    application.set_scene(Window::new(
        "If - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            widget,
            Button::new(
                "Click to change the view above",
                closure!(|_| {
                    *$condition = !*$condition;
                }),
            )
            .frame_fixed_height(45.0)
            .button_style(BorderedProminentStyle),
        )),
    ));

    application.launch();
}
