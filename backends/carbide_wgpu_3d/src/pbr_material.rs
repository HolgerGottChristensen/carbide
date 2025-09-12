use encase::ShaderType;
use once_cell::sync::Lazy;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Device, ShaderStages, TextureSampleType, TextureViewDimension};
use carbide_3d::InnerImageContext3d;
use carbide_3d::material::material_flags::MaterialFlags;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::material::sample_type::SampleType;
use carbide_3d::material::transparency::Transparency;
use carbide_core::color::ColorExt;
use carbide_core::draw::{ImageId, Texture, TextureFormat};
use carbide_core::draw::pre_multiply::PreMultiply;
use carbide_core::environment::Environment;
use carbide_core::image;
use carbide_core::math::{Matrix3, Vector3, Vector4, Zero};
use carbide_core::state::ReadState;
use carbide_wgpu::WgpuContext;
use crate::image_context_3d::{ImageContext3d};

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
    pub(crate) fn from_material(material: &PbrMaterial, env: &mut Environment) -> Self {
        let albedo = material.albedo.value().to_value();

        if let Some(image_id) = material.albedo.value().to_texture() {
            load_image(image_id, env);
        }
        if let Some(image_id) = material.normal.value().to_texture() {
            load_image(image_id, env);
        }

        Self {
            uv_transform0: material.uv_transform0,
            uv_transform1: material.uv_transform1,
            albedo: Vector4::new(albedo.red(), albedo.green(), albedo.blue(), albedo.opacity()),
            roughness: material.roughness_factor.value().unwrap_or(0.0),
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

fn load_image(id: &ImageId, env: &mut Environment) {
    if !ImageContext3d.texture_exist(id, env) {
        let path = if id.is_relative() {
            let assets = carbide_core::locate_folder::Search::KidsThenParents(3, 5)
                .for_folder("assets")
                .unwrap();

            assets.join(id)
        } else {
            id.as_ref().to_path_buf()
        };

        let image = image::open(path)
            .expect("Couldn't load image")
            .pre_multiplied();

        let texture = Texture {
            width: image.width(),
            height: image.height(),
            bytes_per_row: image.width() * 4,
            format: TextureFormat::RGBA8,
            data: &image.to_rgba8().into_raw(),
        };

        ImageContext3d.update_texture(id.clone(), texture, env);
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

pub(crate) fn create_pbr_bind_group(textures: &WgpuPbrMaterialTextures, device: &Device) -> BindGroup {

    let layout = create_pbr_bind_group_layout(device);

    let views = &[
        //TEXTURES.get(&textures.albedo).unwrap().create_view(&Default::default()),
        //TEXTURES.get(&textures.normal).unwrap().create_view(&Default::default())
    ];

    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &views.into_iter().enumerate().map(|(idx, view)| {
            wgpu::BindGroupEntry {
                binding: idx as u32,
                resource: wgpu::BindingResource::TextureView(view),
            }
        }).collect::<Vec<_>>(),
    })
}