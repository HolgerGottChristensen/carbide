use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_media::Video;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(
        Window::new(
        "Help example - Carbide",
        Dimension::new(600.0, 400.0),
        *Video::new("/Users/holgerchristensen/Repositories/SwiftVideoTutorial/PlayingVideo/PlayingVideo/music.mp4").frame(100.0, 100.0),
    ).close_application_on_window_close());

    application.launch();
}
