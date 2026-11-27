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
use carbide::draw::Color;
use carbide::impl_state_value;
use carbide::math::{Matrix3, SquareMatrix, Vector2};
use carbide::state::{AnyReadState, ConvertIntoRead, Map1, RMap1};

// Consider:
//
// - Green screen value
/// A set of textures and values that determine the how an object interacts with
/// light.
///
/// A plain value: build it with the builder methods, compare it, copy it, keep it as a constant.
/// To change a material while it is shown, give [`Object`](crate::Object) a state of it: a
/// `LocalState<PbrMaterial>`, whose fields `lens!` hands out as states for controls, or a map
/// that builds the material from other states.
#[derive(Debug, Clone, PartialEq)]
pub struct PbrMaterial {
    pub albedo: AlbedoComponent,
    pub transparency: Transparency,
    pub normal: NormalTexture,
    /// The occlusion, metallic and roughness maps, and how their channels are laid out. None by
    /// default.
    pub aomr_textures: AoMRTextures,
    /// How much ambient light reaches the surface, from 0 (none) to 1 (all). 1 by default.
    pub ambient_occlusion: f32,
    /// How much the occlusion map darkens, from 0 (not at all) to 1 (as the map says): the map's
    /// value becomes `1 + strength · (value − 1)`, as glTF's `occlusionTexture.strength`. 1 by
    /// default.
    pub occlusion_strength: f32,
    /// Scales the normal map's sideways components: below 1 flattens the bumps, above 1 deepens
    /// them, as glTF's `normalTexture.scale`. 1 by default.
    pub normal_scale: f32,
    /// From 0 (a dielectric, such as plastic) to 1 (a metal). 0 by default.
    pub metallic: f32,
    pub roughness_factor: Option<f32>,
    /// The clear coat's strength and roughness maps, and how their channels are laid out. None
    /// by default.
    pub clearcoat_textures: ClearcoatTextures,
    /// The strength of a clear coat over the surface, from 0 to 1. 0 by default. For now, it only
    /// widens the surface's highlight towards the coat's roughness; there is no separate coat
    /// highlight yet.
    pub clearcoat: f32,
    /// The clear coat's roughness, from 0 to 1. 0 by default.
    pub clearcoat_roughness: f32,
    /// The colour of light the surface gives off itself, in sRGB. Black (none) by default.
    pub emissive: Color,
    /// How bright the emitted light is: at 1, with an exposure of 1, the colour shows at about
    /// its full brightness. 1 by default.
    pub emissive_strength: f32,
    /// An image of the light the surface gives off, in sRGB, multiplied by `emissive` and
    /// `emissive_strength`. None by default.
    pub emissive_texture: Option<MaterialTexture>,
    /// How much light a dielectric reflects head-on, from 0 to 1, where 0.5 is a reflectance of
    /// 4%, as most dielectrics have. Ignored by metals. 0.5 by default.
    pub reflectance: f32,
    /// Not used by the renderer yet.
    pub anisotropy: MaterialComponent<f32>,
    /// Transforms the texture coordinates of every map, after the map's own transform
    /// ([`MaterialTexture::transform`]): a 2D affine transform, for tiling, offsetting or turning
    /// all the maps together. The identity by default.
    pub uv_transform: Matrix3<f32>,
    // TODO: Make unlit a different shader entirely.
    pub unlit: bool,
    /// Whether the back of the material's triangles is drawn too, lit as a surface facing the
    /// other way: for leaves, cloth, paper and other thin surfaces seen from both sides, as glTF's
    /// `doubleSided`. False by default: the back faces are skipped, as for closed shapes.
    pub double_sided: bool,
    /// Whether maps are sampled linearly (smooth, with mipmaps and anisotropic filtering), the
    /// default, or nearest (pixelated): the filtering of every map without its own
    /// ([`MaterialTexture::filtering`](crate::material::material_texture::MaterialTexture::filtering)).
    pub sample_type: SampleType,
}

impl_state_value!(PbrMaterial);

impl PbrMaterial {
    pub fn new() -> PbrMaterial {
        PbrMaterial {
            albedo: AlbedoComponent::Value(WHITE),
            transparency: Default::default(),
            normal: NormalTexture::None,
            aomr_textures: AoMRTextures::None,
            ambient_occlusion: 1.0,
            occlusion_strength: 1.0,
            normal_scale: 1.0,
            metallic: 0.0,
            roughness_factor: None,
            clearcoat_textures: ClearcoatTextures::None,
            clearcoat: 0.0,
            clearcoat_roughness: 0.0,
            emissive: BLACK,
            emissive_strength: 1.0,
            emissive_texture: None,
            reflectance: 0.5,
            anisotropy: Default::default(),
            uv_transform: Matrix3::identity(),
            unlit: false,
            double_sided: false,
            sample_type: SampleType::Linear,
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
    /// roughness and clear coat.
    pub fn textures(&self) -> Vec<ImageId> {
        let albedo = self.albedo.to_texture();
        let normal = self.normal.to_texture();
        let aomr = &self.aomr_textures;
        let clearcoat = &self.clearcoat_textures;
        let emissive = self.emissive_texture.as_ref();
        fn images(texture: Option<&MaterialTexture>) -> Option<&ImageId> {
            texture.map(|texture| &texture.image)
        }

        let mut textures: Vec<ImageId> = Vec::new();
        for id in [
            images(albedo),
            images(normal),
            images(aomr.to_ao_texture()),
            images(aomr.to_metallic_texture()),
            images(aomr.to_roughness_texture()),
            images(clearcoat.to_clearcoat_texture()),
            images(clearcoat.to_clearcoat_roughness_texture()),
            images(emissive),
        ].into_iter().flatten() {
            if *id != ImageId::None && !textures.contains(id) {
                textures.push(id.clone());
            }
        }

        textures
    }

    /// The surface's colour: a `Color`, an `ImageId` or a `MaterialTexture` to sample, or any
    /// other `AlbedoComponent`.
    pub fn color(mut self, color: impl Into<AlbedoComponent>) -> PbrMaterial {
        self.albedo = color.into();
        self
    }

    /// The occlusion, metallic and roughness maps.
    pub fn aomr_textures(mut self, textures: AoMRTextures) -> PbrMaterial {
        self.aomr_textures = textures;
        self
    }

    pub fn normal(mut self, normal: NormalTexture) -> PbrMaterial {
        self.normal = normal;
        self
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
    pub fn double_sided(mut self, double_sided: bool) -> PbrMaterial {
        self.double_sided = double_sided;
        self
    }

    /// Show the material's colour as it is, unaffected by lights, exposure and tone mapping: for
    /// helpers such as axes, and for labels and markers that must keep their exact colour.
    pub fn unlit(mut self) -> PbrMaterial {
        self.unlit = true;
        self
    }

    /// From 0 (a mirror) to 1 (fully rough), or `None` for the roughness map's own, or 0 without
    /// one. Takes an `f32` or an `Option<f32>`.
    pub fn roughness(mut self, roughness: impl Into<Option<f32>>) -> PbrMaterial {
        self.roughness_factor = roughness.into();
        self
    }

    /// From 0 (a dielectric, such as plastic) to 1 (a metal). 0 by default.
    pub fn metallic(mut self, metallic: f32) -> PbrMaterial {
        self.metallic = metallic;
        self
    }

    /// How much light a dielectric reflects head-on, from 0 to 1: 0.5, the default, is the 4% of
    /// most dielectrics, and 1 is 16%, as of gemstones. Ignored by metals.
    pub fn reflectance(mut self, reflectance: f32) -> PbrMaterial {
        self.reflectance = reflectance;
        self
    }

    /// How much ambient light reaches the surface, from 0 (none) to 1 (all, the default). Light
    /// from the scene's lights is not affected.
    pub fn ambient_occlusion(mut self, ambient_occlusion: f32) -> PbrMaterial {
        self.ambient_occlusion = ambient_occlusion;
        self
    }

    /// How much the occlusion map darkens, from 0 (not at all) to 1 (as the map says, the
    /// default): the map's value becomes `1 + strength · (value − 1)`, as glTF's
    /// `occlusionTexture.strength`.
    pub fn occlusion_strength(mut self, strength: f32) -> PbrMaterial {
        self.occlusion_strength = strength;
        self
    }

    /// Scales the normal map's sideways components: below 1 flattens the bumps, 0 removes them,
    /// and above 1 deepens them, as glTF's `normalTexture.scale`. 1 by default.
    pub fn normal_scale(mut self, scale: f32) -> PbrMaterial {
        self.normal_scale = scale;
        self
    }

    /// A clear coat of `strength`, from 0 to 1, with `roughness`, from 0 to 1. For now, it only
    /// widens the surface's highlight towards the coat's roughness; there is no separate coat
    /// highlight yet.
    pub fn clearcoat(mut self, strength: f32, roughness: f32) -> PbrMaterial {
        self.clearcoat = strength;
        self.clearcoat_roughness = roughness;
        self
    }

    /// Light the surface gives off itself, in a colour: added to the light it reflects, with no
    /// shading and no shadows. It doesn't light other objects. Black, none, by default.
    pub fn emissive(mut self, color: Color) -> PbrMaterial {
        self.emissive = color;
        self
    }

    /// The clear coat's strength and roughness maps. Each multiplies its value given with
    /// `clearcoat`.
    pub fn clearcoat_textures(mut self, textures: ClearcoatTextures) -> PbrMaterial {
        self.clearcoat_textures = textures;
        self
    }

    /// An image of the light the surface gives off, in sRGB, multiplied by the `emissive` colour
    /// and strength. The emissive colour is black by default, so set it too, to white to use the
    /// image as it is. Takes a `MaterialTexture` or an `Option` of one.
    pub fn emissive_map(mut self, image: impl Into<Option<MaterialTexture>>) -> PbrMaterial {
        self.emissive_texture = image.into();
        self
    }

    /// The transform of every map's texture coordinates, applied after the map's own
    /// ([`MaterialTexture::transform`]): a 2D affine transform as a 3×3 matrix, such as
    /// `Matrix3::from_nonuniform_scale(2.0, 2.0)`.
    pub fn uv_transform(mut self, transform: Matrix3<f32>) -> PbrMaterial {
        self.uv_transform = transform;
        self
    }

    /// Repeat the maps `tiling.x` times across and `tiling.y` times down: a `uv_transform` that
    /// scales the texture coordinates. Textures repeat, so 2 shows each map twice in each
    /// direction.
    pub fn tiling(self, tiling: Vector2<f32>) -> PbrMaterial {
        self.uv_transform(Matrix3::from_nonuniform_scale(tiling.x, tiling.y))
    }

    /// Whether maps are sampled linearly, the default, or nearest, which shows their pixels: for
    /// every map that doesn't set its own filtering.
    pub fn sample_type(mut self, sample_type: SampleType) -> PbrMaterial {
        self.sample_type = sample_type;
        self
    }

    /// How bright the emitted light is: at 1, the default, with an exposure of 1, the colour shows
    /// at about its full brightness.
    pub fn emissive_strength(mut self, strength: f32) -> PbrMaterial {
        self.emissive_strength = strength;
        self
    }

    /// Multiply the albedo by the mesh's vertex colours, which are sRGB, as carbide's colours are.
    /// Vertices are white unless given a colour, so a mesh without vertex colours is unchanged.
    pub fn vertex_colors(mut self) -> PbrMaterial {
        self.albedo = self.albedo.with_vertex_colors();
        self
    }
}

impl From<PbrMaterial> for Material {
    fn from(value: PbrMaterial) -> Self {
        Material::PBR(value)
    }
}

/// A state of a `PbrMaterial` is a state of a `Material`, so an `Object` takes either.
impl ConvertIntoRead<Material> for PbrMaterial {
    type Output<G: AnyReadState<T=PbrMaterial> + Clone> = RMap1<fn(&PbrMaterial)->Material, PbrMaterial, Material, G>;

    fn convert<F: AnyReadState<T=PbrMaterial> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |material| Material::PBR(material.clone()))
    }
}
