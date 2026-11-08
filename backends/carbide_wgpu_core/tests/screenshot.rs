//! Tests for `capture_texture_to_image`.
//!
//! Each test uploads a known 100x100 pattern into a texture, reads it back and compares the
//! result against a blessed PNG in `tests/snapshots/screenshot/{test}/{name}.png`, the same
//! layout the `carbide_wgpu` harness uses. Run with `BLESS=true` to create or update the
//! blessed images. On a mismatch, the actual output is written next to the blessed image as
//! `{name}.new.png`.
//!
//! A width of 100 gives 400 bytes per row, which is not a multiple of wgpu's 256-byte copy
//! alignment, so every test also exercises the row de-padding.

use carbide_wgpu_core::{capture_texture_to_image, ScreenshotError};
use wgpu::TextureFormat;

use support::{check_supported_format, context, SIZE};

#[test]
fn rgba8_unorm() {
    check_supported_format(TextureFormat::Rgba8Unorm);
}

/// sRGB textures are read back with their stored bytes, without converting them to linear.
#[test]
fn rgba8_unorm_srgb() {
    check_supported_format(TextureFormat::Rgba8UnormSrgb);
}

/// The texture stores BGRA, so this fails if the red/blue swap is missing.
#[test]
fn bgra8_unorm() {
    check_supported_format(TextureFormat::Bgra8Unorm);
}

/// The format carbide_wgpu renders to (`RENDER_TARGET_FORMAT`).
#[test]
fn bgra8_unorm_srgb() {
    check_supported_format(TextureFormat::Bgra8UnormSrgb);
}

/// An unsupported format is rejected before any GPU work. The texture is created without
/// `COPY_SRC`, so if the function tried to copy it anyway, wgpu's validation would panic
/// instead of the function returning the error.
#[test]
fn unsupported_format_is_rejected_before_readback() {
    let context = context();

    for format in [
        TextureFormat::R8Unorm,
        TextureFormat::Rg8Unorm,
        TextureFormat::Rgba8Snorm,
        TextureFormat::Rgba16Float,
        TextureFormat::Rgba32Float,
        TextureFormat::Depth32Float,
    ] {
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("screenshot_test_unsupported_texture"),
            size: wgpu::Extent3d { width: SIZE, height: SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let result = capture_texture_to_image(&context.device, &context.queue, &texture);

        match result {
            Err(ScreenshotError::UnsupportedFormat(rejected)) => assert_eq!(rejected, format),
            Ok(_) => panic!("{format:?} should be rejected, but was read back"),
        }
    }
}

// ===============================
//  Utility
// ===============================

mod support {
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;

    use carbide_wgpu_core::{capture_texture_to_image, WgpuContextNew};
    use wgpu::TextureFormat;

    pub const SIZE: u32 = 100;

    pub fn context() -> &'static WgpuContextNew {
        static CONTEXT: OnceLock<WgpuContextNew> = OnceLock::new();
        CONTEXT.get_or_init(|| futures::executor::block_on(WgpuContextNew::new()))
    }

    /// The expected RGBA pattern. Every channel varies independently, so a swapped, dropped or
    /// shifted channel shows up: red grows left to right, green grows top to bottom, blue is
    /// high in the top-left quadrant only, and alpha is half transparent in the bottom-right
    /// quadrant.
    fn expected_pattern() -> image::RgbaImage {
        image::RgbaImage::from_fn(SIZE, SIZE, |x, y| {
            let r = (x * 255 / (SIZE - 1)) as u8;
            let g = (y * 255 / (SIZE - 1)) as u8;
            let b = if x < SIZE / 2 && y < SIZE / 2 { 200 } else { 30 };
            let a = if x >= SIZE / 2 && y >= SIZE / 2 { 128 } else { 255 };
            image::Rgba([r, g, b, a])
        })
    }

    /// Creates a 100x100 texture of `format`, filled with `expected_pattern` in that format's
    /// channel order.
    fn texture_with_pattern(format: TextureFormat) -> wgpu::Texture {
        let context = context();

        let mut bytes = expected_pattern().into_raw();
        if matches!(format, TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb) {
            for pixel in bytes.as_chunks_mut::<4>().0 {
                pixel.swap(0, 2);
            }
        }

        let extent = wgpu::Extent3d { width: SIZE, height: SIZE, depth_or_array_layers: 1 };

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("screenshot_test_texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        context.queue.write_texture(
            texture.as_image_copy(),
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(SIZE * 4),
                rows_per_image: Some(SIZE),
            },
            extent,
        );

        texture
    }

    /// `tests/snapshots/screenshot/{test}`, where `{test}` is the running test's name (cargo's test
    /// harness names each test's thread after it).
    fn snapshot_dir() -> PathBuf {
        let test = std::thread::current()
            .name()
            .expect("tests run on a thread named after the test")
            .replace("::", "/");

        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/snapshots/screenshot")
            .join(test)
    }

    /// Compares `actual` against the blessed `{name}.png`, or overwrites it when `BLESS` is set.
    fn assert_snapshot(name: &str, actual: &image::RgbaImage) {
        let dir = snapshot_dir();
        let blessed_path = dir.join(format!("{name}.png"));
        let new_path = dir.join(format!("{name}.new.png"));

        if std::env::var("BLESS").is_ok() {
            std::fs::create_dir_all(&dir).unwrap();
            actual.save(&blessed_path).unwrap();
            let _ = std::fs::remove_file(&new_path);
            return;
        }

        let blessed = match image::open(&blessed_path) {
            Ok(image) => image.into_rgba8(),
            Err(_) => panic!(
                "snapshot {name:?}: no blessed image at {}. Run with BLESS=true to create it",
                blessed_path.display()
            ),
        };

        if blessed != *actual {
            let _ = actual.save(&new_path);
            panic!(
                "snapshot {name:?}: readback differs from {}; actual output written to {}",
                blessed_path.display(),
                new_path.display()
            );
        }

        let _ = std::fs::remove_file(&new_path);
    }

    /// Reads back a texture of `format` and checks it against both the input pattern and the
    /// blessed snapshot.
    pub fn check_supported_format(format: TextureFormat) {
        let context = context();
        let texture = texture_with_pattern(format);

        let image = capture_texture_to_image(&context.device, &context.queue, &texture)
            .unwrap_or_else(|error| panic!("{format:?} should be supported: {error}"));

        assert_eq!(image.dimensions(), (SIZE, SIZE));
        assert!(
            image == expected_pattern(),
            "{format:?}: readback does not match the uploaded pattern"
        );

        assert_snapshot("readback", &image);
    }
}
