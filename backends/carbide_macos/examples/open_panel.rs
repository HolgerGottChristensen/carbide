use std::path::PathBuf;

use oneshot::RecvError;

use carbide_core::SpawnTask;
use carbide_core::dialog::FileSpecification;
use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::widget::*;
use carbide_macos::OpenPanel;
use carbide_wgpu::{Application, Window};

fn main() {

    let mut application = Application::new();

    fn window(child: Box<dyn AnyWidget>) -> Box<Window> {
        Window::new(
            "Carbide MacOS open dialog example",
            Dimension::new(400.0, 600.0),
            child
        ).close_application_on_window_close()
    }

    let widgets = MouseArea::new(Rectangle::new()
        .fill(EnvironmentColor::Yellow))
        .on_click(move |env: &mut Environment, _:_| {

            OpenPanel::new()
                .set_message("This is a message")
                .set_allowed_content_types(&vec![FileSpecification::new("Gif", &["gif"])])
                .set_prompt("I agree")
                .set_name_field_label("Here ->")
                .set_allows_multiple_selection(true)
                .begin_sheet_modal_for_window(env)
                .spawn(env, |res: Result<Option<Vec<PathBuf>>, RecvError>, _| {
                    println!("Received open paths: {:?}", res.unwrap());
                });

        }).frame(100.0, 100.0);

    application.set_scene(
        window(widgets)
    );

    application.launch()
}
