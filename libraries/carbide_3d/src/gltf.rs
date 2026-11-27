//! glTF 2.0 import, through the `gltf` crate.
//!
//! [`load_gltf`] reads a `.gltf` or `.glb` file into a [`GltfScene`]: the node tree, with each
//! node's transform, its meshes and their [`PbrMaterial`]s, and its light. [`GltfModel`] is the
//! scene as a node, for the common case.
//!
//! The mapping from glTF's materials and extensions is described in the design document (Map to
//! glTF). In short:
//! - **Materials:** base colour, metallic and roughness factors and maps, occlusion (with its
//!   strength), normal maps (with their scale), emission, alpha modes, double-sided materials,
//!   and the extensions `KHR_materials_unlit`, `KHR_materials_emissive_strength`,
//!   `KHR_materials_ior` and `KHR_materials_clearcoat` (without its normal map).
//! - **Textures:** every map keeps its own `KHR_texture_transform`, texture coordinate set and
//!   sampler: its wrap modes along u and v, and its filters, smooth unless it asks for nearest
//!   magnification (see `filtering`).
//!   Images in separate files load through their paths, as other images do; images inside the file
//!   (a `.glb`'s buffer views or `data:` URIs) are decoded the first time they are drawn.
//! - **Meshes:** triangles (lists, strips and fans), with normals (flat ones calculated when
//!   missing, as glTF asks), tangents (MikkTSpace ones calculated when a normal map needs them),
//!   two texture coordinate sets and vertex colours.
//! - **Lights:** `KHR_lights_punctual` point, spot and directional lights, placed by their node.
//!
//! What isn't supported yet (cameras, skins, animations, morph targets, other extensions, points
//! and lines) is listed in [`GltfScene::unsupported`] and
//! reported once, when the file is loaded.

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ::gltf::image::Source;
use ::gltf::json::Value;
use ::gltf::khr_lights_punctual::Kind;
use ::gltf::material::AlphaMode;
use ::gltf::mesh::Mode;
use ::gltf::texture::{MagFilter, MinFilter, WrappingMode};
use base64::Engine;
use carbide::color::rgba;
use carbide::draw::{Color, ImageFormat, ImageId};
use carbide::math::{Deg, Matrix4, Point3, Rad, SquareMatrix, Vector2, Vector3, Vector4};

use crate::light::{DirectionalLight, PointLight, SpotLight};
use crate::material::albedo_component::AlbedoComponent;
use crate::material::ao_mr_textures::AoMRTextures;
use crate::material::clearcoat_textures::ClearcoatTextures;
use crate::material::material_texture::MaterialTexture;
use crate::material::normal_texture::NormalTexture;
use crate::material::normal_texture_y_direction::NormalTextureYDirection;
use crate::material::pbr_material::PbrMaterial;
use crate::material::texture_sampler::{Filter, Filtering, Wrap};
use crate::material::transparency::Transparency;
use crate::node3d::{AnyNode3d, CommonNode3d, Node3dExt, NodeId};
use carbide::state::ReadState;
use crate::obj::{color, linear_to_srgb};
use crate::render::{register_encoded_image, Render3d, RenderContext3d};
use crate::{Group, Mesh, Object, Vertex};

/// The extensions read, fully or in part. Others a file uses are reported as unsupported.
const SUPPORTED_EXTENSIONS: [&str; 6] = [
    "KHR_texture_transform",
    "KHR_materials_unlit",
    "KHR_materials_emissive_strength",
    "KHR_materials_ior",
    "KHR_materials_clearcoat",
    "KHR_lights_punctual",
];

/// Why a glTF file could not be loaded.
#[derive(Debug)]
pub enum GltfError {
    /// The file, or a buffer it refers to, could not be read or parsed.
    Gltf(::gltf::Error),
    /// The file has no scene to load.
    NoScene,
    /// The file has no scene with this index.
    NoSceneAt(usize),
}

impl Display for GltfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GltfError::Gltf(error) => write!(f, "{}", error),
            GltfError::NoScene => write!(f, "the file has no scene"),
            GltfError::NoSceneAt(index) => write!(f, "the file has no scene {}", index),
        }
    }
}

impl std::error::Error for GltfError {}

impl From<::gltf::Error> for GltfError {
    fn from(error: ::gltf::Error) -> Self {
        GltfError::Gltf(error)
    }
}

/// A glTF light, as `KHR_lights_punctual` defines it. Colours are carbide's (sRGB), converted from
/// glTF's linear ones; intensities are in glTF's units, which carbide's lights use: candela for
/// point and spot lights, lux for directional ones.
#[derive(Debug, Clone, PartialEq)]
pub enum GltfLight {
    /// Shines along its node's −Z.
    Directional { color: Color, intensity: f32 },
    /// At its node's origin.
    Point { color: Color, intensity: f32, range: Option<f32> },
    /// At its node's origin, shining along its node's −Z.
    Spot { color: Color, intensity: f32, range: Option<f32>, inner: Deg<f32>, outer: Deg<f32> },
}

impl GltfLight {
    /// The light as a carbide light node.
    pub fn node(&self) -> Box<dyn AnyNode3d> {
        match self.clone() {
            GltfLight::Directional { color, intensity } => {
                DirectionalLight::new(Vector3::new(0.0, 0.0, -1.0)).color(color).intensity(intensity).boxed()
            }
            GltfLight::Point { color, intensity, range } => {
                let light = PointLight::new().color(color).intensity(intensity);
                match range {
                    Some(range) => light.range(range).boxed(),
                    None => light.boxed(),
                }
            }
            GltfLight::Spot { color, intensity, range, inner, outer } => {
                let light = SpotLight::new(inner, outer).color(color).intensity(intensity);
                match range {
                    Some(range) => light.range(range).boxed(),
                    None => light.boxed(),
                }
            }
        }
    }
}

/// A node of a glTF scene.
#[derive(Debug, Clone)]
pub struct GltfNode {
    pub name: Option<String>,
    /// The node's transform, relative to its parent.
    pub transform: Matrix4<f32>,
    /// The node's mesh, one part per glTF primitive, each with its material.
    pub primitives: Vec<(Mesh, PbrMaterial)>,
    pub light: Option<GltfLight>,
    pub children: Vec<GltfNode>,
}

impl GltfNode {
    /// The node, its parts, children and (with `lights`) its light, as a carbide node.
    pub fn node(&self, lights: bool) -> Box<dyn AnyNode3d> {
        let mut parts: Vec<Box<dyn AnyNode3d>> = self.primitives.iter()
            .map(|(mesh, material)| Object::new(mesh.clone(), material.clone()).boxed())
            .collect();
        if lights {
            if let Some(light) = &self.light {
                parts.push(light.node());
            }
        }
        parts.extend(self.children.iter().map(|child| child.node(lights)));
        Group::new(parts).transform(self.transform).boxed()
    }

    /// Visit the node and its descendants, with each one's transform relative to the scene.
    fn visit<'a>(&'a self, parent: Matrix4<f32>, f: &mut impl FnMut(&'a GltfNode, Matrix4<f32>)) {
        let transform = parent * self.transform;
        f(self, transform);
        for child in &self.children {
            child.visit(transform, f);
        }
    }
}

/// A glTF scene: its root nodes, and what the file uses that isn't supported.
#[derive(Debug, Clone)]
pub struct GltfScene {
    pub nodes: Vec<GltfNode>,
    /// What the file uses that is left out, each described once, such as `"cameras"` or
    /// `"extension KHR_materials_sheen"`. Reported when the file is loaded.
    pub unsupported: Vec<String>,
}

impl GltfScene {
    /// Visit every node, with its transform relative to the scene.
    pub fn visit<'a>(&'a self, mut f: impl FnMut(&'a GltfNode, Matrix4<f32>)) {
        for node in &self.nodes {
            node.visit(Matrix4::identity(), &mut f);
        }
    }

    /// Every part of every node, with its transform relative to the scene.
    pub fn primitives(&self) -> Vec<(&Mesh, &PbrMaterial, Matrix4<f32>)> {
        let mut primitives = vec![];
        self.visit(|node, transform| {
            for (mesh, material) in &node.primitives {
                primitives.push((mesh, material, transform));
            }
        });
        primitives
    }

    /// The lights, with their transforms relative to the scene.
    pub fn lights(&self) -> Vec<(&GltfLight, Matrix4<f32>)> {
        let mut lights = vec![];
        self.visit(|node, transform| {
            if let Some(light) = &node.light {
                lights.push((light, transform));
            }
        });
        lights
    }

    /// The smallest box around every mesh, in the scene's coordinates: its least and greatest
    /// corners. None for a scene without vertices.
    pub fn bounds(&self) -> Option<(Point3<f32>, Point3<f32>)> {
        let mut bounds: Option<(Point3<f32>, Point3<f32>)> = None;
        for (mesh, _, transform) in self.primitives() {
            for vertex in mesh.vertices() {
                let p = transform * vertex.position.extend(1.0);
                let p = Point3::new(p.x, p.y, p.z);
                bounds = Some(match bounds {
                    None => (p, p),
                    Some((min, max)) => (
                        Point3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z)),
                        Point3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z)),
                    ),
                });
            }
        }
        bounds
    }
}

/// The default scene of the glTF file at `path` (`.gltf` or `.glb`), or its first scene if it
/// names no default. What the file uses that isn't supported is reported once, on standard error,
/// and listed in [`GltfScene::unsupported`].
pub fn load_gltf<P: AsRef<Path>>(path: P) -> Result<GltfScene, GltfError> {
    load(path.as_ref(), None)
}

/// The scene with `index` of the glTF file at `path`, as [`load_gltf`] loads the default one.
pub fn load_gltf_scene<P: AsRef<Path>>(path: P, index: usize) -> Result<GltfScene, GltfError> {
    load(path.as_ref(), Some(index))
}

fn load(path: &Path, index: Option<usize>) -> Result<GltfScene, GltfError> {
    let ::gltf::Gltf { document, blob } = ::gltf::Gltf::open(path)?;
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let buffers = ::gltf::import_buffers(&document, Some(&folder), blob)?;

    let scene = match index {
        Some(index) => document.scenes().nth(index).ok_or(GltfError::NoSceneAt(index))?,
        None => document.default_scene().or_else(|| document.scenes().next()).ok_or(GltfError::NoScene)?,
    };

    let images: Vec<ImageId> = document.images().map(|image| image_id(image, &buffers, &folder)).collect();
    let mut importer = Importer {
        buffers: &buffers,
        textures: document.textures().map(|texture| {
            let sampler = texture.sampler();
            let sampling = Sampling {
                wrap_u: wrap(sampler.wrap_s()),
                wrap_v: wrap(sampler.wrap_t()),
                filtering: filtering(sampler.mag_filter(), sampler.min_filter()),
            };
            (images[texture.source().index()].clone(), sampling)
        }).collect(),
        materials: HashMap::new(),
        unsupported: vec![],
    };

    for extension in document.extensions_used() {
        if !SUPPORTED_EXTENSIONS.contains(&extension) {
            importer.report(format!("extension {}", extension));
        }
    }
    if document.cameras().next().is_some() {
        importer.report("cameras");
    }
    if document.animations().next().is_some() {
        importer.report("animations");
    }
    if document.skins().next().is_some() {
        importer.report("skins");
    }

    let nodes = scene.nodes().map(|node| importer.node(node)).collect();

    if !importer.unsupported.is_empty() {
        eprintln!("carbide_3d: {}: not supported, left out: {}", path.display(), importer.unsupported.join(", "));
    }

    Ok(GltfScene { nodes, unsupported: importer.unsupported })
}

/// The id of a glTF image: its path, for an image in a file of its own, which loads as other
/// images do; otherwise a new in-memory id, with the encoded bytes registered to be decoded when
/// first drawn.
fn image_id(image: ::gltf::Image, buffers: &[::gltf::buffer::Data], folder: &Path) -> ImageId {
    let (bytes, mime) = match image.source() {
        Source::View { view, mime_type } => {
            let buffer = &buffers[view.buffer().index()].0;
            (buffer[view.offset()..view.offset() + view.length()].to_vec(), Some(mime_type))
        }
        Source::Uri { uri, mime_type } => match uri.strip_prefix("data:") {
            Some(data) => {
                let (header, encoded) = data.split_once(',').unwrap_or(("", data));
                let bytes = if header.ends_with(";base64") {
                    base64::engine::general_purpose::STANDARD.decode(encoded).unwrap_or_default()
                } else {
                    encoded.as_bytes().to_vec()
                };
                (bytes, mime_type.or(header.split(';').next()))
            }
            None => return ImageId::new(folder.join(percent_decode(uri))),
        },
    };

    let format = match mime {
        Some("image/png") => ImageFormat::Png,
        Some("image/jpeg") => ImageFormat::Jpeg,
        Some("image/webp") => ImageFormat::WebP,
        _ => ImageFormat::Unknown,
    };
    let id = ImageId::temp(format);
    register_encoded_image(&id, Arc::from(bytes));
    id
}

/// `uri` with its percent-escapes (`%20`) decoded, as glTF's relative URIs are encoded.
fn percent_decode(uri: &str) -> PathBuf {
    let bytes = uri.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(value) = u8::from_str_radix(&uri[i + 1..i + 3], 16) {
                decoded.push(value);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    PathBuf::from(String::from_utf8_lossy(&decoded).into_owned())
}

struct Importer<'a> {
    buffers: &'a [::gltf::buffer::Data],
    /// Each texture's image, and how its sampler samples it, by texture index.
    textures: Vec<(ImageId, Sampling)>,
    /// Each glTF material, converted once, by index (`None` for glTF's default material).
    materials: HashMap<Option<usize>, PbrMaterial>,
    unsupported: Vec<String>,
}

impl Importer<'_> {
    /// Note something left out, once.
    fn report(&mut self, what: impl Into<String>) {
        let what = what.into();
        if !self.unsupported.contains(&what) {
            self.unsupported.push(what);
        }
    }

    fn node(&mut self, node: ::gltf::Node) -> GltfNode {
        let matrix = node.transform().matrix();
        let transform = Matrix4::from(matrix);

        let mut primitives = vec![];
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                if primitive.morph_targets().next().is_some() {
                    self.report("morph targets");
                }
                let material = self.material(primitive.material());
                let normal_mapped = material.normal.value().to_texture().is_some();
                let Some((mesh, has_colors)) = self.mesh(&primitive, normal_mapped) else {
                    continue;
                };
                let material = if has_colors { with_vertex_colors(material) } else { material };
                primitives.push((mesh, material));
            }
        }

        let light = node.light().map(|light| {
            let color = color(light.color(), 1.0);
            let intensity = light.intensity();
            match light.kind() {
                Kind::Directional => GltfLight::Directional { color, intensity },
                Kind::Point => GltfLight::Point { color, intensity, range: light.range() },
                Kind::Spot { inner_cone_angle, outer_cone_angle } => GltfLight::Spot {
                    color,
                    intensity,
                    range: light.range(),
                    inner: Rad(inner_cone_angle).into(),
                    outer: Rad(outer_cone_angle).into(),
                },
            }
        });

        let children = node.children().map(|child| self.node(child)).collect();

        GltfNode { name: node.name().map(str::to_string), transform, primitives, light, children }
    }

    /// The primitive as a mesh, and whether it has vertex colours. None for primitives that aren't
    /// triangles, which are reported.
    fn mesh(&mut self, primitive: &::gltf::Primitive, normal_mapped: bool) -> Option<(Mesh, bool)> {
        let buffers = self.buffers;
        let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| &data.0[..]));

        let positions: Vec<[f32; 3]> = reader.read_positions()?.collect();
        let indices: Vec<u32> = match reader.read_indices() {
            Some(indices) => indices.into_u32().collect(),
            None => (0..positions.len() as u32).collect(),
        };
        let indices = match primitive.mode() {
            Mode::Triangles => indices,
            Mode::TriangleStrip => strip_to_list(&indices),
            Mode::TriangleFan => fan_to_list(&indices),
            mode => {
                self.report(format!("primitive mode {:?}", mode));
                return None;
            }
        };

        let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(Iterator::collect);
        let tangents: Option<Vec<[f32; 4]>> = reader.read_tangents().map(Iterator::collect);
        let uv0: Option<Vec<[f32; 2]>> = reader.read_tex_coords(0).map(|uv| uv.into_f32().collect());
        let uv1: Option<Vec<[f32; 2]>> = reader.read_tex_coords(1).map(|uv| uv.into_f32().collect());
        let colors: Option<Vec<[f32; 4]>> = reader.read_colors(0).map(|colors| colors.into_rgba_f32().collect());

        let vertices: Vec<Vertex> = positions.iter().enumerate().map(|(i, p)| {
            let mut vertex = Vertex::new(Vector3::from(*p));
            if let Some(normals) = &normals {
                vertex.normal = Vector3::from(normals[i]);
            }
            if let Some(tangents) = &tangents {
                vertex.tangent = Vector4::from(tangents[i]);
            }
            if let Some(uv) = &uv0 {
                vertex.texture_coords_0 = Vector2::from(uv[i]);
            }
            if let Some(uv) = &uv1 {
                vertex.texture_coords_1 = Vector2::from(uv[i]);
            }
            if let Some(colors) = &colors {
                // Linear, as glTF's are; the material reads them as linear.
                let [r, g, b, a] = colors[i];
                vertex.color_0 = rgba(r, g, b, a);
            }
            vertex
        }).collect();

        let mesh = match normals {
            Some(_) => Mesh::new(vertices, indices),
            // Without normals, glTF asks for flat ones: a vertex per triangle corner.
            None => {
                let unwelded = indices.iter().map(|index| vertices[*index as usize]).collect();
                Mesh::new(unwelded, (0..indices.len() as u32).collect()).calculate_normals()
            }
        };
        let mesh = if tangents.is_none() && normal_mapped { mesh.calculate_tangents() } else { mesh };

        Some((mesh, colors.is_some()))
    }

    fn material(&mut self, material: ::gltf::Material) -> PbrMaterial {
        let key = material.index();
        if let Some(converted) = self.materials.get(&key) {
            return converted.clone();
        }
        let converted = self.convert_material(&material);
        self.materials.insert(key, converted.clone());
        converted
    }

    fn convert_material(&mut self, material: &::gltf::Material) -> PbrMaterial {
        let pbr = material.pbr_metallic_roughness();
        let [r, g, b, a] = pbr.base_color_factor();
        let base_color = rgba(linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(b), a);

        let base_texture = pbr.base_color_texture().map(|info| self.info_texture(&info));
        let albedo = match base_texture.clone() {
            Some(texture) => AlbedoComponent::TextureValue { texture, value: base_color },
            None => AlbedoComponent::Value(base_color),
        };

        // glTF's metallic and roughness default to 1, unlike carbide's, so both are always set.
        let mut converted = PbrMaterial::new()
            .color(albedo)
            .metallic(pbr.metallic_factor())
            .roughness(Some(pbr.roughness_factor()))
            .double_sided(material.double_sided());

        // Occlusion, roughness and metallic: glTF packs roughness and metallic in green and blue
        // of one map, and occlusion in red of its own, often the same image.
        let metallic_roughness = pbr.metallic_roughness_texture().map(|info| self.info_texture(&info));
        let occlusion = material.occlusion_texture().map(|occlusion| {
            converted = converted.clone().occlusion_strength(occlusion.strength());
            let transform = occlusion.extension_value("KHR_texture_transform").map(TextureTransform::from_json);
            self.texture(occlusion.texture().index(), occlusion.tex_coord(), transform)
        });
        let aomr = match (metallic_roughness, occlusion) {
            (None, None) => AoMRTextures::None,
            (Some(mr), Some(ao)) if mr == ao => AoMRTextures::Combined { texture: Some(mr) },
            (mr_texture, ao_texture) => AoMRTextures::SwizzledSplit { ao_texture, mr_texture },
        };
        converted = converted.aomr_textures(aomr);

        if let Some(normal) = material.normal_texture() {
            let transform = normal.extension_value("KHR_texture_transform").map(TextureTransform::from_json);
            let texture = self.texture(normal.texture().index(), normal.tex_coord(), transform);
            converted = converted
                .normal(NormalTexture::Tricomponent(texture, NormalTextureYDirection::Up))
                .normal_scale(normal.scale());
        }

        converted = converted
            .emissive(color(material.emissive_factor(), 1.0))
            .emissive_strength(material.emissive_strength().unwrap_or(1.0));
        if let Some(info) = material.emissive_texture() {
            let texture = self.info_texture(&info);
            converted = converted.emissive_map(Some(texture));
        }

        converted.transparency = match material.alpha_mode() {
            AlphaMode::Opaque => Transparency::Opaque,
            AlphaMode::Mask => Transparency::Cutout { cutout: material.alpha_cutoff().unwrap_or(0.5) },
            AlphaMode::Blend => Transparency::Blend,
        };

        if material.unlit() {
            converted = converted.unlit();
        }

        // f0 = ((ior − 1) / (ior + 1))², and carbide's reflectance is √(f0 / 0.16): glTF's
        // default of 1.5 is carbide's 0.5.
        if let Some(ior) = material.ior() {
            let f0 = ((ior - 1.0) / (ior + 1.0)).powi(2);
            converted = converted.reflectance((f0 / 0.16).sqrt());
        }

        if let Some(clearcoat) = material.extension_value("KHR_materials_clearcoat") {
            converted = self.clearcoat(converted, clearcoat);
        }

        converted
    }

    /// `KHR_materials_clearcoat`, read from its JSON: `gltf` 1.4.1 doesn't parse it.
    fn clearcoat(&mut self, material: PbrMaterial, clearcoat: &Value) -> PbrMaterial {
        let factor = |key: &str| clearcoat.get(key).and_then(|value| value.as_f64()).unwrap_or(0.0) as f32;
        let strength = self.json_texture(clearcoat.get("clearcoatTexture"));
        let roughness = self.json_texture(clearcoat.get("clearcoatRoughnessTexture"));
        if clearcoat.get("clearcoatNormalTexture").is_some() {
            self.report("clear-coat normal maps");
        }

        let material = material.clearcoat(factor("clearcoatFactor"), factor("clearcoatRoughnessFactor"));
        if strength.is_none() && roughness.is_none() {
            return material;
        }
        material.clearcoat_textures(ClearcoatTextures::GltfSplit { clearcoat_texture: strength, clearcoat_roughness_texture: roughness })
    }

    /// A texture info written as JSON, inside an extension: its texture, coordinate set and
    /// transform.
    fn json_texture(&mut self, info: Option<&Value>) -> Option<MaterialTexture> {
        let info = info?;
        let index = info.get("index")?.as_u64()? as usize;
        let tex_coord = info.get("texCoord").and_then(Value::as_u64).unwrap_or(0) as u32;
        let transform = info.get("extensions")
            .and_then(|extensions| extensions.get("KHR_texture_transform"))
            .map(TextureTransform::from_json);
        Some(self.texture(index, tex_coord, transform))
    }

    /// A texture info's texture, coordinate set and transform.
    fn info_texture(&mut self, info: &::gltf::texture::Info) -> MaterialTexture {
        let transform = info.texture_transform().map(|transform| TextureTransform {
            offset: Vector2::from(transform.offset()),
            rotation: transform.rotation(),
            scale: Vector2::from(transform.scale()),
            tex_coord: transform.tex_coord(),
        });
        self.texture(info.texture().index(), info.tex_coord(), transform)
    }

    /// The texture with `index`, read with the coordinate set `tex_coord` and `transform`.
    fn texture(&mut self, index: usize, tex_coord: u32, transform: Option<TextureTransform>) -> MaterialTexture {
        let (image, sampling) = self.textures[index].clone();

        let mut texture = MaterialTexture::new(image).wrap_uv(sampling.wrap_u, sampling.wrap_v);
        texture.filtering = sampling.filtering;
        let mut tex_coord = tex_coord;
        if let Some(transform) = transform {
            texture = texture.offset_rotation_scale(transform.offset, Rad(transform.rotation), transform.scale);
            tex_coord = transform.tex_coord.unwrap_or(tex_coord);
        }

        if tex_coord > 1 {
            self.report("texture coordinate sets above 1");
            tex_coord = 0;
        }
        texture.uv_set(tex_coord as u8)
    }
}

/// A texture's sampler: its wrap modes, and its filters, `None` when the file sets none, leaving
/// the material's default (linear).
#[derive(Debug, Copy, Clone)]
struct Sampling {
    wrap_u: Wrap,
    wrap_v: Wrap,
    filtering: Option<Filtering>,
}

fn wrap(mode: WrappingMode) -> Wrap {
    match mode {
        WrappingMode::Repeat => Wrap::Repeat,
        WrappingMode::ClampToEdge => Wrap::ClampToEdge,
        WrappingMode::MirroredRepeat => Wrap::MirroredRepeat,
    }
}

/// A sampler's filters, with a floor of smooth filtering. glTF leaves both optional; both left out
/// leave the material's default. A nearest magnification filter, which asks for sharp texels up
/// close as pixel art does, keeps the file's filters as they are (a minification filter without a
/// mipmap part, `NEAREST` or `LINEAR`, reads only the full-size image). Otherwise the map is
/// linear throughout, with linear mipmaps (and anisotropic filtering): files often ask for
/// `NEAREST_MIPMAP_LINEAR` or an unmipmapped `LINEAR` from afar, as most of Khronos's samples do,
/// which aliases minified detail and text, and is unlikely to be meant.
fn filtering(mag: Option<MagFilter>, min: Option<MinFilter>) -> Option<Filtering> {
    match (mag, min) {
        (None, None) => None,
        (Some(MagFilter::Nearest), min) => {
            let (min, mipmap) = match min {
                Some(MinFilter::Nearest) => (Filter::Nearest, None),
                Some(MinFilter::Linear) => (Filter::Linear, None),
                Some(MinFilter::NearestMipmapNearest) => (Filter::Nearest, Some(Filter::Nearest)),
                Some(MinFilter::LinearMipmapNearest) => (Filter::Linear, Some(Filter::Nearest)),
                Some(MinFilter::NearestMipmapLinear) => (Filter::Nearest, Some(Filter::Linear)),
                Some(MinFilter::LinearMipmapLinear) | None => (Filter::Linear, Some(Filter::Linear)),
            };
            Some(Filtering { mag: Filter::Nearest, min, mipmap })
        }
        _ => Some(Filtering::LINEAR),
    }
}

/// A `KHR_texture_transform`.
struct TextureTransform {
    offset: Vector2<f32>,
    rotation: f32,
    scale: Vector2<f32>,
    /// Overrides the texture info's coordinate set.
    tex_coord: Option<u32>,
}

impl TextureTransform {
    /// The extension as JSON, with glTF's defaults for what it leaves out.
    fn from_json(json: &Value) -> TextureTransform {
        let pair = |key: &str, default: f32| -> Vector2<f32> {
            let values = json.get(key).and_then(Value::as_array);
            let at = |i: usize| values.and_then(|values| values.get(i)).and_then(Value::as_f64).map_or(default, |value| value as f32);
            Vector2::new(at(0), at(1))
        };
        TextureTransform {
            offset: pair("offset", 0.0),
            rotation: json.get("rotation").and_then(Value::as_f64).unwrap_or(0.0) as f32,
            scale: pair("scale", 1.0),
            tex_coord: json.get("texCoord").and_then(Value::as_u64).map(|value| value as u32),
        }
    }
}

/// `material` with its albedo multiplied by the vertex colours, read as linear, as glTF's are.
fn with_vertex_colors(material: PbrMaterial) -> PbrMaterial {
    let albedo = match material.albedo.value().clone() {
        AlbedoComponent::None => AlbedoComponent::Vertex { srgb: false },
        AlbedoComponent::Value(value) => AlbedoComponent::ValueVertex { value, srgb: false },
        AlbedoComponent::Texture(texture) => AlbedoComponent::TextureVertex { texture, srgb: false },
        AlbedoComponent::TextureValue { texture, value } => AlbedoComponent::TextureVertexValue { texture, srgb: false, value },
        other => other,
    };
    material.color(albedo)
}

/// A triangle strip's indices as a list, every other triangle turned so all face the same way.
fn strip_to_list(strip: &[u32]) -> Vec<u32> {
    (2..strip.len()).flat_map(|i| {
        if i % 2 == 0 { [strip[i - 2], strip[i - 1], strip[i]] } else { [strip[i - 1], strip[i - 2], strip[i]] }
    }).collect()
}

/// A triangle fan's indices as a list.
fn fan_to_list(fan: &[u32]) -> Vec<u32> {
    (2..fan.len()).flat_map(|i| [fan[0], fan[i - 1], fan[i]]).collect()
}

/// A model loaded from a glTF file: its scene as nodes, a [`Group`] per glTF node with the node's
/// transform, an [`Object`] per primitive, and its lights. Place it with `.position()` and the
/// like, as any node.
#[derive(Debug, Clone)]
pub struct GltfModel {
    id: NodeId,
    scene: GltfScene,
    lights: bool,
    nodes: Vec<Box<dyn AnyNode3d>>,
}

impl GltfModel {
    /// The default scene of the glTF file at `path`. See [`load_gltf`].
    pub fn load<P: AsRef<Path>>(path: P) -> Result<GltfModel, GltfError> {
        Ok(GltfModel::new(load_gltf(path)?))
    }

    /// The scene with `index` of the glTF file at `path`.
    pub fn load_scene<P: AsRef<Path>>(path: P, index: usize) -> Result<GltfModel, GltfError> {
        Ok(GltfModel::new(load_gltf_scene(path, index)?))
    }

    /// A model of `scene`, with its lights.
    pub fn new(scene: GltfScene) -> GltfModel {
        let nodes = scene.nodes.iter().map(|node| node.node(true)).collect();
        GltfModel { id: NodeId::new(), scene, lights: true, nodes }
    }

    /// Whether the file's lights light the scene: on by default. Apps that light their scenes
    /// themselves often turn them off.
    pub fn lights(mut self, lights: bool) -> GltfModel {
        self.lights = lights;
        self.nodes = self.scene.nodes.iter().map(|node| node.node(lights)).collect();
        self
    }

    /// The scene the model shows.
    pub fn scene(&self) -> &GltfScene {
        &self.scene
    }
}

impl Render3d for GltfModel {
    fn render(&mut self, context: &mut RenderContext3d) {
        for node in &mut self.nodes {
            node.render(context);
        }
    }
}

impl CommonNode3d for GltfModel {
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
        for node in &self.nodes {
            f(node.as_ref());
        }
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        for node in &mut self.nodes {
            f(node.as_mut());
        }
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        for node in self.nodes.iter_mut().rev() {
            f(node.as_mut());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_and_fans_become_lists_facing_one_way() {
        assert_eq!(strip_to_list(&[0, 1, 2, 3, 4]), vec![0, 1, 2, 2, 1, 3, 2, 3, 4]);
        assert_eq!(fan_to_list(&[0, 1, 2, 3]), vec![0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn uris_are_percent_decoded() {
        assert_eq!(percent_decode("my%20texture.png"), PathBuf::from("my texture.png"));
        assert_eq!(percent_decode("plain.png"), PathBuf::from("plain.png"));
        assert_eq!(percent_decode("bad%2"), PathBuf::from("bad%2"));
    }
}

#[cfg(test)]
mod filtering_tests {
    use ::gltf::texture::{MagFilter, MinFilter};

    use super::filtering;
    use crate::material::texture_sampler::{Filter, Filtering};

    /// A sampler that sets no filters leaves the material's default.
    #[test]
    fn no_filters_leave_the_default() {
        assert_eq!(filtering(None, None), None);
    }

    /// Linear or unset magnification: linear throughout, whatever minification asks for.
    #[test]
    fn linear_magnification_is_smooth_throughout() {
        for min in [None, Some(MinFilter::Nearest), Some(MinFilter::Linear), Some(MinFilter::NearestMipmapLinear), Some(MinFilter::LinearMipmapNearest)] {
            assert_eq!(filtering(Some(MagFilter::Linear), min), Some(Filtering::LINEAR), "{min:?}");
        }
        assert_eq!(filtering(None, Some(MinFilter::Nearest)), Some(Filtering::LINEAR));
    }

    /// Nearest magnification, as pixel art asks for: the file's filters as they are, without
    /// mipmaps for a minification filter without a mipmap part.
    #[test]
    fn nearest_magnification_keeps_the_files_filters() {
        assert_eq!(filtering(Some(MagFilter::Nearest), Some(MinFilter::Nearest)), Some(Filtering { mag: Filter::Nearest, min: Filter::Nearest, mipmap: None }));
        assert_eq!(filtering(Some(MagFilter::Nearest), Some(MinFilter::LinearMipmapNearest)), Some(Filtering { mag: Filter::Nearest, min: Filter::Linear, mipmap: Some(Filter::Nearest) }));
        assert_eq!(filtering(Some(MagFilter::Nearest), None), Some(Filtering { mag: Filter::Nearest, min: Filter::Linear, mipmap: Some(Filter::Linear) }));
    }
}
