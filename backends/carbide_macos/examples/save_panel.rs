use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::{LocalState, Map1, State, StateExt};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::{task, Color, SpawnTask};
use carbide_core::draw::Dimension;
use carbide_core::environment::Environment;
use carbide_macos::SavePanel;
use carbide_wgpu::{Application, Window};
use futures::future::Map;
use futures::FutureExt;
use oneshot::RecvError;

fn main() {

    let mut application = Application::new();

    fn window(child: Box<dyn Widget>) -> Box<Window> {
        Window::new(
            "Carbide MacOS save dialog example",
            Dimension::new(400.0, 600.0),
            child
        ).close_application_on_window_close()
    }

    let widgets = MouseArea::new(Rectangle::new()
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
        }).frame(100.0, 100.0);

    application.set_scene(
        window(widgets)
    );

    application.launch()
}
