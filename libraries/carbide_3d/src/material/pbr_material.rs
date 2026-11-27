use carbide::draw::ImageId;
use crate::material::albedo_component::AlbedoComponent;
use crate::material::ao_mr_textures::AoMRTextures;
use crate::material::clearcoat_textures::ClearcoatTextures;
use crate::material::material_component::MaterialComponent;
use crate::material::normal_texture::NormalTexture;
use crate::material::sample_type::SampleType;
use crate::material::transparency::Transparency;
use crate::material::Material;
use crate::material::material_texture::MaterialTexture;
use carbide::color::{BLACK, WHITE};
use carbide::environment::Environment;
use carbide::draw::Color;
use carbide::math::{Matrix3, SquareMatrix, Vector2};
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
    /// How much the occlusion map darkens, from 0 (not at all) to 1 (as the map says): the map's
    /// value becomes `1 + strength · (value − 1)`, as glTF's `occlusionTexture.strength`. 1 by
    /// default.
    pub occlusion_strength: Box<dyn AnyReadState<T=f32>>,
    /// Scales the normal map's sideways components: below 1 flattens the bumps, above 1 deepens
    /// them, as glTF's `normalTexture.scale`. 1 by default.
    pub normal_scale: Box<dyn AnyReadState<T=f32>>,
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
    pub emissive_texture: Box<dyn AnyReadState<T=Option<MaterialTexture>>>,
    /// How much light a dielectric reflects head-on, from 0 to 1, where 0.5 is a reflectance of
    /// 4%, as most dielectrics have. Ignored by metals. 0.5 by default.
    pub reflectance: Box<dyn AnyReadState<T=f32>>,
    /// Not used by the renderer yet.
    pub anisotropy: MaterialComponent<f32>,
    /// Transforms the texture coordinates of every map, after the map's own transform
    /// ([`MaterialTexture::transform`]): a 2D affine transform, for tiling, offsetting or turning
    /// all the maps together. The identity by default.
    pub uv_transform: Box<dyn AnyReadState<T=Matrix3<f32>>>,
    // TODO: Make unlit a different shader entirely.
    pub unlit: bool,
    /// Whether the back of the material's triangles is drawn too, lit as a surface facing the
    /// other way: for leaves, cloth, paper and other thin surfaces seen from both sides, as glTF's
    /// `doubleSided`. False by default: the back faces are skipped, as for closed shapes.
    pub double_sided: Box<dyn AnyReadState<T=bool>>,
    /// Whether maps are sampled linearly (smooth, with mipmaps and anisotropic filtering), the
    /// default, or nearest (pixelated): the filtering of every map without its own
    /// ([`MaterialTexture::filtering`](crate::material::material_texture::MaterialTexture::filtering)).
    pub sample_type: Box<dyn AnyReadState<T=SampleType>>,
}

impl StateSync for PbrMaterial {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.albedo.sync(env)
            | self.normal.sync(env)
            | self.aomr_textures.sync(env)
            | self.clearcoat_textures.sync(env)
            | self.emissive_texture.sync(env)
            | self.uv_transform.sync(env)
            | self.sample_type.sync(env)
            | self.ambient_occlusion.sync(env)
            | self.occlusion_strength.sync(env)
            | self.normal_scale.sync(env)
            | self.metallic.sync(env)
            | self.roughness_factor.sync(env)
            | self.clearcoat.sync(env)
            | self.clearcoat_roughness.sync(env)
            | self.emissive.sync(env)
            | self.emissive_strength.sync(env)
            | self.reflectance.sync(env)
            | self.double_sided.sync(env)
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
            occlusion_strength: Box::new(1.0f32),
            normal_scale: Box::new(1.0f32),
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
            uv_transform: Box::new(Matrix3::<f32>::identity()),
            unlit: false,
            double_sided: Box::new(false),
            sample_type: SampleType::Linear.as_dyn_read(),
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
        fn images(texture: Option<&MaterialTexture>) -> Option<&ImageId> {
            texture.map(|texture| &texture.image)
        }

        let mut textures: Vec<ImageId> = Vec::new();
        for id in [
            images(albedo.as_ref()),
            images(normal.as_ref()),
            images(aomr.to_ao_texture()),
            images(aomr.to_metallic_texture()),
            images(aomr.to_roughness_texture()),
            images(clearcoat.to_clearcoat_texture()),
            images(clearcoat.to_clearcoat_roughness_texture()),
            images(emissive.as_ref()),
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

    /// How the material's alpha is used: ignored (`Opaque`, the default), as a cutoff below which
    /// the surface is cut away (`Cutout`), or blended with what is behind it (`Blend`), drawn after
    /// the opaque meshes, farthest first, and casting shadows where its alpha is at least 0.5.
    pub fn transparency(mut self, transparency: Transparency) -> PbrMaterial {
        self.transparency = transparency;
        self
    }

    /// Draw the back of the material's triangles too, lit as a surface facing the other way: for
    /// leaves, cloth, paper and other thin surfaces seen from both sides, as glTF's `doubleSided`.
    /// Off by default, which skips back faces, as for closed shapes.
    pub fn double_sided<D: IntoReadState<bool>>(mut self, double_sided: D) -> PbrMaterial {
        self.double_sided = Box::new(double_sided.into_read_state());
        self
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

    /// How much the occlusion map darkens, from 0 (not at all) to 1 (as the map says, the
    /// default): the map's value becomes `1 + strength · (value − 1)`, as glTF's
    /// `occlusionTexture.strength`.
    pub fn occlusion_strength<S: IntoReadState<f32>>(mut self, strength: S) -> PbrMaterial {
        self.occlusion_strength = Box::new(strength.into_read_state());
        self
    }

    /// Scales the normal map's sideways components: below 1 flattens the bumps, 0 removes them,
    /// and above 1 deepens them, as glTF's `normalTexture.scale`. 1 by default.
    pub fn normal_scale<S: IntoReadState<f32>>(mut self, scale: S) -> PbrMaterial {
        self.normal_scale = Box::new(scale.into_read_state());
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
    pub fn emissive_map<T: IntoReadState<Option<MaterialTexture>>>(self, image: T) -> PbrMaterial {
        PbrMaterial {
            emissive_texture: image.into_dyn_read_state(),
            ..self
        }
    }

    /// The transform of every map's texture coordinates, applied after the map's own
    /// ([`MaterialTexture::transform`]): a 2D affine transform as a 3×3 matrix, such as
    /// `Matrix3::from_nonuniform_scale(2.0, 2.0)`.
    pub fn uv_transform<T: IntoReadState<Matrix3<f32>>>(self, transform: T) -> PbrMaterial {
        PbrMaterial {
            uv_transform: transform.into_dyn_read_state(),
            ..self
        }
    }

    /// Repeat the maps `tiling.x` times across and `tiling.y` times down: a `uv_transform` that
    /// scales the texture coordinates. Textures repeat, so 2 shows each map twice in each
    /// direction.
    pub fn tiling<T: IntoReadState<Vector2<f32>>>(self, tiling: T) -> PbrMaterial {
        let transform = Map1::read_map(tiling.into_read_state(), |tiling: &Vector2<f32>| Matrix3::from_nonuniform_scale(tiling.x, tiling.y));
        self.uv_transform(transform)
    }

    /// Whether maps are sampled linearly, the default, or nearest, which shows their pixels: for
    /// every map that doesn't set its own filtering.
    pub fn sample_type<T: IntoReadState<SampleType>>(self, sample_type: T) -> PbrMaterial {
        PbrMaterial {
            sample_type: sample_type.into_dyn_read_state(),
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