use wgpu::CommandEncoderDescriptor;

/// Reads a render-target texture back from the GPU as an `image::RgbaImage`. Standard
/// offscreen-texture readback: copy into a padded `COPY_DST | MAP_READ` buffer, map it,
/// `device.poll(Wait)` to actually drive the GPU queue (the map future never resolves
/// otherwise), then de-pad rows and swizzle BGRA->RGBA (`RENDER_TARGET_FORMAT` is
/// `Bgra8UnormSrgb`, but `image::RgbaImage` expects RGBA channel order).
pub(crate) fn capture_texture_to_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> image::RgbaImage {
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

    // RENDER_TARGET_FORMAT is Bgra8UnormSrgb; image::RgbaImage expects RGBA channel order.
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    image::RgbaImage::from_raw(width, height, pixels).expect("readback buffer size should match width*height*4")
}
