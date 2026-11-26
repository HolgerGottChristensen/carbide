//! Tests for the mip levels `WgpuTextureStore` generates.

use std::sync::OnceLock;
use carbide_core::draw::{ImageId, Texture, TextureFormat};
use carbide_wgpu_core::{WgpuContextNew, WgpuTextureStore};

fn context() -> &'static WgpuContextNew {
    static CONTEXT: OnceLock<WgpuContextNew> = OnceLock::new();
    CONTEXT.get_or_init(|| futures::executor::block_on(WgpuContextNew::new()))
}

/// Read back mip level `level` of `texture`, as its bytes, without padding: 4 a texel for RGBA8,
/// 8 for RGBA16F.
fn read_level(texture: &wgpu::Texture, level: u32) -> Vec<u8> {
    let context = context();
    let width = (texture.width() >> level).max(1);
    let height = (texture.height() >> level).max(1);
    let texel = if texture.format() == wgpu::TextureFormat::Rgba16Float { 8 } else { 4 };
    let padded = (width * texel).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

    let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = context.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture, mip_level: level, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: None },
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    context.queue.submit(Some(encoder.finish()));

    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    context.device.poll(wgpu::PollType::Wait).unwrap();

    let data = slice.get_mapped_range();
    let mut bytes = Vec::new();
    for row in 0..height {
        let start = (row * padded) as usize;
        bytes.extend_from_slice(&data[start..start + (width * texel) as usize]);
    }
    bytes
}

fn store_rgba(store: &mut WgpuTextureStore, id: &ImageId, width: u32, height: u32, data: &[u8]) {
    let context = context();
    store.insert(id.clone(), Texture { width, height, bytes_per_row: width * 4, format: TextureFormat::RGBA8, data }, &context.device, &context.queue);
}

/// A 2×2 black and white checker averages to linear grey, 0.5, which is 188 in sRGB. Averaging
/// the stored sRGB bytes instead would give 128, too dark.
#[test]
fn levels_are_averaged_in_linear_space() {
    let mut store = WgpuTextureStore::new();
    let id = ImageId::temp(carbide_core::draw::ImageFormat::Png);
    let checker = [
        0, 0, 0, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 0, 0, 0, 255,
    ];

    store_rgba(&mut store, &id, 2, 2, &checker);

    let stored = store.get(&id).unwrap();
    assert_eq!(stored.mip_level_count, 2);

    let level1 = read_level(&stored.texture, 1);
    for channel in &level1[..3] {
        assert!((186..=190).contains(channel), "grey was {:?}", &level1);
    }
    assert_eq!(level1[3], 255);
}

/// A 64×32 image gets levels down to 1×1, and a single colour stays that colour in every level.
#[test]
fn a_full_chain_of_levels_keeps_a_flat_colour() {
    let mut store = WgpuTextureStore::new();
    let id = ImageId::temp(carbide_core::draw::ImageFormat::Png);
    let colour = [200u8, 100, 50, 255];
    let data: Vec<u8> = colour.iter().copied().cycle().take(64 * 32 * 4).collect();

    store_rgba(&mut store, &id, 64, 32, &data);

    let stored = store.get(&id).unwrap();
    assert_eq!(stored.mip_level_count, 7);

    for level in 0..stored.mip_level_count {
        let pixels = read_level(&stored.texture, level);
        for pixel in pixels.chunks_exact(4) {
            for (got, want) in pixel.iter().zip(colour) {
                assert!(got.abs_diff(want) <= 1, "level {level}: {pixel:?}");
            }
        }
    }
}

/// A float (HDR) texture keeps values above 1, and its levels are averaged as they are, linear and
/// unclamped: a 2×2 checker of 4 and 0 averages to 2. Both of its views read the floats as stored.
#[test]
fn float_textures_keep_values_above_one() {
    let mut store = WgpuTextureStore::new();
    let id = ImageId::temp(carbide_core::draw::ImageFormat::Hdr);
    let texels: [[f32; 4]; 4] = [[4.0, 4.0, 4.0, 1.0], [0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0], [4.0, 4.0, 4.0, 1.0]];
    let data: Vec<u8> = texels.iter().flatten().flat_map(|value| half::f16::from_f32(*value).to_le_bytes()).collect();

    let context = context();
    store.insert(id.clone(), Texture { width: 2, height: 2, bytes_per_row: 16, format: TextureFormat::RGBA16F, data: &data }, &context.device, &context.queue);

    let stored = store.get(&id).unwrap();
    assert!(stored.float);
    assert_eq!(stored.texture.format(), wgpu::TextureFormat::Rgba16Float);
    assert_eq!(stored.mip_level_count, 2);

    let float = |bytes: &[u8]| bytes.chunks_exact(2).map(|pair| half::f16::from_le_bytes([pair[0], pair[1]]).to_f32()).collect::<Vec<f32>>();
    let level0 = float(&read_level(&stored.texture, 0));
    assert_eq!(&level0[..4], &[4.0, 4.0, 4.0, 1.0]);

    let level1 = float(&read_level(&stored.texture, 1));
    for value in &level1[..3] {
        assert!((value - 2.0).abs() < 0.01, "the level averages to {level1:?}");
    }
    assert!((level1[3] - 1.0).abs() < 0.01);
}
