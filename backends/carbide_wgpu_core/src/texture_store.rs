use std::any::TypeId;
use std::collections::HashMap;
use std::pin::Pin;
use carbide_core::draw::{ImageId, Texture, TextureFormat};
use carbide_core::environment::{Environment, EnvironmentKey, Exclusive};
use carbide_core::plugin::Plugin;
use carbide_core::register;
use wgpu::{Device, Queue, TextureView};
use crate::mipmap::{mip_level_count, MipmapGenerator};

/// A raster image on the GPU, shared by the 2D and 3D renderers.
///
/// The store holds textures and views, not bind groups: each renderer builds its own bind groups
/// from the views, with its own layouts and samplers.
#[derive(Debug)]
pub struct StoredTexture {
    pub texture: wgpu::Texture,
    /// Reads the texels as sRGB, decoding them to linear: for colours, such as images in 2D and
    /// albedo and emission in 3D. For a float texture, which is linear already, the same as
    /// `linear_view`.
    pub srgb_view: TextureView,
    /// Reads the texels as they are stored: for data, such as normal, metallic-roughness and
    /// occlusion maps in 3D.
    pub linear_view: TextureView,
    /// Whether the texels are 16-bit floats (`TextureFormat::RGBA16F`): HDR images, whose values
    /// go above 1.
    pub float: bool,
    pub width: u32,
    pub height: u32,
    /// The number of mip levels: a full chain down to 1×1, generated when the image is stored.
    pub mip_level_count: u32,
    /// Increases every time a texture is stored, under any id. A renderer that caches a bind group
    /// for the texture compares it, to notice that the image under an id was replaced.
    pub version: u64,
}

/// The raster images uploaded through the image context, by `ImageId`, shared by every wgpu
/// renderer on the device. 8-bit textures are stored with their alpha premultiplied, as loaded;
/// float (HDR) textures as they are, linear.
///
/// Every texture gets a full chain of mip levels, generated on the GPU from the image, so images
/// drawn smaller than their size, in 2D or 3D, are filtered instead of aliasing.
///
/// Nothing is freed yet: an image stays until the program ends.
#[derive(Debug, Default)]
pub struct WgpuTextureStore {
    textures: HashMap<ImageId, StoredTexture>,
    versions: u64,
    /// Made with the first texture stored.
    mipmaps: Option<MipmapGenerator>,
}

impl WgpuTextureStore {
    pub fn new() -> WgpuTextureStore {
        WgpuTextureStore::default()
    }

    pub fn get(&self, id: &ImageId) -> Option<&StoredTexture> {
        self.textures.get(id)
    }

    pub fn contains(&self, id: &ImageId) -> bool {
        self.textures.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    /// Upload `texture` and store it under `id`, replacing the texture stored there before.
    pub fn insert(&mut self, id: ImageId, texture: Texture, device: &Device, queue: &Queue) -> &StoredTexture {
        self.versions += 1;
        let mipmaps = self.mipmaps.get_or_insert_with(|| MipmapGenerator::new(device));
        let stored = upload(texture, self.versions, mipmaps, device, queue);
        self.textures.insert(id.clone(), stored);
        &self.textures[&id]
    }
}

fn upload(texture: Texture, version: u64, mipmaps: &mut MipmapGenerator, device: &Device, queue: &Queue) -> StoredTexture {
    let size = wgpu::Extent3d {
        width: texture.width,
        height: texture.height,
        depth_or_array_layers: 1,
    };

    // The texture's format, read as sRGB, and the same texels read as stored. A float texture
    // is linear: both are its own format.
    let (srgb_format, linear_format) = match texture.format {
        TextureFormat::RGBA8 => (wgpu::TextureFormat::Rgba8UnormSrgb, wgpu::TextureFormat::Rgba8Unorm),
        TextureFormat::BGRA8 => (wgpu::TextureFormat::Bgra8UnormSrgb, wgpu::TextureFormat::Bgra8Unorm),
        TextureFormat::RGBA16F => (wgpu::TextureFormat::Rgba16Float, wgpu::TextureFormat::Rgba16Float),
    };
    let float = srgb_format == linear_format;
    let linear_formats = [linear_format];

    let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("carbide_image"),
        size,
        mip_level_count: mip_level_count(texture.width, texture.height),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: srgb_format,
        // Rendered to, to generate the mip levels, and copied from, to read images back.
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: if float { &[] } else { &linear_formats },
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &wgpu_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: Default::default(),
        },
        texture.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(texture.bytes_per_row),
            rows_per_image: Some(texture.height),
        },
        size,
    );

    // Through sRGB views, so the levels are averaged in linear space; a float texture is linear
    // already.
    mipmaps.generate(&wgpu_texture, srgb_format, device, queue);

    let srgb_view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let linear_view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("carbide_image_linear"),
        format: Some(linear_format),
        ..Default::default()
    });

    StoredTexture {
        width: wgpu_texture.width(),
        height: wgpu_texture.height(),
        mip_level_count: wgpu_texture.mip_level_count(),
        texture: wgpu_texture,
        srgb_view,
        linear_view,
        float,
        version,
    }
}

impl EnvironmentKey for WgpuTextureStore {
    type Value = WgpuTextureStore;
    type Access = Exclusive;
}

fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
    Box::pin(async {
        env.insert_owned::<WgpuTextureStore>(WgpuTextureStore::new())
    })
}

register!(Plugin {
    dependencies: &[],
    registers: TypeId::of::<WgpuTextureStore>(),
    register
});
