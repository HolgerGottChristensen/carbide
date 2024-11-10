use carbide_core::draw::Dimension;
use carbide_media::Video;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(
        Window::new(
        "Video example - Carbide",
        Dimension::new(1000.0, 700.0),
        Video::new("/Users/holgerchristensen/Repositories/SwiftVideoTutorial/PlayingVideo/PlayingVideo/music.mp4").scaled_to_fit()

        // streamlink "https://www.twitch.tv/videos/2012818262" best --json
        //Video::new("https://d2nvs31859zcd8.cloudfront.net/6e350bf6efa46d049e84_tangotek_43279002651_1703437657/chunked/index-dvr.m3u8")
        //Video::new("https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_5MB.mp4")
            //.clip_shape(*Circle::new()),
    ));

    application.launch();
}
