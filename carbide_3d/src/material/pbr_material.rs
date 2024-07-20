use carbide::draw::Color;
use carbide::render::matrix::{Matrix3, SquareMatrix, Vector3};
use crate::material::albedo_component::AlbedoComponent;
use crate::material::ao_mr_textures::AoMRTextures;
use crate::material::clearcoat_textures::ClearcoatTextures;
use crate::material::Material;
use crate::material::material_component::MaterialComponent;
use crate::material::normal_texture::NormalTexture;
use crate::material::sample_type::SampleType;
use crate::material::transparency::Transparency;

// Consider:
//
// - Green screen value
/// A set of textures and values that determine the how an object interacts with
/// light.
#[derive(Debug, Clone)]
pub struct PbrMaterial {
    pub albedo: AlbedoComponent,
    pub transparency: Transparency,
    pub normal: NormalTexture,
    pub aomr_textures: AoMRTextures,
    pub ao_factor: Option<f32>,
    pub metallic_factor: Option<f32>,
    pub roughness_factor: Option<f32>,
    pub clearcoat_textures: ClearcoatTextures,
    pub clearcoat_factor: Option<f32>,
    pub clearcoat_roughness_factor: Option<f32>,
    pub emissive: MaterialComponent<Vector3<f32>>,
    pub reflectance: MaterialComponent<f32>,
    pub anisotropy: MaterialComponent<f32>,
    pub uv_transform0: Matrix3<f32>,
    pub uv_transform1: Matrix3<f32>,
    // TODO: Make unlit a different shader entirely.
    pub unlit: bool,
    pub sample_type: SampleType,
}

impl PbrMaterial {
    pub fn new(color: Color) -> Material {
        Material::PBR(PbrMaterial {
            albedo: AlbedoComponent::Value(color),
            transparency: Default::default(),
            normal: Default::default(),
            aomr_textures: Default::default(),
            ao_factor: None,
            metallic_factor: None,
            roughness_factor: None,
            clearcoat_textures: Default::default(),
            clearcoat_factor: None,
            clearcoat_roughness_factor: None,
            emissive: Default::default(),
            reflectance: Default::default(),
            anisotropy: Default::default(),
            uv_transform0: Matrix3::identity(),
            uv_transform1: Matrix3::identity(),
            unlit: false,
            sample_type: Default::default(),
        })
    }
}