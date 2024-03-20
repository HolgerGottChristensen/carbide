use std::path::PathBuf;

use oneshot::RecvError;

use carbide_core::SpawnTask;
use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::widget::*;
use carbide_macos::{FileSpecification, OpenPanel};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "MacOS Open Dialog - Carbide",
            Dimension::new(400.0, 600.0),
            MouseArea::new(Rectangle::new()
                .fill(EnvironmentColor::Yellow))
                .on_click(move |env: &mut Environment, _:_| {

                    OpenPanel::new()
                        .set_message("This is a message")
                        .set_allowed_content_types(&vec![FileSpecification::new("Gif", &["gif"])])
                        .set_prompt("I agree")
                        .set_name_field_label("Here ->")
                        .set_allows_multiple_selection(true)
                        .begin_sheet_modal_for_window(env)
                        .spawn(|res: Result<Option<Vec<PathBuf>>, RecvError>, _| {
                            println!("Received open paths: {:?}", res.unwrap());
                        });

                }).frame(100.0, 100.0)
        ).close_application_on_window_close()
    );

    application.launch()
}
