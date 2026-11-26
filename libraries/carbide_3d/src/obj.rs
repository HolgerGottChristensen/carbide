//! OBJ models with their MTL materials.
//!
//! MTL describes Phong materials; carbide_3d's are physically based. [`material_from_mtl`] maps
//! one onto the other:
//! - **Colour:** `Kd`, the diffuse colour, with `d`, the dissolve, as its opacity. Colours in MTL
//!   files are linear, as Blender and most renderers read them, so they are converted to sRGB,
//!   as carbide's colours are.
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
//! Not used: `Ka`, since ambient light is the scene's; `Tr`, which exporters disagree on (some
//! write transparency, others opacity); `Ni`; and textures, which carbide_3d doesn't sample yet.
//! An opacity below 1 sets [`Transparency::Blend`], which the renderer doesn't blend yet.

use std::path::Path;

use carbide::color::{rgba, Color};
use carbide::math::{Matrix4, SquareMatrix};

use crate::material::pbr_material::PbrMaterial;
use crate::material::transparency::Transparency;
use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render::{Render3d, RenderContext3d};
use crate::{Mesh, Object};

/// A linear colour component converted to sRGB, with the exact sRGB curve.
fn linear_to_srgb(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    if value <= 0.0031308 { value * 12.92 } else { 1.055 * value.powf(1.0 / 2.4) - 0.055 }
}

/// A linear MTL colour as a carbide colour, with `alpha`.
fn color(linear: [f32; 3], alpha: f32) -> Color {
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

/// The roughness whose GGX highlight is about as wide as a Blinn-Phong highlight of `exponent`.
pub fn roughness_from_phong_exponent(exponent: f32) -> f32 {
    let alpha = (2.0 / (exponent.max(0.0) + 2.0)).sqrt();
    alpha.sqrt().clamp(0.0, 1.0)
}

/// The physically based material closest to an MTL material. See the [module](self) for how each
/// value is mapped.
pub fn material_from_mtl(material: &tobj::Material) -> PbrMaterial {
    let opacity = material.dissolve.unwrap_or(1.0);
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

    if opacity < 1.0 {
        pbr.transparency = Transparency::Blend;
    }

    if material.illumination_model == Some(0) {
        pbr = pbr.unlit();
    }

    pbr
}

impl Mesh {
    /// The meshes of the OBJ file at `path`, as [`load_obj`](Mesh::load_obj) loads them, each with
    /// its material from the file's MTL library, mapped as described in [`obj`](crate::obj). A mesh
    /// without a material gets [`PbrMaterial::new`]. Fails if the OBJ file or its MTL library can't
    /// be loaded. Textures are not loaded.
    pub fn load_obj_with_materials<P: AsRef<Path>>(path: P) -> Result<Vec<(Mesh, PbrMaterial)>, tobj::LoadError> {
        let (models, materials) = tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS)?;
        let materials = materials?.iter().map(material_from_mtl).collect::<Vec<_>>();

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
    fn phong_exponents_map_to_roughness() {
        assert!((roughness_from_phong_exponent(0.0) - 1.0).abs() < 1e-6);
        assert!(roughness_from_phong_exponent(96.0) < roughness_from_phong_exponent(64.0));
        assert!(roughness_from_phong_exponent(10_000.0) < 0.15);
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
