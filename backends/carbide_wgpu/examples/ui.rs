use carbide_controls::{Button, TextInput};
use carbide_core as carbide; // Required only in internal examples
use carbide_core::a;
use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_macro::ui;
use carbide_wgpu::{Application, Window};

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
                *$integer_state = (*$integer_state + 1) % 4;
            })).frame_fixed_height(45.0),
        ))
    ).close_application_on_window_close());


    application.launch();
}