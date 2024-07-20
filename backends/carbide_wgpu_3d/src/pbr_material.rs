use encase::ShaderType;
use carbide_3d::material::material_flags::MaterialFlags;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::material::sample_type::SampleType;
use carbide_3d::material::transparency::Transparency;
use carbide_core::color::ColorExt;
use carbide_core::render::matrix::{Matrix3, Vector3, Vector4, Zero};

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
}

unsafe impl bytemuck::Zeroable for WgpuPbrMaterial {}
unsafe impl bytemuck::Pod for WgpuPbrMaterial {}

impl WgpuPbrMaterial {
    pub(crate) fn from_material(material: &PbrMaterial) -> Self {
        let albedo = material.albedo.to_value();

        Self {
            uv_transform0: material.uv_transform0,
            uv_transform1: material.uv_transform1,
            albedo: Vector4::new(albedo.red(), albedo.green(), albedo.blue(), albedo.opacity()),
            roughness: material.roughness_factor.unwrap_or(0.0),
            metallic: material.metallic_factor.unwrap_or(0.0),
            reflectance: material.reflectance.to_value(0.5),
            clear_coat: material.clearcoat_factor.unwrap_or(0.0),
            clear_coat_roughness: material.clearcoat_roughness_factor.unwrap_or(0.0),
            emissive: material.emissive.to_value(Vector3::<f32>::zero()),
            anisotropy: material.anisotropy.to_value(0.0),
            ambient_occlusion: material.ao_factor.unwrap_or(1.0),
            alpha_cutout: match material.transparency {
                Transparency::Cutout { cutout } => cutout,
                _ => 0.0,
            },
            material_flags: {
                let mut flags = material.albedo.to_flags();
                flags |= material.normal.to_flags();
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
        }
    }
}