//! OBJ models with their MTL materials.
//!
//! MTL describes Phong materials; carbide_3d's are physically based. [`material_from_mtl`] maps
//! one onto the other:
//! - **Colour:** `Kd`, the diffuse colour, with `d`, the dissolve, as its opacity. Colours in MTL
//!   files are linear, as Blender and most renderers read them, so they are converted to sRGB,
//!   as carbide's colours are.
//! - **Glass:** a material with a glass or refraction illumination model (`illum` 4, 6, 7 or 9)
//!   and a transmission filter `Tf`, and no `d` below 1, has an opacity of 1 minus `Tf`'s mean,
//!   as exporters such as 3ds Max write glass (`Tf 0.4 0.4 0.4` with `d 1`). `Tf`'s colour isn't
//!   used: tinting the light behind takes another blend mode.
//! - **Roughness:** `Pr` when the file has the PBR extension. Otherwise from `Ns`, the Phong
//!   exponent: a Blinn-Phong highlight of exponent `n` is about as wide as a GGX highlight of
//!   alpha `√(2 / (n + 2))`, and roughness is the square root of alpha.
//! - **Metallic:** `Pm` from the PBR extension, otherwise 0.
//! - **Reflectance:** `Ks`, the specular colour, scales the usual 4% of dielectrics by its
//!   luminance: a `Ks` of 1 gives a reflectance of 0.5 (4%), of 0.5 gives 0.25 (1%), and of 0
//!   gives no highlight at all.
//! - **Clear coat:** `Pc` and `Pcr` from the PBR extension.
//! - **Emission:** `Ke`.
//! - **Illumination models:** `illum 0` (a constant colour) makes the material unlit, and
//!   `illum 1` (no specular) gives it no reflectance.
//!
//! - **Maps,** with paths relative to the MTL file:
//!   - `map_Kd`: the colour map, multiplied by `Kd`, as MTL defines it.
//!   - `norm` (PBR extension), or else `map_Bump` / `bump`: a normal map, with green up (OpenGL).
//!     Classic MTL meant `map_Bump` as a height map, but exporters such as Blender's write normal
//!     maps there, so it is read as one.
//!   - `map_Pr`, `map_Pm` (PBR extension): roughness and metallic maps, read from red. Without
//!     `Pr` or `Pm`, the map is used as it is (a factor of 1).
//!   - `map_Ke`: an emission map, multiplied by `Ke`, or by white without one.
//!   - `-s` and `-o` on a map (scale and offset) become that map's own transform
//!     ([`MaterialTexture::transform`]). Other options are skipped.
//!
//! Not used: `Ka`, since ambient light is the scene's; `Tr`, which exporters disagree on (some
//! write transparency, others opacity); `Ni`; and `map_Ka`, `map_Ks`, `map_Ns` and `map_d`,
//! which have no physically based counterpart, or no slot yet. An opacity below 1 sets
//! [`Transparency::Blend`].

use std::path::{Path, PathBuf};

use carbide::color::{rgba, Color, WHITE};
use carbide::draw::ImageId;
use carbide::math::{Matrix3, Matrix4, SquareMatrix};

use crate::material::ao_mr_textures::AoMRTextures;
use crate::material::material_texture::MaterialTexture;
use crate::material::normal_texture::NormalTexture;
use crate::material::normal_texture_y_direction::NormalTextureYDirection;

use crate::material::pbr_material::PbrMaterial;
use crate::material::transparency::Transparency;
use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render::{Render3d, RenderContext3d};
use crate::{Mesh, Object};

/// A linear colour component converted to sRGB, with the exact sRGB curve.
pub(crate) fn linear_to_srgb(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    if value <= 0.0031308 { value * 12.92 } else { 1.055 * value.powf(1.0 / 2.4) - 0.055 }
}

/// A linear MTL colour as a carbide colour, with `alpha`.
pub(crate) fn color(linear: [f32; 3], alpha: f32) -> Color {
    rgba(linear_to_srgb(linear[0]), linear_to_srgb(linear[1]), linear_to_srgb(linear[2]), alpha.clamp(0.0, 1.0))
}

/// A number from the PBR extension, which tobj leaves among the unknown parameters.
fn extension(material: &tobj::Material, key: &str) -> Option<f32> {
    material.unknown_param.get(key).and_then(|value| value.split_whitespace().next()?.parse().ok())
}

/// A colour from the unknown parameters: `Ke` in the tobj version used.
fn extension_color(material: &tobj::Material, key: &str) -> Option<[f32; 3]> {
    let mut values = material.unknown_param.get(key)?.split_whitespace().map(|value| value.parse::<f32>().ok());
    Some([values.next()??, values.next()??, values.next()??])
}

/// The illumination models the MTL specification calls glass (4, 9) and refraction (6, 7).
const GLASS_ILLUMINATION_MODELS: [u8; 4] = [4, 6, 7, 9];

/// A material's opacity: `d`, or for glass without a `d` below 1, 1 minus the mean of its
/// transmission filter `Tf`.
fn opacity(material: &tobj::Material) -> f32 {
    let dissolve = material.dissolve.unwrap_or(1.0);
    let glass = material.illumination_model.is_some_and(|model| GLASS_ILLUMINATION_MODELS.contains(&model));
    match extension_color(material, "Tf") {
        Some([r, g, b]) if glass && dissolve >= 1.0 => (1.0 - (r + g + b) / 3.0).clamp(0.0, 1.0),
        _ => dissolve,
    }
}

/// The roughness whose GGX highlight is about as wide as a Blinn-Phong highlight of `exponent`.
pub fn roughness_from_phong_exponent(exponent: f32) -> f32 {
    let alpha = (2.0 / (exponent.max(0.0) + 2.0)).sqrt();
    alpha.sqrt().clamp(0.0, 1.0)
}

/// A texture map entry of an MTL file: its file, and the options before it that carbide uses.
#[derive(Debug, Clone, PartialEq)]
pub struct MtlMap {
    /// The file, as written: relative to the MTL file, or absolute.
    pub file: String,
    /// `-s`: the texture coordinates' scale, (1, 1) by default.
    pub scale: [f32; 2],
    /// `-o`: the texture coordinates' offset, (0, 0) by default.
    pub offset: [f32; 2],
}

impl MtlMap {
    /// Parse the text after a map keyword: options, each a `-name` followed by its values, then
    /// the file name, which may contain spaces. Returns `None` when there is no file name.
    pub fn parse(text: &str) -> Option<MtlMap> {
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let mut map = MtlMap { file: String::new(), scale: [1.0, 1.0], offset: [0.0, 0.0] };
        let mut i = 0;

        while i < tokens.len() && tokens[i].starts_with('-') && tokens[i].len() > 1 {
            let option = tokens[i];
            i += 1;

            // How many values the option takes: up to three numbers for -o, -s and -t, two for -mm,
            // one for the others.
            let numbers = |i: usize, max: usize| tokens[i..].iter().take(max).take_while(|token| token.parse::<f32>().is_ok()).map(|token| token.parse::<f32>().unwrap()).collect::<Vec<f32>>();
            match option {
                "-s" | "-o" | "-t" => {
                    let values = numbers(i, 3);
                    i += values.len();
                    let u = values.first().copied();
                    let v = values.get(1).copied().or(u);
                    match (option, u, v) {
                        ("-s", Some(u), Some(v)) => map.scale = [u, v],
                        ("-o", Some(u), Some(v)) => map.offset = [u, v],
                        _ => {}
                    }
                }
                "-mm" => i += 2,
                _ => i += 1,
            }
        }

        map.file = tokens.get(i..)?.join(" ");
        (!map.file.is_empty()).then_some(map)
    }

    /// The image, with its path resolved against `folder`, the MTL file's folder.
    pub fn image(&self, folder: &Path) -> ImageId {
        let path = PathBuf::from(&self.file);
        ImageId::new(if path.is_absolute() { path } else { folder.join(path) })
    }

    /// The image as a material's map, with the map's own scale and offset.
    pub fn texture(&self, folder: &Path) -> MaterialTexture {
        MaterialTexture::new(self.image(folder)).transform(self.uv_transform())
    }

    /// The scale and offset as a transform of carbide's texture coordinates. MTL's apply to OBJ's
    /// coordinates, whose v points up; carbide's v is `1 - v`, so the offset in v becomes
    /// `1 - scale - offset`.
    pub fn uv_transform(&self) -> Matrix3<f32> {
        let [su, sv] = self.scale;
        let [ou, ov] = self.offset;
        // Columns: x, y and the translation of the 2D affine transform.
        Matrix3::new(su, 0.0, 0.0, 0.0, sv, 0.0, ou, 1.0 - sv - ov, 1.0)
    }
}

/// The physically based material closest to an MTL material. See the [module](self) for how each
/// value is mapped. Without `folder`, the MTL file's folder, texture maps are left out.
pub fn material_from_mtl(material: &tobj::Material) -> PbrMaterial {
    material_from_mtl_in(material, None)
}

/// [`material_from_mtl`], with the texture maps, their paths resolved against `folder`, the MTL
/// file's folder.
pub fn material_from_mtl_in(material: &tobj::Material, folder: Option<&Path>) -> PbrMaterial {
    let opacity = opacity(material);
    let diffuse = material.diffuse.unwrap_or([0.8, 0.8, 0.8]);

    let roughness = extension(material, "Pr")
        .unwrap_or_else(|| roughness_from_phong_exponent(material.shininess.unwrap_or(32.0)));
    let metallic = extension(material, "Pm").unwrap_or(0.0);

    let reflectance = match (material.illumination_model, material.specular) {
        (Some(1), _) => 0.0,
        (_, Some([r, g, b])) => (0.5 * (0.2126 * r + 0.7152 * g + 0.0722 * b)).clamp(0.0, 1.0),
        (_, None) => 0.5,
    };

    let mut pbr = PbrMaterial::new()
        .color(color(diffuse, opacity))
        .roughness(Some(roughness))
        .metallic(metallic)
        .reflectance(reflectance)
        .clearcoat(extension(material, "Pc").unwrap_or(0.0), extension(material, "Pcr").unwrap_or(0.0));

    if let Some(emissive) = extension_color(material, "Ke") {
        pbr = pbr.emissive(color(emissive, 1.0));
    }

    if let Some(folder) = folder {
        pbr = with_maps(pbr, material, folder);
    }

    if opacity < 1.0 {
        pbr.transparency = Transparency::Blend;
    }

    if material.illumination_model == Some(0) {
        pbr = pbr.unlit();
    }

    pbr
}

/// `pbr` with the texture maps of `material`, resolved against `folder`.
fn with_maps(mut pbr: PbrMaterial, material: &tobj::Material, folder: &Path) -> PbrMaterial {
    let map = |text: Option<&String>| text.and_then(|text| MtlMap::parse(text));
    let unknown = |key: &str| map(material.unknown_param.get(key));

    if let Some(colour) = map(material.diffuse_texture.as_ref()) {
        let diffuse = material.diffuse.unwrap_or([1.0, 1.0, 1.0]);
        let opacity = opacity(material);
        pbr = pbr
            .color(crate::material::albedo_component::AlbedoComponent::TextureValue { texture: colour.texture(folder), value: color(diffuse, opacity) });
    }

    if let Some(normal) = unknown("norm").or_else(|| map(material.normal_texture.as_ref())) {
        pbr = pbr.normal(NormalTexture::Tricomponent(normal.texture(folder), NormalTextureYDirection::Up));
    }

    let roughness = unknown("map_Pr");
    let metallic = unknown("map_Pm");
    if roughness.is_some() || metallic.is_some() {
        if roughness.is_some() && extension(material, "Pr").is_none() {
            pbr = pbr.roughness(None::<f32>);
        }
        if metallic.is_some() && extension(material, "Pm").is_none() {
            pbr = pbr.metallic(1.0);
        }
        pbr = pbr.aomr_textures(AoMRTextures::BWSplit {
            ao_texture: None,
            m_texture: metallic.map(|map| map.texture(folder)),
            r_texture: roughness.map(|map| map.texture(folder)),
        });
    }

    if let Some(emission) = unknown("map_Ke") {
        if extension_color(material, "Ke").is_none() {
            pbr = pbr.emissive(WHITE);
        }
        pbr = pbr.emissive_map(Some(emission.texture(folder)));
    }

    pbr
}

impl Mesh {
    /// The meshes of the OBJ file at `path`, as [`load_obj`](Mesh::load_obj) loads them, each with
    /// its material from the file's MTL library, mapped as described in [`obj`](crate::obj). A mesh
    /// without a material gets [`PbrMaterial::new`]. Fails if the OBJ file or its MTL library can't
    /// be loaded. Texture maps are resolved against the OBJ file's folder, where tobj looks for the
    /// MTL file too, and load when the model is first drawn.
    pub fn load_obj_with_materials<P: AsRef<Path>>(path: P) -> Result<Vec<(Mesh, PbrMaterial)>, tobj::LoadError> {
        let (models, materials) = tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS)?;
        let folder = path.as_ref().parent().map(Path::to_path_buf).unwrap_or_default();
        let materials = materials?.iter().map(|material| material_from_mtl_in(material, Some(&folder))).collect::<Vec<_>>();

        Ok(models.into_iter().map(|model| {
            let material = model.mesh.material_id
                .and_then(|id| materials.get(id).cloned())
                .unwrap_or_default();
            (Mesh::from(model), material)
        }).collect())
    }
}

/// A model loaded from an OBJ file with its MTL materials: an [`Object`] per mesh, each with its
/// own material. Place it with `.position()` and the like, as any node.
#[derive(Debug, Clone)]
pub struct ObjModel {
    id: NodeId,
    objects: Vec<Object>,
}

impl ObjModel {
    /// The model in the OBJ file at `path`, with the materials of its MTL library. See
    /// [`Mesh::load_obj_with_materials`].
    pub fn load<P: AsRef<Path>>(path: P) -> Result<ObjModel, tobj::LoadError> {
        Ok(ObjModel::new(Mesh::load_obj_with_materials(path)?))
    }

    /// A model of `parts`, each a mesh with its material.
    pub fn new(parts: Vec<(Mesh, PbrMaterial)>) -> ObjModel {
        ObjModel {
            id: NodeId::new(),
            objects: parts.into_iter().map(|(mesh, material)| Object::new(mesh, material)).collect(),
        }
    }
}

impl Render3d for ObjModel {
    fn render(&mut self, context: &mut RenderContext3d) {
        for object in &mut self.objects {
            object.render(context);
        }
    }
}

impl CommonNode3d for ObjModel {
    fn id(&self) -> NodeId {
        self.id
    }

    fn transform(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    fn visible(&self) -> bool {
        true
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d)) {
        for object in &self.objects {
            f(object);
        }
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        for object in &mut self.objects {
            f(object);
        }
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        for object in self.objects.iter_mut().rev() {
            f(object);
        }
    }
}

#[cfg(test)]
mod tests {
    use carbide::color::ColorExt;
    use carbide::state::ReadState;

    use super::*;

    #[test]
    fn map_entries_skip_options_and_keep_spaces() {
        assert_eq!(MtlMap::parse("wood.png").unwrap().file, "wood.png");
        assert_eq!(MtlMap::parse("this diffuse texture has spaces.jpg").unwrap().file, "this diffuse texture has spaces.jpg");

        let map = MtlMap::parse("-bm 0.5 -s 2 3 1 -o 0.25 0 0 -clamp on -mm 0 1 textures/brick.png").unwrap();
        assert_eq!(map.file, "textures/brick.png");
        assert_eq!(map.scale, [2.0, 3.0]);
        assert_eq!(map.offset, [0.25, 0.0]);

        // One value for -s scales u and v alike.
        assert_eq!(MtlMap::parse("-s 4 a.png").unwrap().scale, [4.0, 4.0]);
        assert!(MtlMap::parse("-bm 1.0").is_none());
        assert!(MtlMap::parse("").is_none());
    }

    #[test]
    fn map_scale_and_offset_follow_the_flipped_v() {
        use carbide::math::Vector3;
        let map = MtlMap::parse("-s 2 2 1 -o 0.5 0.25 a.png").unwrap();
        let transform = map.uv_transform();

        // An OBJ coordinate (u, v) is (u, 1 - v) in carbide. MTL maps it to (2u + 0.5, 2v + 0.25)
        // in OBJ terms, which in carbide is (2u + 0.5, 1 - (2v + 0.25)).
        for (u, v) in [(0.0f32, 0.0f32), (0.3, 0.7), (1.0, 1.0)] {
            let carbide = transform * Vector3::new(u, 1.0 - v, 1.0);
            assert!((carbide.x - (2.0 * u + 0.5)).abs() < 1e-5);
            assert!((carbide.y - (1.0 - (2.0 * v + 0.25))).abs() < 1e-5);
        }
    }

    #[test]
    fn mtl_maps_set_the_material_maps() {
        let mut mtl = tobj::Material::default();
        mtl.diffuse_texture = Some("colour.png".to_string());
        mtl.normal_texture = Some("-bm 1.0 normal.png".to_string());
        mtl.unknown_param.insert("map_Pr".to_string(), "rough.png".to_string());
        mtl.unknown_param.insert("map_Ke".to_string(), "glow.png".to_string());

        let folder = Path::new("/models/chair");
        let pbr = material_from_mtl_in(&mtl, Some(folder));
        let image = |name: &str| ImageId::new(folder.join(name));

        let image_of = |texture: Option<&MaterialTexture>| texture.map(|texture| texture.image.clone());
        assert_eq!(image_of(pbr.albedo.value().to_texture()), Some(image("colour.png")));
        assert_eq!(image_of(pbr.normal.value().to_texture()), Some(image("normal.png")));
        assert_eq!(image_of(pbr.aomr_textures.value().to_roughness_texture()), Some(image("rough.png")));
        assert_eq!(*pbr.roughness_factor.value(), None, "a roughness map without Pr is used as it is");
        assert_eq!(image_of(pbr.emissive_texture.value().as_ref()), Some(image("glow.png")));

        // Without the folder, as `material_from_mtl` is called, there are no maps.
        let plain = material_from_mtl(&mtl);
        assert!(plain.albedo.value().to_texture().is_none());
    }

    #[test]
    fn phong_exponents_map_to_roughness() {
        assert!((roughness_from_phong_exponent(0.0) - 1.0).abs() < 1e-6);
        assert!(roughness_from_phong_exponent(96.0) < roughness_from_phong_exponent(64.0));
        assert!(roughness_from_phong_exponent(10_000.0) < 0.15);
    }

    /// Glass as 3ds Max writes it, `illum 4` with `Tf 0.4 0.4 0.4` and `d 1`, is blended at an
    /// opacity of 0.6. The same filter with a model that isn't glass leaves the material opaque,
    /// and a `d` below 1 wins over the filter.
    #[test]
    fn glass_takes_its_opacity_from_the_transmission_filter() {
        let material = |illum: u8, dissolve: f32| {
            let mut mtl = tobj::Material::default();
            mtl.illumination_model = Some(illum);
            mtl.dissolve = Some(dissolve);
            mtl.unknown_param.insert("Tf".to_string(), "0.4 0.4 0.4".to_string());
            material_from_mtl(&mtl)
        };

        let glass = material(4, 1.0);
        assert!((glass.albedo.value().to_value().opacity() - 0.6).abs() < 1e-6);
        assert!(matches!(glass.transparency, Transparency::Blend));

        let plastic = material(2, 1.0);
        assert_eq!(plastic.albedo.value().to_value().opacity(), 1.0);
        assert!(matches!(plastic.transparency, Transparency::Opaque));

        let dissolved = material(4, 0.25);
        assert!((dissolved.albedo.value().to_value().opacity() - 0.25).abs() < 1e-6);
    }

    #[test]
    fn mtl_values_map_to_pbr() {
        let mut mtl = tobj::Material::default();
        mtl.diffuse = Some([0.214, 0.214, 0.214]);
        mtl.specular = Some([0.5, 0.5, 0.5]);
        mtl.shininess = Some(96.0);
        mtl.dissolve = Some(0.5);

        let pbr = material_from_mtl(&mtl);
        let albedo = pbr.albedo.value().to_value();

        // Linear 0.214 is sRGB 0.5.
        assert!((albedo.red() - 0.5).abs() < 0.01, "{albedo:?}");
        assert!((albedo.opacity() - 0.5).abs() < 1e-6);
        assert!((*pbr.reflectance.value() - 0.25).abs() < 1e-6);
        assert!((pbr.roughness_factor.value().unwrap() - roughness_from_phong_exponent(96.0)).abs() < 1e-6);
        assert!(matches!(pbr.transparency, Transparency::Blend));
    }

    #[test]
    fn emission_comes_from_ke() {
        let mut mtl = tobj::Material::default();
        mtl.unknown_param.insert("Ke".into(), "1.0 0.0 0.0".into());

        let emissive = *material_from_mtl(&mtl).emissive.value();
        assert!((emissive.red() - 1.0).abs() < 1e-5 && emissive.green().abs() < 1e-5, "{emissive:?}");
    }

    #[test]
    fn the_pbr_extension_wins() {
        let mut mtl = tobj::Material::default();
        mtl.shininess = Some(96.0);
        mtl.unknown_param.insert("Pr".into(), "0.8".into());
        mtl.unknown_param.insert("Pm".into(), "1.0".into());

        let pbr = material_from_mtl(&mtl);
        assert_eq!(pbr.roughness_factor.value().unwrap(), 0.8);
        assert_eq!(*pbr.metallic.value(), 1.0);
    }
}
