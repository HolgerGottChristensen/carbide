//! The environment maps `EnvironmentMaps` makes on the GPU, read back: the reflection cube, the
//! spherical harmonics and the DFG table.

use std::sync::OnceLock;

use carbide_core::draw::{ImageFormat, ImageId, Texture, TextureFormat};
use carbide_wgpu_3d::{EnvironmentMaps, DFG_SIZE};
use carbide_wgpu_core::{WgpuContextNew, WgpuTextureStore};
use half::f16;

fn context() -> &'static WgpuContextNew {
    static CONTEXT: OnceLock<WgpuContextNew> = OnceLock::new();
    CONTEXT.get_or_init(|| futures::executor::block_on(WgpuContextNew::new()))
}

/// Read back `layer` of mip `level` of `texture` as RGBA floats, from RGBA16F or RGBA32F.
fn read(texture: &wgpu::Texture, level: u32, layer: u32) -> Vec<[f32; 4]> {
    let context = context();
    let width = (texture.width() >> level).max(1);
    let height = (texture.height() >> level).max(1);
    let wide = texture.format() == wgpu::TextureFormat::Rgba32Float;
    let texel = if wide { 16 } else { 8 };
    let padded = (width * texel).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

    let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = context.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture, mip_level: level, origin: wgpu::Origin3d { x: 0, y: 0, z: layer }, aspect: wgpu::TextureAspect::All },
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
    let mut texels = Vec::new();
    for row in 0..height {
        let start = (row * padded) as usize;
        for bytes in data[start..start + (width * texel) as usize].chunks(texel as usize) {
            let mut value = [0.0; 4];
            for (channel, bytes) in bytes.chunks(texel as usize / 4).enumerate() {
                value[channel] = if wide {
                    f32::from_le_bytes(bytes.try_into().unwrap())
                } else {
                    f16::from_le_bytes(bytes.try_into().unwrap()).to_f32()
                };
            }
            texels.push(value);
        }
    }
    texels
}

/// Store a `width` × `height` float panorama, its colour at (u, v) given by `color`, from 0 to 1
/// across and down.
fn store_panorama(store: &mut WgpuTextureStore, id: &ImageId, width: u32, height: u32, color: impl Fn(f32, f32) -> [f32; 3]) {
    let mut data = Vec::with_capacity((width * height * 8) as usize);
    for y in 0..height {
        for x in 0..width {
            let [r, g, b] = color((x as f32 + 0.5) / width as f32, (y as f32 + 0.5) / height as f32);
            for value in [r, g, b, 1.0] {
                data.extend_from_slice(&f16::from_f32(value).to_le_bytes());
            }
        }
    }
    let texture = Texture { width, height, bytes_per_row: width * 8, format: TextureFormat::RGBA16F, data: &data };
    store.insert(id.clone(), texture, &context().device, &context().queue);
}

/// The light the harmonics give a white Lambertian surface facing `n`, as the shading will
/// evaluate them.
fn evaluate(harmonics: &[[f32; 4]], n: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = n;
    let basis = [
        0.282095,
        0.488603 * y,
        0.488603 * z,
        0.488603 * x,
        1.092548 * x * y,
        1.092548 * y * z,
        0.315392 * (3.0 * z * z - 1.0),
        1.092548 * x * z,
        0.546274 * (x * x - y * y),
    ];
    let mut sum = [0.0; 3];
    for (coefficient, basis) in harmonics.iter().zip(basis) {
        for channel in 0..3 {
            sum[channel] += coefficient[channel] * basis;
        }
    }
    sum
}

fn close(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

/// A panorama of one colour lights every direction alike: every level of the reflection cube is
/// that colour, and the harmonics give it for every normal, with nothing in their higher bands.
#[test]
fn a_constant_environment_gives_constant_light() {
    let color = [0.5, 1.0, 2.0];
    let mut store = WgpuTextureStore::new();
    let id = ImageId::temp(ImageFormat::Png);
    store_panorama(&mut store, &id, 16, 8, |_, _| color);

    let context = context();
    let mut maps = EnvironmentMaps::new();
    let map = maps.prepare(&id, Some(&store), &context.device, &context.queue).unwrap();
    assert_eq!((map.radiance.width(), map.levels), (32, 3));

    for level in 0..map.levels {
        for face in 0..6 {
            for texel in read(&map.radiance, level, face) {
                for channel in 0..3 {
                    assert!(close(texel[channel], color[channel], color[channel] * 0.02), "level {level}, face {face}: {texel:?}");
                }
            }
        }
    }

    let harmonics = read(&map.harmonics, 0, 0);
    for (index, coefficient) in harmonics.iter().enumerate().skip(1) {
        assert!(coefficient[..3].iter().all(|value| value.abs() < 0.01), "coefficient {index} is {coefficient:?}");
    }
    for n in [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.6, 0.8]] {
        let light = evaluate(&harmonics, n);
        for channel in 0..3 {
            assert!(close(light[channel], color[channel], color[channel] * 0.01), "{n:?} gives {light:?}");
        }
    }
}

/// The faces' directions, from where things are in the panorama: its top half is the sky, +Y;
/// its middle column +X; a quarter to the right of it +Z.
#[test]
fn the_panorama_is_mapped_to_the_cube_faces() {
    let context = context();
    let mut store = WgpuTextureStore::new();
    let mut maps = EnvironmentMaps::new();
    // The middle of a face's first level: 32 × 32.
    let centre = |texels: &[[f32; 4]]| texels[16 * 32 + 16][0];

    // A white sky over a black ground.
    let sky = ImageId::temp(ImageFormat::Png);
    store_panorama(&mut store, &sky, 128, 64, |_, v| if v < 0.5 { [1.0; 3] } else { [0.0; 3] });
    let map = maps.prepare(&sky, Some(&store), &context.device, &context.queue).unwrap();
    assert!(close(centre(&read(&map.radiance, 0, 2)), 1.0, 0.01), "+Y isn't the sky");
    assert!(close(centre(&read(&map.radiance, 0, 3)), 0.0, 0.01), "-Y isn't the ground");

    // Facing up, a surface sees the whole sky: about 1. Facing down, none of it.
    let harmonics = read(&map.harmonics, 0, 0);
    let up = evaluate(&harmonics, [0.0, 1.0, 0.0])[0];
    let down = evaluate(&harmonics, [0.0, -1.0, 0.0])[0];
    let side = evaluate(&harmonics, [1.0, 0.0, 0.0])[0];
    assert!(up > 0.85 && down < 0.15 && close(side, 0.5, 0.05), "up {up}, down {down}, side {side}");

    // A bright band down the middle, and one a quarter to the right of it.
    for (u, face, opposite) in [(0.5, 0, 1), (0.75, 4, 5)] {
        let band = ImageId::temp(ImageFormat::Png);
        store_panorama(&mut store, &band, 128, 64, |x, _| if (x - u).abs() < 0.1 { [1.0; 3] } else { [0.0; 3] });
        let map = maps.prepare(&band, Some(&store), &context.device, &context.queue).unwrap();
        assert!(close(centre(&read(&map.radiance, 0, face)), 1.0, 0.01), "the band at {u} isn't on face {face}");
        assert!(close(centre(&read(&map.radiance, 0, opposite)), 0.0, 0.01), "the band at {u} is on face {opposite}");
    }
}

/// A rougher level is blurrier: a small bright spot spreads into its surroundings as the levels go
/// down, while the total light stays about the same.
#[test]
fn rougher_levels_are_blurrier() {
    let context = context();
    let mut store = WgpuTextureStore::new();
    let id = ImageId::temp(ImageFormat::Png);
    // A spot 10° across around +X, on a dim background.
    store_panorama(&mut store, &id, 256, 128, |u, v| {
        let near = (u - 0.5).abs() < 10.0 / 360.0 && (v - 0.5).abs() < 10.0 / 180.0;
        if near { [20.0; 3] } else { [0.1; 3] }
    });
    let mut maps = EnvironmentMaps::new();
    let map = maps.prepare(&id, Some(&store), &context.device, &context.queue).unwrap();
    assert_eq!(map.levels, 4);

    // The brightest texel of +X, and the texel at its edge, level by level.
    let mut peaks = vec![];
    for level in 0..map.levels {
        let size = 64 >> level;
        let texels = read(&map.radiance, level, 0);
        let peak = texels.iter().map(|texel| texel[0]).fold(0.0, f32::max);
        let edge = texels[(size / 2) * size + size / 8][0];
        peaks.push((peak, edge));
    }
    for pair in peaks.windows(2) {
        assert!(pair[1].0 < pair[0].0, "a rougher level isn't softer: {peaks:?}");
    }
    assert!(peaks[0].1 < 0.2 && peaks[3].1 > peaks[0].1 * 2.0, "the spot doesn't spread: {peaks:?}");
}

/// The scale and bias of the split-sum DFG term for `nov` and `perceptual` roughness, integrated
/// on the CPU over a grid of light directions, without importance sampling: the GGX distribution,
/// the height-correlated Smith visibility and Schlick's Fresnel, as the shading uses.
fn integrate_dfg(nov: f32, perceptual: f32) -> [f32; 2] {
    let alpha = (perceptual * perceptual) as f64;
    let nov = nov as f64;
    let v = [(1.0 - nov * nov).sqrt(), 0.0, nov];
    let (thetas, phis) = (600, 600);
    let (d_theta, d_phi) = (std::f64::consts::FRAC_PI_2 / thetas as f64, std::f64::consts::TAU / phis as f64);

    let (mut scale, mut bias) = (0.0, 0.0);
    for i in 0..thetas {
        let theta = (i as f64 + 0.5) * d_theta;
        for j in 0..phis {
            let phi = (j as f64 + 0.5) * d_phi;
            let l = [theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos()];
            let nol = l[2];
            let h = [v[0] + l[0], v[1] + l[1], v[2] + l[2]];
            let length = (h[0] * h[0] + h[1] * h[1] + h[2] * h[2]).sqrt();
            let (noh, voh) = (h[2] / length, (v[0] * h[0] + v[1] * h[1] + v[2] * h[2]) / length);

            let a2 = alpha * alpha;
            let f = noh * noh * (a2 - 1.0) + 1.0;
            let d = a2 / (std::f64::consts::PI * f * f);
            let visibility = 0.5 / (nov * (nol * nol * (1.0 - a2) + a2).sqrt() + nol * (nov * nov * (1.0 - a2) + a2).sqrt());
            let fc = (1.0 - voh).powi(5);

            // The BRDF without F0, times the cosine and the patch's solid angle.
            let weight = d * visibility * nol * theta.sin() * d_theta * d_phi;
            scale += (1.0 - fc) * weight;
            bias += fc * weight;
        }
    }
    [scale as f32, bias as f32]
}

/// The DFG table against the same integral worked out on the CPU without importance sampling, at
/// texels across the table: they agree to 0.01. And the known ends: a smooth surface seen head-on
/// reflects F0 as it is, scale 1 and bias 0; the bias grows towards grazing angles; and no texel
/// reflects more light than comes in.
#[test]
fn the_dfg_table_matches_known_values() {
    let context = context();
    let mut maps = EnvironmentMaps::new();
    maps.dfg(&context.device, &context.queue);
    let table = read(maps.dfg_texture().unwrap(), 0, 0);
    assert_eq!(table.len(), (DFG_SIZE * DFG_SIZE) as usize);
    // A texel and its N·V and roughness, at its middle.
    let centre = |index: u32| (index as f32 + 0.5) / DFG_SIZE as f32;
    let at = |x: u32, y: u32| table[(y * DFG_SIZE + x) as usize];

    let head_on = at(DFG_SIZE - 1, 0);
    assert!(close(head_on[0], 1.0, 0.01) && close(head_on[1], 0.0, 0.01), "head-on and smooth: {head_on:?}");
    assert!(at(25, 25)[1] > at(115, 25)[1], "the bias doesn't grow towards grazing angles");
    assert!(table.iter().all(|texel| texel[0] + texel[1] <= 1.01), "a texel reflects more light than comes in");

    for x in [12, 38, 64, 90, 127] {
        for y in [38, 64, 90, 115] {
            let [scale, bias, ..] = at(x, y);
            let [expected_scale, expected_bias] = integrate_dfg(centre(x), centre(y));
            assert!(
                close(scale, expected_scale, 0.01) && close(bias, expected_bias, 0.01),
                "at N·V {:.2}, roughness {:.2}: {scale:.4}, {bias:.4} against {expected_scale:.4}, {expected_bias:.4}",
                centre(x), centre(y),
            );
        }
    }
}

/// The maps are made the first time an image is prepared, not again on the next; and again when
/// the image under its id is replaced. An image not in the store has none.
#[test]
fn maps_are_made_once_per_image() {
    let context = context();
    let mut store = WgpuTextureStore::new();
    let mut maps = EnvironmentMaps::new();
    let id = ImageId::temp(ImageFormat::Png);

    assert!(maps.prepare(&id, Some(&store), &context.device, &context.queue).is_none());
    assert!(maps.prepare(&id, None, &context.device, &context.queue).is_none());

    store_panorama(&mut store, &id, 16, 8, |_, _| [1.0; 3]);
    maps.prepare(&id, Some(&store), &context.device, &context.queue).unwrap();
    maps.prepare(&id, Some(&store), &context.device, &context.queue).unwrap();
    assert_eq!(maps.builds(), 1);

    store_panorama(&mut store, &id, 16, 8, |_, _| [2.0; 3]);
    let map = maps.prepare(&id, Some(&store), &context.device, &context.queue).unwrap();
    assert!(close(read(&map.radiance, 0, 0)[0][0], 2.0, 0.02), "the replaced image's maps weren't made again");
    assert_eq!(maps.builds(), 2);
}
