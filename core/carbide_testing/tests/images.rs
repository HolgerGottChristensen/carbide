use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::widget::{Image, Rectangle, WidgetExt, ZStack};
use carbide_testing::Harness;

/// A raster image with transparent parts, loaded from the assets folder, drawn at its own size
/// over a coloured background. Covers loading, premultiplying and drawing images in 2D.
#[test]
fn local_raster_image() {
    let widget = ZStack::new((
        Rectangle::new().fill(EnvironmentColor::Yellow),
        Image::new("images/rust.png"),
    ));

    let mut harness = Harness::new(widget, Dimension::new(160.0, 160.0));

    harness.snapshot("rust");
}

/// The same image drawn twice, at different sizes: both draws use the one stored texture.
#[test]
fn one_image_drawn_twice() {
    let widget = ZStack::new((
        Rectangle::new().fill(EnvironmentColor::Yellow),
        Image::new("images/rust.png").resizeable().frame(40.0, 40.0).offset(-40.0, -40.0),
        Image::new("images/rust.png").resizeable().frame(80.0, 80.0).offset(30.0, 30.0),
    ));

    let mut harness = Harness::new(widget, Dimension::new(160.0, 160.0));

    harness.snapshot("twice");
}
