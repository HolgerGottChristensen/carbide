use encase::ShaderType;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, Device};
use carbide_3d::material::material_flags::MaterialFlags;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::material::sample_type::SampleType;
use carbide_3d::material::transparency::Transparency;
use carbide_core::color::ColorExt;
use carbide_core::draw::ImageId;
use carbide_core::environment::Environment;
use carbide_core::math::{Matrix3, Vector3, Vector4};
use carbide_core::state::ReadState;

#[derive(Debug, Copy, Clone, ShaderType, PartialEq)]
pub struct WgpuPbrMaterial {
    uv_transform0: Matrix3<f32>,
    uv_transform1: Matrix3<f32>,

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
}

unsafe impl bytemuck::Zeroable for WgpuPbrMaterial {}
unsafe impl bytemuck::Pod for WgpuPbrMaterial {}

impl WgpuPbrMaterial {
    /// The material's values, as the shader reads them. Textures are not loaded yet (see
    /// `ImageContext3d`): the texture flags are set, but the shader doesn't sample them.
    pub(crate) fn from_material(material: &PbrMaterial, _env: &mut Environment) -> Self {
        // Colours are given in sRGB; the shader lights in linear space, as the 2D renderer draws.
        let albedo = material.albedo.value().to_value().gamma_srgb_to_linear();

        Self {
            uv_transform0: material.uv_transform0,
            uv_transform1: material.uv_transform1,
            albedo: Vector4::new(albedo.red(), albedo.green(), albedo.blue(), albedo.opacity()),
            roughness: material.roughness_factor.value().unwrap_or(0.0),
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
                flags |= material.aomr_textures.to_flags();
                flags |= material.clearcoat_textures.to_flags();
                flags.set(MaterialFlags::UNLIT, material.unlit);
                flags.set(
                    MaterialFlags::NEAREST,
                    match material.sample_type {
                        SampleType::Nearest => true,
                        SampleType::Linear => false,
                    },
                );
                //println!("{:#?}", flags);
                flags.bits()
            },
            texture_enable: {
                let list = &[
                    material.albedo.value().is_texture(),
                    material.normal.value().to_texture().is_some(),
                ];
                let mut bits = 0x0;
                for t in list.into_iter().rev() {
                    // Shift must happen first, if it happens second, the last bit will also be shifted
                    bits <<= 1;
                    bits |= *t as u32;
                }
                bits
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
pub struct WgpuPbrMaterialTextures {
    pub(crate) albedo: ImageId,
    pub(crate) normal: ImageId,
}

pub(crate) fn create_pbr_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            /*BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },*/
        ],
    })
}

pub(crate) fn create_pbr_bind_group(_textures: &WgpuPbrMaterialTextures, layout: &BindGroupLayout, device: &Device) -> BindGroup {
    // Textures are not loaded yet, so the layout has no entries, and neither has the bind group.
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("carbide_3d_pbr_textures"),
        layout,
        entries: &[],
    })
}
