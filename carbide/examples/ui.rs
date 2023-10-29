use carbide::controls::{Button, TextInput};
use carbide::draw::Dimension;
use carbide::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide::{a, ui};
use carbide::state::LocalState;
use carbide::widget::*;
use carbide::{Application, Window};


fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let integer_state = LocalState::new(0);

    let widget = ui!(
        match integer_state {
            0 => Rectangle::new().fill(EnvironmentColor::Blue).boxed(),
            1 => ZStack::new((
                Rectangle::new().fill(EnvironmentColor::Yellow),
                TextInput::new(LocalState::new("Hello world!".to_string()))
                    .padding(30.0),
            )).boxed(),
            n => ZStack::new((
                Rectangle::new().fill(EnvironmentColor::Red),
                Text::new(n).font_size(EnvironmentFontSize::LargeTitle),
            )).boxed(),
        }
    );

    application.set_scene(Window::new(
        "UI macro example",
        Dimension::new(400.0, 600.0),
        VStack::new((
            widget,
            Button::new_primary("Click to change the view above", a!(|_, _| {
                *$integer_state = (*$integer_state + 1) % 3;
            })).frame_fixed_height(45.0),
        ))
    ).close_application_on_window_close());


    application.launch();
}