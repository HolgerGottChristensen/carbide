use std::ops::Deref;
use carbide::color::WHITE;
use carbide::draw::Color;
use carbide::environment::{Environment, EnvironmentStack};
use carbide::render::matrix::{Matrix3, SquareMatrix, Vector3};
use carbide::state::{AnyReadState, IntoReadState, ReadState, ReadStateExtNew, StateSync};
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
    pub albedo: Box<dyn AnyReadState<T=AlbedoComponent>>,
    pub transparency: Transparency,
    pub normal: Box<dyn AnyReadState<T=NormalTexture>>,
    pub aomr_textures: AoMRTextures,
    pub ao_factor: Option<f32>,
    pub metallic_factor: Option<f32>,
    pub roughness_factor: Box<dyn AnyReadState<T=Option<f32>>>,
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

impl StateSync for PbrMaterial {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        self.albedo.sync(env)
    }
}

impl PbrMaterial {
    pub fn new() -> PbrMaterial {
        PbrMaterial {
            albedo: AlbedoComponent::Value(WHITE).as_dyn_read(),
            transparency: Default::default(),
            normal: NormalTexture::None.as_dyn_read(),
            aomr_textures: Default::default(),
            ao_factor: None,
            metallic_factor: None,
            roughness_factor: None.as_dyn_read(),
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
        }
    }
}

impl PbrMaterial {
    pub fn color<C: IntoReadState<AlbedoComponent>>(self, color: C) -> PbrMaterial {
        PbrMaterial {
            albedo: color.into_read_state().as_dyn_read(),
            ..self
        }
    }

    pub fn normal<C: IntoReadState<NormalTexture>>(self, normal: C) -> PbrMaterial {
        PbrMaterial {
            normal: normal.into_read_state().as_dyn_read(),
            ..self
        }
    }

    pub fn roughness<R: IntoReadState<Option<f32>>>(self, roughness: R) -> PbrMaterial {
        PbrMaterial {
            roughness_factor: roughness.into_read_state().as_dyn_read(),
            ..self
        }
    }
}

impl From<PbrMaterial> for Material {
    fn from(value: PbrMaterial) -> Self {
        Material::PBR(value)
    }
}