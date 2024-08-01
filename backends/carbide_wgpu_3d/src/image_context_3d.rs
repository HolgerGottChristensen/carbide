use dashmap::DashMap;
use once_cell::sync::Lazy;
use carbide_3d::InnerImageContext3d;
use carbide_core::draw::{ImageId, Texture, TextureFormat};
use carbide_wgpu::{DEVICE, QUEUE};

pub(crate) fn image_context_3d_initializer() -> Box<dyn InnerImageContext3d> {
    Box::new(ImageContext3d)
}

#[derive(Debug, Clone)]
pub struct ImageContext3d;

pub(crate) static TEXTURES: Lazy<DashMap<ImageId, wgpu::Texture>> = Lazy::new(|| {
    let mut map = DashMap::new();

    let texture = Texture {
        width: 1,
        height: 1,
        bytes_per_row: 4,
        format: TextureFormat::RGBA8,
        data: &[0u8, 255u8, 0u8, 255u8],
    };

    map.insert(ImageId::default(), create_wgpu_texture(texture));
    map
});

impl InnerImageContext3d for ImageContext3d {
    fn texture_exist(&self, id: &ImageId) -> bool {
        TEXTURES.contains_key(id)
    }

    fn texture_dimensions(&self, id: &ImageId) -> Option<(u32, u32)> {
        TEXTURES.get(id).map(|a| (a.width(), a.height()))
    }

    fn update_texture(&mut self, id: ImageId, texture: Texture) -> bool {
        let texture = create_wgpu_texture(texture);
        TEXTURES.insert(id, texture);
        true
    }
}

fn create_wgpu_texture(texture: Texture) -> wgpu::Texture {
    let width = texture.width;
    let height = texture.height;

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let format = match texture.format {
        TextureFormat::RGBA8 => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::BGRA8 => wgpu::TextureFormat::Bgra8Unorm,
    };

    let wgpu_texture = DEVICE.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    //println!("len: {}", texture.data.len());
    //println!("bytes_per_row: {}", texture.bytes_per_row);
    //println!("height: {}", texture.height);
    //println!("size: {}", texture.height * texture.bytes_per_row);

    println!("{:?}", texture.data.chunks(4).next());

    QUEUE.write_texture(
        wgpu::ImageCopyTexture {
            texture: &wgpu_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: Default::default(),
        },
        &texture.data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(texture.bytes_per_row),
            rows_per_image: Some(texture.height),
        },
        size,
    );

    wgpu_texture
}