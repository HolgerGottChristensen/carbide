use carbide_controls::{ControlsExt, TextInput};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let text_state = LocalState::new("Hello world!".to_string());
    let text_state2 = LocalState::new("Hej verden!".to_string());
    let text_state3 = LocalState::new("Hallo Welt!".to_string());

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(
        Window::new(
        "Labeled example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            TextInput::new(text_state.clone())
                .frame_fixed_width(150.0)
                .fit_height()
                .label("Label:"),
            TextInput::new(text_state2.clone())
                .frame_fixed_width(150.0)
                .fit_height()
                .label("Longer label:"),
            TextInput::new(text_state3.clone())
                .frame_fixed_width(150.0)
                .fit_height()
                .label("Disabled label:")
                .enabled(false)
                .accent_color(EnvironmentColor::Orange),
        ))
            .spacing(10.0)
            .cross_axis_alignment(CrossAxisAlignment::End)
            .padding(EdgeInsets::all(40.0)),
        ));

    application.launch();
}
