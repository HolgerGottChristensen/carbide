use carbide_core::draw::{Dimension, ImageId};
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

/// An HDR image, written as a Radiance `.hdr` file: a grey ramp from 0 on the left to 2 on the
/// right, 64 × 16. It is decoded as floats and stored as RGBA16F, its values above 1 kept; drawn
/// in 2D, the right half, above 1, is clamped to white, and the left half ramps up from black,
/// through 0.5 (sRGB 188) a quarter of the way along.
#[test]
fn an_hdr_image_is_stored_as_floats_and_drawn_clamped() {
    let path = std::env::temp_dir().join(format!("carbide_hdr_ramp_{}.hdr", std::process::id()));
    let ramp = image::Rgb32FImage::from_fn(64, 16, |x, _| {
        let value = x as f32 / 63.0 * 2.0;
        image::Rgb([value, value, value])
    });
    image::DynamicImage::ImageRgb32F(ramp).save(&path).unwrap();

    let id = ImageId::new(path.clone());
    let widget = Image::new(id.clone()).resizeable().frame(128.0, 32.0);
    let mut harness = Harness::new(widget, Dimension::new(128.0, 32.0));
    harness.render();

    let store = harness.environment().get::<carbide_wgpu_core::WgpuTextureStore>().unwrap();
    let stored = store.get(&id).expect("the image isn't stored");
    assert!(stored.float, "the HDR image is stored as 8-bit");

    let pixels = harness.pixels();
    let grey = |x: u32| pixels.get_pixel(x, 16).0[0];
    assert_eq!(grey(120), 255, "above 1 isn't white");
    // A quarter of the way along the ramp is 0.5, linear: sRGB 188.
    assert!(grey(32).abs_diff(188) <= 8, "0.5 linear is {}, not sRGB 188", grey(32));
    assert!(grey(32) > grey(16) && grey(16) > grey(4), "the left half doesn't ramp up");

    harness.snapshot("hdr_ramp");
    let _ = std::fs::remove_file(path);
}

/// An OpenEXR image is stored as floats too, with its values above 1 kept.
#[test]
fn an_exr_image_is_stored_as_floats() {
    let path = std::env::temp_dir().join(format!("carbide_exr_flat_{}.exr", std::process::id()));
    let flat = image::Rgba32FImage::from_pixel(8, 8, image::Rgba([3.0, 2.0, 1.0, 1.0]));
    image::DynamicImage::ImageRgba32F(flat).save(&path).unwrap();

    let id = ImageId::new(path.clone());
    let mut harness = Harness::new(Image::new(id.clone()), Dimension::new(16.0, 16.0));
    harness.render();

    let store = harness.environment().get::<carbide_wgpu_core::WgpuTextureStore>().unwrap();
    assert!(store.get(&id).expect("the image isn't stored").float, "the EXR image is stored as 8-bit");
    let _ = std::fs::remove_file(path);
}
