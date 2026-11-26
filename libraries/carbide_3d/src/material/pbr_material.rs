use carbide::draw::ImageId;
use crate::material::albedo_component::AlbedoComponent;
use crate::material::ao_mr_textures::AoMRTextures;
use crate::material::clearcoat_textures::ClearcoatTextures;
use crate::material::material_component::MaterialComponent;
use crate::material::normal_texture::NormalTexture;
use crate::material::sample_type::SampleType;
use crate::material::transparency::Transparency;
use crate::material::Material;
use carbide::color::{BLACK, WHITE};
use carbide::environment::Environment;
use carbide::draw::Color;
use carbide::math::{Matrix3, SquareMatrix};
use carbide::state::{AnyReadState, IntoReadState, Map1, StateSync, ReadState};

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
    /// The occlusion, metallic and roughness maps, and how their channels are laid out. None by
    /// default.
    pub aomr_textures: Box<dyn AnyReadState<T=AoMRTextures>>,
    /// How much ambient light reaches the surface, from 0 (none) to 1 (all). 1 by default.
    pub ambient_occlusion: Box<dyn AnyReadState<T=f32>>,
    /// From 0 (a dielectric, such as plastic) to 1 (a metal). 0 by default.
    pub metallic: Box<dyn AnyReadState<T=f32>>,
    pub roughness_factor: Box<dyn AnyReadState<T=Option<f32>>>,
    /// The clear coat's strength and roughness maps, and how their channels are laid out. None
    /// by default.
    pub clearcoat_textures: Box<dyn AnyReadState<T=ClearcoatTextures>>,
    /// The strength of a clear coat over the surface, from 0 to 1. 0 by default. For now, it only
    /// widens the surface's highlight towards the coat's roughness; there is no separate coat
    /// highlight yet.
    pub clearcoat: Box<dyn AnyReadState<T=f32>>,
    /// The clear coat's roughness, from 0 to 1. 0 by default.
    pub clearcoat_roughness: Box<dyn AnyReadState<T=f32>>,
    /// The colour of light the surface gives off itself, in sRGB. Black (none) by default.
    pub emissive: Box<dyn AnyReadState<T=Color>>,
    /// How bright the emitted light is: at 1, with an exposure of 1, the colour shows at about
    /// its full brightness. 1 by default.
    pub emissive_strength: Box<dyn AnyReadState<T=f32>>,
    /// An image of the light the surface gives off, in sRGB, multiplied by `emissive` and
    /// `emissive_strength`. None by default.
    pub emissive_texture: Box<dyn AnyReadState<T=Option<ImageId>>>,
    /// How much light a dielectric reflects head-on, from 0 to 1, where 0.5 is a reflectance of
    /// 4%, as most dielectrics have. Ignored by metals. 0.5 by default.
    pub reflectance: Box<dyn AnyReadState<T=f32>>,
    /// Not used by the renderer yet.
    pub anisotropy: MaterialComponent<f32>,
    pub uv_transform0: Matrix3<f32>,
    pub uv_transform1: Matrix3<f32>,
    // TODO: Make unlit a different shader entirely.
    pub unlit: bool,
    pub sample_type: SampleType,
}

impl StateSync for PbrMaterial {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.albedo.sync(env)
            | self.normal.sync(env)
            | self.aomr_textures.sync(env)
            | self.clearcoat_textures.sync(env)
            | self.emissive_texture.sync(env)
            | self.ambient_occlusion.sync(env)
            | self.metallic.sync(env)
            | self.roughness_factor.sync(env)
            | self.clearcoat.sync(env)
            | self.clearcoat_roughness.sync(env)
            | self.emissive.sync(env)
            | self.emissive_strength.sync(env)
            | self.reflectance.sync(env)
    }
}

impl PbrMaterial {
    pub fn new() -> PbrMaterial {
        PbrMaterial {
            albedo: AlbedoComponent::Value(WHITE).as_dyn_read(),
            transparency: Default::default(),
            normal: NormalTexture::None.as_dyn_read(),
            aomr_textures: AoMRTextures::None.as_dyn_read(),
            ambient_occlusion: Box::new(1.0f32),
            metallic: Box::new(0.0f32),
            roughness_factor: None.as_dyn_read(),
            clearcoat_textures: ClearcoatTextures::None.as_dyn_read(),
            clearcoat: Box::new(0.0f32),
            clearcoat_roughness: Box::new(0.0f32),
            emissive: Box::new(BLACK),
            emissive_strength: Box::new(1.0f32),
            emissive_texture: None.as_dyn_read(),
            reflectance: Box::new(0.5f32),
            anisotropy: Default::default(),
            uv_transform0: Matrix3::identity(),
            uv_transform1: Matrix3::identity(),
            unlit: false,
            sample_type: Default::default(),
        }
    }
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self::new()
    }
}

impl PbrMaterial {
    /// Every image the material samples, each once: albedo, normal, occlusion, metallic,
    /// roughness and clear coat. Read from the current values, so sync the material first.
    pub fn textures(&self) -> Vec<ImageId> {
        let albedo = self.albedo.value().to_texture().cloned();
        let normal = self.normal.value().to_texture().cloned();
        let aomr = self.aomr_textures.value();
        let clearcoat = self.clearcoat_textures.value();
        let emissive = self.emissive_texture.value().clone();

        let mut textures: Vec<ImageId> = Vec::new();
        for id in [
            albedo.as_ref(),
            normal.as_ref(),
            aomr.to_ao_texture(),
            aomr.to_metallic_texture(),
            aomr.to_roughness_texture(),
            clearcoat.to_clearcoat_texture(),
            clearcoat.to_clearcoat_roughness_texture(),
            emissive.as_ref(),
        ].into_iter().flatten() {
            if *id != ImageId::None && !textures.contains(id) {
                textures.push(id.clone());
            }
        }

        textures
    }

    pub fn color<C: IntoReadState<AlbedoComponent>>(self, color: C) -> PbrMaterial {
        PbrMaterial {
            albedo: color.into_dyn_read_state(),
            ..self
        }
    }

    /// The occlusion, metallic and roughness maps. Each is a state, so a material can change maps
    /// while it is shown.
    pub fn aomr_textures<T: IntoReadState<AoMRTextures>>(self, textures: T) -> PbrMaterial {
        PbrMaterial {
            aomr_textures: textures.into_dyn_read_state(),
            ..self
        }
    }

    pub fn normal<C: IntoReadState<NormalTexture>>(self, normal: C) -> PbrMaterial {
        PbrMaterial {
            normal: normal.into_dyn_read_state(),
            ..self
        }
    }

    /// Show the material's colour as it is, unaffected by lights, exposure and tone mapping: for
    /// helpers such as axes, and for labels and markers that must keep their exact colour.
    pub fn unlit(mut self) -> PbrMaterial {
        self.unlit = true;
        self
    }

    pub fn roughness<R: IntoReadState<Option<f32>>>(self, roughness: R) -> PbrMaterial {
        PbrMaterial {
            roughness_factor: roughness.into_dyn_read_state(),
            ..self
        }
    }

    /// From 0 (a dielectric, such as plastic) to 1 (a metal). 0 by default.
    pub fn metallic<M: IntoReadState<f32>>(mut self, metallic: M) -> PbrMaterial {
        self.metallic = Box::new(metallic.into_read_state());
        self
    }

    /// How much light a dielectric reflects head-on, from 0 to 1: 0.5, the default, is the 4% of
    /// most dielectrics, and 1 is 16%, as of gemstones. Ignored by metals.
    pub fn reflectance<R: IntoReadState<f32>>(mut self, reflectance: R) -> PbrMaterial {
        self.reflectance = Box::new(reflectance.into_read_state());
        self
    }

    /// How much ambient light reaches the surface, from 0 (none) to 1 (all, the default). Light
    /// from the scene's lights is not affected.
    pub fn ambient_occlusion<A: IntoReadState<f32>>(mut self, ambient_occlusion: A) -> PbrMaterial {
        self.ambient_occlusion = Box::new(ambient_occlusion.into_read_state());
        self
    }

    /// A clear coat of `strength`, from 0 to 1, with `roughness`, from 0 to 1. For now, it only
    /// widens the surface's highlight towards the coat's roughness; there is no separate coat
    /// highlight yet.
    pub fn clearcoat<S: IntoReadState<f32>, R: IntoReadState<f32>>(mut self, strength: S, roughness: R) -> PbrMaterial {
        self.clearcoat = Box::new(strength.into_read_state());
        self.clearcoat_roughness = Box::new(roughness.into_read_state());
        self
    }

    /// Light the surface gives off itself, in a colour: added to the light it reflects, with no
    /// shading and no shadows. It doesn't light other objects. Black, none, by default.
    pub fn emissive<C: IntoReadState<Color>>(mut self, color: C) -> PbrMaterial {
        self.emissive = Box::new(color.into_read_state());
        self
    }

    /// The clear coat's strength and roughness maps. Each multiplies its value given with
    /// `clearcoat`.
    pub fn clearcoat_textures<T: IntoReadState<ClearcoatTextures>>(self, textures: T) -> PbrMaterial {
        PbrMaterial {
            clearcoat_textures: textures.into_dyn_read_state(),
            ..self
        }
    }

    /// An image of the light the surface gives off, in sRGB, multiplied by the `emissive` colour
    /// and strength. The emissive colour is black by default, so set it too, to white to use the
    /// image as it is.
    pub fn emissive_map<T: IntoReadState<Option<ImageId>>>(self, image: T) -> PbrMaterial {
        PbrMaterial {
            emissive_texture: image.into_dyn_read_state(),
            ..self
        }
    }

    /// How bright the emitted light is: at 1, the default, with an exposure of 1, the colour shows
    /// at about its full brightness.
    pub fn emissive_strength<S: IntoReadState<f32>>(mut self, strength: S) -> PbrMaterial {
        self.emissive_strength = Box::new(strength.into_read_state());
        self
    }

    /// Multiply the albedo by the mesh's vertex colours, which are sRGB, as carbide's colours are.
    /// Vertices are white unless given a colour, so a mesh without vertex colours is unchanged.
    pub fn vertex_colors(self) -> PbrMaterial {
        let albedo = Map1::read_map(self.albedo.clone(), |albedo: &AlbedoComponent| albedo.with_vertex_colors());
        PbrMaterial {
            albedo: Box::new(albedo),
            ..self
        }
    }
}

impl From<PbrMaterial> for Material {
    fn from(value: PbrMaterial) -> Self {
        Material::PBR(value)
    }
}