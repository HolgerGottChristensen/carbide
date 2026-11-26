use encase::ShaderType;
use wgpu::util::{DeviceExt, TextureDataOrder};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device, Extent3d, Queue, ShaderStages, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension};
use carbide_wgpu_core::WgpuTextureStore;
use carbide_3d::material::material_flags::MaterialFlags;
use carbide_3d::material::material_texture::MaterialTexture;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::material::sample_type::SampleType;
use carbide_3d::material::transparency::Transparency;
use carbide_core::color::ColorExt;
use carbide_core::draw::ImageId;
use carbide_core::environment::Environment;
use carbide_core::math::{Matrix3, SquareMatrix, Vector3, Vector4};
use carbide_core::state::ReadState;

#[derive(Debug, Copy, Clone, ShaderType, PartialEq)]
pub struct WgpuPbrMaterial {
    /// Each slot's texture-coordinate transform, in binding order: the map's own, then the
    /// material's `uv_transform`.
    map_transforms: [Matrix3<f32>; SLOT_COUNT],

    albedo: Vector4<f32>,
    emissive: Vector3<f32>,
    roughness: f32,
    metallic: f32,
    reflectance: f32,
    clear_coat: f32,
    clear_coat_roughness: f32,
    anisotropy: f32,
    ambient_occlusion: f32,
    alpha_cutout: f32,

    material_flags: u32,
    texture_enable: u32,
    occlusion_strength: f32,
    normal_scale: f32,
    /// A bit per slot, in binding order, set when the map reads the second texture coordinates.
    uv_sets: u32,
}

unsafe impl bytemuck::Zeroable for WgpuPbrMaterial {}
unsafe impl bytemuck::Pod for WgpuPbrMaterial {}

impl WgpuPbrMaterial {
    /// Whether the material blends with what is behind it (`Transparency::Blend`).
    pub(crate) fn blended(&self) -> bool {
        MaterialFlags::from_bits_truncate(self.material_flags).contains(MaterialFlags::ALPHA_BLEND)
    }

    /// Whether both sides of the material's triangles are drawn.
    pub(crate) fn double_sided(&self) -> bool {
        MaterialFlags::from_bits_truncate(self.material_flags).contains(MaterialFlags::DOUBLE_SIDED)
    }

    /// The same values, with `texture_enable` set to the slots whose images are loaded.
    pub(crate) fn with_texture_enable(self, texture_enable: u32) -> Self {
        WgpuPbrMaterial { texture_enable, ..self }
    }

    /// The material's values, as the shader reads them. `texture_enable` is left at 0: it depends
    /// on which images have loaded, and is set when the frame is drawn.
    pub(crate) fn from_material(material: &PbrMaterial, _env: &mut Environment) -> Self {
        // Colours are given in sRGB; the shader lights in linear space, as the 2D renderer draws.
        let albedo = material.albedo.value().to_value().gamma_srgb_to_linear();

        let uv_transform = *material.uv_transform.value();
        let maps = slot_textures(material);

        Self {
            map_transforms: maps.each_ref().map(|map| uv_transform * map.as_ref().map_or(Matrix3::identity(), |map| map.transform)),
            uv_sets: maps.iter().enumerate().fold(0, |bits, (slot, map)| {
                bits | ((map.as_ref().is_some_and(|map| map.uv_set == 1) as u32) << slot)
            }),
            albedo: Vector4::new(albedo.red(), albedo.green(), albedo.blue(), albedo.opacity()),
            // Unset, the roughness is the map's when there is one (a factor of 1, as glTF's
            // default), and 0 without one.
            roughness: material.roughness_factor.value().unwrap_or(
                if material.aomr_textures.value().to_roughness_texture().is_some() { 1.0 } else { 0.0 }
            ),
            metallic: *material.metallic.value(),
            reflectance: *material.reflectance.value(),
            clear_coat: *material.clearcoat.value(),
            clear_coat_roughness: *material.clearcoat_roughness.value(),
            // Linear, times the strength, as the shader adds it to the light leaving the surface.
            emissive: {
                let emissive = material.emissive.value().gamma_srgb_to_linear();
                Vector3::new(emissive.red(), emissive.green(), emissive.blue()) * *material.emissive_strength.value()
            },
            anisotropy: material.anisotropy.to_value(0.0),
            ambient_occlusion: *material.ambient_occlusion.value(),
            alpha_cutout: match material.transparency {
                Transparency::Cutout { cutout } => cutout,
                _ => 0.0,
            },
            material_flags: {
                let mut flags = material.albedo.value().to_flags();
                flags |= material.normal.value().to_flags();
                flags |= material.aomr_textures.value().to_flags();
                flags |= material.clearcoat_textures.value().to_flags();
                flags.set(MaterialFlags::UNLIT, material.unlit);
                flags.set(MaterialFlags::ALPHA_BLEND, matches!(material.transparency, Transparency::Blend));
                flags.set(MaterialFlags::DOUBLE_SIDED, *material.double_sided.value());
                flags.set(
                    MaterialFlags::NEAREST,
                    match *material.sample_type.value() {
                        SampleType::Nearest => true,
                        SampleType::Linear => false,
                    },
                );
                //println!("{:#?}", flags);
                flags.bits()
            },
            // Set per frame from the texture store (`WgpuPbrMaterialTextures::enabled`).
            texture_enable: 0,
            occlusion_strength: *material.occlusion_strength.value(),
            normal_scale: *material.normal_scale.value(),
        }
    }
}

/// The images a material samples, one per slot of the material bind group. `ImageId::None` is an
/// empty slot, which gets a fallback texture.
#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
pub struct WgpuPbrMaterialTextures {
    pub(crate) albedo: ImageId,
    pub(crate) normal: ImageId,
    pub(crate) ambient_occlusion: ImageId,
    pub(crate) metallic: ImageId,
    pub(crate) roughness: ImageId,
    pub(crate) clear_coat: ImageId,
    pub(crate) clear_coat_roughness: ImageId,
    pub(crate) emissive: ImageId,
}

/// How a slot is read: colours through the texture's sRGB view, data through its linear view.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SlotKind {
    Color,
    Data,
    Normal,
}

impl WgpuPbrMaterialTextures {
    pub(crate) fn from_material(material: &PbrMaterial) -> Self {
        let [albedo, normal, ambient_occlusion, metallic, roughness, clear_coat, clear_coat_roughness, emissive] =
            slot_textures(material).map(|map| map.map(|map| map.image).unwrap_or_default());

        WgpuPbrMaterialTextures { albedo, normal, ambient_occlusion, metallic, roughness, clear_coat, clear_coat_roughness, emissive }
    }

    /// The slots in binding order, with how each is read.
    fn slots(&self) -> [(&ImageId, SlotKind); SLOT_COUNT] {
        [
            (&self.albedo, SlotKind::Color),
            (&self.normal, SlotKind::Normal),
            (&self.ambient_occlusion, SlotKind::Data),
            (&self.metallic, SlotKind::Data),
            (&self.roughness, SlotKind::Data),
            (&self.clear_coat, SlotKind::Data),
            (&self.clear_coat_roughness, SlotKind::Data),
            (&self.emissive, SlotKind::Color),
        ]
    }

    /// The stored version of each slot's image, 0 for an empty slot or an image not loaded yet.
    /// A bind group made for other versions is out of date.
    pub(crate) fn versions(&self, store: Option<&WgpuTextureStore>) -> [u64; SLOT_COUNT] {
        self.slots().map(|(id, _)| store.and_then(|store| store.get(id)).map_or(0, |stored| stored.version))
    }

    /// A bit per slot whose image is loaded, in binding order: the shader samples those, and uses
    /// the material's values alone for the others.
    pub(crate) fn enabled(&self, store: Option<&WgpuTextureStore>) -> u32 {
        self.versions(store).iter().enumerate().fold(0, |bits, (slot, version)| bits | ((*version != 0) as u32) << slot)
    }
}

/// The map in each slot of the material bind group, in binding order: albedo, normal, occlusion,
/// metallic, roughness, clear coat, clear-coat roughness and emission.
fn slot_textures(material: &PbrMaterial) -> [Option<MaterialTexture>; SLOT_COUNT] {
    let aomr = material.aomr_textures.value();
    let clearcoat = material.clearcoat_textures.value();
    [
        material.albedo.value().to_texture().cloned(),
        material.normal.value().to_texture().cloned(),
        aomr.to_ao_texture().cloned(),
        aomr.to_metallic_texture().cloned(),
        aomr.to_roughness_texture().cloned(),
        clearcoat.to_clearcoat_texture().cloned(),
        clearcoat.to_clearcoat_roughness_texture().cloned(),
        material.emissive_texture.value().clone(),
    ]
}

/// The number of texture slots in the material bind group.
pub(crate) const SLOT_COUNT: usize = 8;

pub(crate) fn create_pbr_bind_group_layout(device: &Device) -> BindGroupLayout {
    let entries: Vec<BindGroupLayoutEntry> = (0..SLOT_COUNT as u32).map(|binding| BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }).collect();

    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("carbide_3d_pbr_textures_layout"),
        entries: &entries,
    })
}

/// 1×1 textures for empty slots and images that have not loaded: white for colours and data, so
/// the material's values are used as they are, and a normal pointing straight out of the surface.
#[derive(Debug)]
pub(crate) struct FallbackTextures {
    white: TextureView,
    flat_normal: TextureView,
}

impl FallbackTextures {
    pub(crate) fn new(device: &Device, queue: &Queue) -> Self {
        let texel = |label: &str, rgba: [u8; 4]| {
            device.create_texture_with_data(queue, &TextureDescriptor {
                label: Some(label),
                size: Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }, TextureDataOrder::LayerMajor, &rgba).create_view(&TextureViewDescriptor::default())
        };

        FallbackTextures {
            white: texel("carbide_3d_fallback_white", [255, 255, 255, 255]),
            flat_normal: texel("carbide_3d_fallback_normal", [128, 128, 255, 255]),
        }
    }
}

pub(crate) fn create_pbr_bind_group(textures: &WgpuPbrMaterialTextures, store: Option<&WgpuTextureStore>, fallback: &FallbackTextures, layout: &BindGroupLayout, device: &Device) -> BindGroup {
    let views: Vec<&TextureView> = textures.slots().iter().map(|(id, kind)| {
        match (store.and_then(|store| store.get(id)), kind) {
            (Some(stored), SlotKind::Color) => &stored.srgb_view,
            (Some(stored), _) => &stored.linear_view,
            (None, SlotKind::Normal) => &fallback.flat_normal,
            (None, _) => &fallback.white,
        }
    }).collect();

    let entries: Vec<BindGroupEntry> = views.iter().enumerate().map(|(binding, view)| BindGroupEntry {
        binding: binding as u32,
        resource: BindingResource::TextureView(view),
    }).collect();

    device.create_bind_group(&BindGroupDescriptor {
        label: Some("carbide_3d_pbr_textures"),
        layout,
        entries: &entries,
    })
}
