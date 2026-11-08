use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::{ControlsExt, TextInput};
use carbide::closure;
use carbide::draw::Dimension;
use carbide::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide::state::LocalState;
use carbide::widget::*;
use carbide::ui;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let integer_state = LocalState::new(0);

    let widget = ui!(
        match integer_state {
            0 => Rectangle::new().fill(EnvironmentColor::Blue),
            1 => ZStack::new((
                Rectangle::new().fill(EnvironmentColor::Yellow),
                TextInput::new(LocalState::new("Hello world!".to_string()))
                    .padding(30.0),
            )),
            n => ZStack::new((
                Rectangle::new().fill(EnvironmentColor::Red),
                Text::new(n).font_size(EnvironmentFontSize::LargeTitle),
            )),
        }
    );

    application.set_scene(Window::new(
        "Match - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            widget,
            Button::new("Click to change the view above", closure!(|_| {
                *$integer_state = (*$integer_state + 1) % 4;
            })).frame_fixed_height(45.0)
                .button_style(BorderedProminentStyle),
        ))
    ));


    application.launch();
}