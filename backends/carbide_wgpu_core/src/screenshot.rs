use std::fmt;
use wgpu::{CommandEncoderDescriptor, TextureFormat};

/// Why a texture could not be captured by [`capture_texture_to_image`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotError {
    /// The texture's format is not one of the 8-bit RGBA/BGRA formats the readback supports.
    UnsupportedFormat(TextureFormat),
}

impl fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScreenshotError::UnsupportedFormat(format) => write!(
                f,
                "cannot capture texture with format {:?}; supported formats are Rgba8Unorm, Rgba8UnormSrgb, Bgra8Unorm and Bgra8UnormSrgb",
                format
            ),
        }
    }
}

impl std::error::Error for ScreenshotError {}

/// Reads a render-target texture back from the GPU as an `image::RgbaImage`. Standard
/// offscreen-texture readback: copy into a padded `COPY_DST | MAP_READ` buffer, map it,
/// `device.poll(Wait)` to actually drive the GPU queue (the map future never resolves
/// otherwise), then de-pad rows and, for BGRA textures, swizzle to the RGBA channel order
/// `image::RgbaImage` expects.
///
/// Only 8-bit RGBA and BGRA textures (unorm or sRGB) are supported. Any other format is
/// rejected with [`ScreenshotError::UnsupportedFormat`] before any GPU work is done. sRGB
/// textures are returned with their stored (sRGB-encoded) bytes, unconverted.
pub fn capture_texture_to_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> Result<image::RgbaImage, ScreenshotError> {
    let swap_red_blue = match texture.format() {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => false,
        TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => true,
        format => return Err(ScreenshotError::UnsupportedFormat(format)),
    };

    let width = texture.width();
    let height = texture.height();

    let unpadded_bytes_per_row = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("carbide_screenshot_readback_buffer"),
        size: (padded_bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("carbide_screenshot_readback_encoder"),
    });

    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );

    queue.submit(Some(encoder.finish()));

    let slice = buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });

    device.poll(wgpu::PollType::Wait).expect("device.poll failed while reading back frame for screenshot");
    futures::executor::block_on(rx).expect("map_async callback dropped").expect("failed to map screenshot readback buffer");

    let unpadded_bytes_per_row = unpadded_bytes_per_row as usize;
    let padded_bytes_per_row = padded_bytes_per_row as usize;

    let mut pixels = vec![0u8; unpadded_bytes_per_row * height as usize];
    {
        let data = slice.get_mapped_range();
        for row in 0..height as usize {
            let src = &data[row * padded_bytes_per_row..row * padded_bytes_per_row + unpadded_bytes_per_row];
            let dst_start = row * unpadded_bytes_per_row;
            pixels[dst_start..dst_start + unpadded_bytes_per_row].copy_from_slice(src);
        }
    }
    buffer.unmap();

    if swap_red_blue {
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }
    }

    Ok(image::RgbaImage::from_raw(width, height, pixels).expect("readback buffer size should match width*height*4"))
}
