use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::ControlsExt;
use carbide::draw::Dimension;
use carbide::widget::*;
use carbide::dialogs::{DialogsExt, FileType, NativeStyle};
use carbide::dialogs::open_dialog::OpenDialog;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Open Dialog example - Carbide",
            Dimension::new(400.0, 600.0),
            Button::new("Open dialog", |ctx| {

                OpenDialog::new()
                    .set_prompt(Some("Pick this".to_string()))
                    .set_multiple_selection(true)
                    .set_message(Some("This is a message".to_string()))
                    .set_title(Some("This is a title".to_string()))
                    .set_file_types(vec![FileType::new("Rust source", vec!["rs"])])
                    .open(ctx.env, |res, _| {
                        println!("Received open paths: {:?}", res.unwrap());
                    });

            }).frame(120.0, 22.0)
                .button_style(BorderedProminentStyle)
                .open_dialog_style(NativeStyle)
        )
    );

    application.launch()
}
