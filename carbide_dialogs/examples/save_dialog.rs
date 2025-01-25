use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_controls::ControlsExt;
use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_dialogs::save_dialog::SaveDialog;
use carbide_dialogs::{DialogsExt, FileType, NativeStyle};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Save Dialog example - Carbide",
            Dimension::new(400.0, 600.0),
            Button::new("Save dialog", |ctx| {

                SaveDialog::new()
                    .set_prompt(Some("Pick this".to_string()))
                    .set_message(Some("This is a message".to_string()))
                    .set_title(Some("This is a title".to_string()))
                    .set_default_file_name(Some("NewFile".to_string()))
                    .set_file_types(vec![FileType::new("Rust source", vec!["rs"])])
                    .open(ctx.env, |res, _| {
                        println!("Received save path: {:?}", res.unwrap());
                    });

            }).frame(120.0, 22.0)
                .button_style(BorderedProminentStyle)
                .save_dialog_style(NativeStyle)
        )
    );

    application.launch()
}
