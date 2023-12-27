use std::path::PathBuf;

use oneshot::RecvError;

use carbide_core::SpawnTask;
use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::widget::*;
use carbide_macos::SavePanel;
use carbide_wgpu::{Application, Window};

fn main() {

    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Carbide MacOS save dialog example",
            Dimension::new(400.0, 600.0),
            MouseArea::new(Rectangle::new()
                .fill(EnvironmentColor::Yellow))
                .on_click(move |env: &mut Environment, _:_| {
                    SavePanel::new()
                        .set_name_field_label("Here ->")
                        .set_message("This is a message")
                        .set_shows_hidden_files(true)
                        .begin_sheet_modal_for_window(env)
                        .spawn(env, |res: Result<Option<PathBuf>, RecvError>, _| {
                            println!("Received save path: {:?}", res);
                        });
                }).frame(100.0, 100.0)
        ).close_application_on_window_close()
    );

    application.launch()
}
