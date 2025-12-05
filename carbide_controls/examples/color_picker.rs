use carbide_controls::color_picker::{ColorPicker, PlainStyle};
use carbide_controls::ControlsExt;
use carbide_core::draw::{Color, Dimension};
use carbide_core::state::{LocalState, ReadStateExtNew};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let color_state = LocalState::new(Color::new_rgb(255, 0, 0));
    let color_state2 = LocalState::new(Color::new_rgb(255, 0, 0));
    let color_state3 = LocalState::new(Color::new_rgb(255, 0, 0));

    application.set_scene(Window::new(
        "ColorPicker Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Text::new(color_state.map(|color: &Color| format!("{:?}", color)))
                .font_size(16u32),

            ColorPicker::new("Pick color", color_state),
            ColorPicker::new("Pick color 2", color_state2),
            ColorPicker::new("Disabled", color_state3)
                .enabled(false),
        ))
            .spacing(20.0)
            .color_picker_style(PlainStyle)
    ));

    application.launch();
}
