use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_media::{Video, VideoPlayer};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(
        Window::new(
        "Video example - Carbide",
        Dimension::new(1000.0, 700.0),
        VideoPlayer::new("/Users/holgerchristensen/Repositories/SwiftVideoTutorial/PlayingVideo/PlayingVideo/music.mp4")
        //VideoPlayer::new("https://d2nvs31859zcd8.cloudfront.net/062c31ad3f7ecda5e14d_tangotek_42836465403_1696014555/chunked/index-dvr.m3u8")
        //Video::new("https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_5MB.mp4")
            //.clip_shape(*Circle::new()),
            .border()
    ).close_application_on_window_close());

    application.launch();
}
