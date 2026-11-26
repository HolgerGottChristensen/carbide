//! The playground: the scene, shown in one or two views, with its options in a side bar.

use carbide::color::{rgba_bytes, Color, BLACK, BLUE, GREEN, ORANGE, RED, TRANSPARENT, WHITE};
use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::picker::{MenuStyle, Picker};
use carbide::controls::slider::Slider;
use carbide::controls::toggle::{CheckboxStyle, Toggle};
use carbide::controls::ControlsExt;
use carbide::d3::camera::{look_at, Camera, OrbitController, Projection};
use carbide::d3::light::{DirectionalLight, PointLight, SpotLight};
use carbide::d3::material::albedo_component::AlbedoComponent;
use carbide::d3::material::pbr_material::PbrMaterial;
use carbide::d3::node3d::{AnyNode3d, CommonNode3d, NodeId};
use carbide::d3::render::{Render3d, RenderContext3d};
use std::rc::Rc;
use carbide::d3::render::{ContextFactory3d, NoopRenderContext3d, RenderContext3dId, Tonemapping};
use carbide::d3::wgpu::WgpuRenderContext3d;
use carbide::d3::{PickHit, AxisHelper, Group, Mesh, Node3dExt, Node3dSequence, Object, Rotation, Scene3d, ShadowsExt};
use carbide::draw::{resolve_asset_path, ImageId, Scalar};
use carbide::d3::material::ao_mr_textures::AoMRTextures;
use carbide::d3::material::normal_texture::NormalTexture;
use carbide::d3::material::sample_type::SampleType;
use carbide::d3::material::normal_texture_y_direction::NormalTextureYDirection;
use std::path::{Path, PathBuf};
use carbide::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide::math::{Deg, Euler, Matrix4, Point3, Quaternion, SquareMatrix, Vector2, Vector3};
use carbide::state::{AnimatedState, LocalState, Map1, Map2, Map3, Map6, ReadState, State};
use carbide::time::Duration;
use carbide::ui;
use carbide::widget::*;

use crate::meshes;

/// What covers the model: its colour alone, an image as its colour, or a set of material maps.
/// The choices are the images listed in `surfaces`, and every folder in `assets/materials`.
#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    pub name: String,
    pub albedo: Option<ImageId>,
    /// The normal map, and whether its green channel points up (OpenGL) or down (DirectX).
    pub normal: Option<(ImageId, NormalTextureYDirection)>,
    pub ambient_occlusion: Option<ImageId>,
    pub roughness: Option<ImageId>,
    pub metallic: Option<ImageId>,
    /// The light the surface gives off, as an image.
    pub emission: Option<ImageId>,
}

impl Surface {
    /// No texture: the side bar's colour.
    pub fn none() -> Surface {
        Surface { name: "None".to_string(), albedo: None, normal: None, ambient_occlusion: None, roughness: None, metallic: None, emission: None }
    }

    /// An image as the colour, from `assets/images`.
    pub fn image(name: &str, path: &'static str) -> Surface {
        Surface { name: name.to_string(), albedo: Some(ImageId::new(path)), ..Surface::none() }
    }

    /// Whether it has maps besides the colour.
    pub fn has_maps(&self) -> bool {
        self.normal.is_some() || self.ambient_occlusion.is_some() || self.roughness.is_some() || self.metallic.is_some() || self.emission.is_some()
    }

    fn aomr(&self) -> AoMRTextures {
        if self.ambient_occlusion.is_none() && self.roughness.is_none() && self.metallic.is_none() {
            return AoMRTextures::None;
        }

        // Separate greyscale maps, each in its red channel.
        AoMRTextures::BWSplit {
            ao_texture: self.ambient_occlusion.clone(),
            m_texture: self.metallic.clone(),
            r_texture: self.roughness.clone(),
        }
    }

    fn normal_texture(&self) -> NormalTexture {
        match &self.normal {
            Some((id, direction)) => NormalTexture::Tricomponent(id.clone(), *direction),
            None => NormalTexture::None,
        }
    }
}

/// The surfaces to pick from: none, a few images, then one per folder in `assets/materials`, read
/// at startup, so adding a folder adds a choice.
pub fn surfaces() -> Vec<Surface> {
    let mut surfaces = vec![
        Surface::none(),
        Surface::image("Landscape", "images/landscape.png"),
        Surface::image("Pixel grid", "images/pixelgrid.png"),
        Surface::image("Rust logo", "images/rust.png"),
    ];
    surfaces.extend(material_folders());
    surfaces
}

/// A surface for each folder in `assets/materials`, by name.
fn material_folders() -> Vec<Surface> {
    let Ok(folder) = resolve_asset_path(Path::new("materials")) else {
        return vec![];
    };
    let Ok(entries) = std::fs::read_dir(&folder) else {
        return vec![];
    };

    let mut folders: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).filter(|path| path.is_dir()).collect();
    folders.sort();

    folders.iter().filter_map(|folder| material_folder(folder)).collect()
}

/// The maps in one material folder, recognised by words in their file names after the first (the
/// material's own name), as material sites name them: `color`, `albedo`, `basecolor` or `diffuse`; `normal`, with `dx` for DirectX normals;
/// `ambientocclusion`, `occlusion` or `ao`; `roughness`; `metal`; and `emission` or `emissive`. Other files, such as height
/// maps, are left out. A folder with no recognised map gives no surface.
pub(crate) fn material_folder(folder: &Path) -> Option<Surface> {
    let name = folder.file_name()?.to_string_lossy().to_string();
    let mut surface = Surface { name: capitalised(&name), ..Surface::none() };

    let mut files: Vec<PathBuf> = std::fs::read_dir(folder).ok()?.flatten().map(|entry| entry.path()).collect();
    files.sort();

    for file in files {
        let extension = file.extension().map(|ext| ext.to_string_lossy().to_lowercase()).unwrap_or_default();
        if !matches!(extension.as_str(), "png" | "jpg" | "jpeg") {
            continue;
        }

        // The words of the name, without the first when there are more: material sites start the
        // name with the material's own, which may contain a map's word itself (`Metal012`).
        let stem = file.file_stem().map(|stem| stem.to_string_lossy().to_lowercase()).unwrap_or_default();
        let all_words: Vec<&str> = stem.split(|c: char| !c.is_ascii_alphanumeric()).filter(|word| !word.is_empty()).collect();
        let words = if all_words.len() > 1 { &all_words[1..] } else { &all_words[..] };
        let has = |test: &dyn Fn(&str) -> bool| words.iter().any(|word| test(word));
        let id = ImageId::new(file.clone());

        if has(&|word| word.starts_with("normal")) {
            let directx = has(&|word| word == "dx" || (word.starts_with("normal") && word.ends_with("dx")));
            let direction = if directx { NormalTextureYDirection::Down } else { NormalTextureYDirection::Up };
            surface.normal = Some((id, direction));
        } else if has(&|word| word.contains("roughness")) {
            surface.roughness = Some(id);
        } else if has(&|word| word.contains("occlusion") || word == "ao") {
            surface.ambient_occlusion = Some(id);
        } else if has(&|word| word.starts_with("emissi")) {
            surface.emission = Some(id);
        } else if has(&|word| word.starts_with("metal")) {
            surface.metallic = Some(id);
        } else if has(&|word| word.contains("color") || word.contains("albedo") || word.contains("diffuse")) {
            surface.albedo = Some(id);
        }
    }

    (surface.albedo.is_some() || surface.has_maps()).then_some(surface)
}

fn capitalised(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The models to show.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Model {
    /// No model: only the lights, and the axes when shown.
    Empty,
    Cube,
    Sphere,
    Cubes,
    /// The OBJ models in `libraries/carbide_3d/ressources`, with their MTL materials.
    RenderBall,
    /// A wolf, with colour, metallic, roughness and normal maps (CC0, Grab3D; AI-assisted).
    Wolf,
    /// Yasutoshi Mori's material test object (CC BY 4.0).
    MoriKnob,
    /// A sports car with many materials, glowing lights and glass.
    SportsCar,
    /// Five by five spheres: roughness from 0 to 1 left to right, metallic from 0 to 1 top to
    /// bottom, as in the usual PBR material charts.
    MaterialGrid,
}

impl Model {
    /// The OBJ file of a model loaded from one, relative to the workspace.
    pub fn obj_file(&self) -> Option<&'static str> {
        macro_rules! ressource {
            ($file:literal) => { concat!(env!("CARGO_MANIFEST_DIR"), "/../../libraries/carbide_3d/ressources/", $file) };
        }
        match self {
            Model::RenderBall => Some(ressource!("renderball.obj")),
            Model::Wolf => Some(ressource!("wolf/wolf.obj")),
            Model::MoriKnob => Some(ressource!("mori_knob/testObj.obj")),
            Model::SportsCar => Some(ressource!("sportscar/sportsCar.obj")),
            _ => None,
        }
    }

    /// The model, loaded and fitted to the view. Mori's knob sits in a studio scene, with a large
    /// backdrop and a light panel, which the fit leaves out.
    pub fn load(&self) -> Option<Rc<meshes::FittedObj>> {
        self.obj_file().map(|file| meshes::load_obj(file, *self == Model::MoriKnob))
    }
}

/// The steps of the material grid, for both roughness and metallic.
const GRID_STEPS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// How far apart the material grid's spheres are, centre to centre, in metres.
const GRID_SPACING: f32 = 0.28;

/// The radius of the material grid's spheres, in metres.
const GRID_RADIUS: f32 = 0.1;

/// How many views of the scene to show.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Views {
    One,
    /// The orbiting view, and a front orthographic view of the same scene beside it.
    Two,
}

/// The kind of projection of the main view.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProjectionKind {
    Perspective,
    Orthographic,
}

/// The kind of light lighting the model.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LightKind {
    Directional,
    /// A point light 2.5 metres from the model.
    Point,
    /// A spot light 2.5 metres from the model, aimed at it.
    Spot,
}

/// The colour of the light.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LightColor {
    White,
    /// A warm white, as an incandescent bulb: 2700 K.
    Warm,
    /// A cool white, as an overcast sky: 8000 K.
    Cool,
    Red,
    Green,
    Blue,
    /// A black body's colour at the side bar's temperature.
    Temperature,
}

/// How far point and spot lights are from the model, in metres.
const LIGHT_DISTANCE: f32 = 2.5;

/// The 3D renderer the views render with.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Renderer {
    Wgpu,
    /// No renderer: the views render nothing.
    None,
}

/// Every option of the playground, as states the scene reads.
#[derive(Clone)]
pub struct Options {
    pub model: LocalState<Model>,
    pub views: LocalState<Views>,
    pub renderer: LocalState<Renderer>,
    /// Whether the axes are drawn at the origin.
    pub axes: LocalState<bool>,
    /// Whether a floor is shown under the model, to catch its shadow.
    pub floor: LocalState<bool>,
    /// Whether the views respond to the pointer and keyboard: orbiting, hover and clicks.
    pub interactive: LocalState<bool>,
    /// Whether the model is selected, and so outlined: toggled by clicking it.
    pub outlined: LocalState<bool>,
    /// Whether the model reacts to the pointer over it: a thin outline, and the side bar's read-out.
    /// Off, the model has no hover handler, so the views don't pick as the pointer moves.
    pub hover: LocalState<bool>,
    /// Whether the pointer is over the model.
    pub hovered: LocalState<bool>,
    /// Where the last click hit the model, in world space.
    pub last_hit: LocalState<Option<Vector3<f32>>>,
    /// The outline's thickness, in logical pixels.
    pub outline_thickness: LocalState<Scalar>,

    pub color: LocalState<Color>,
    /// What covers the model: the colour, an image in its place, or a set of material maps.
    pub surface: LocalState<Surface>,
    /// The surfaces to pick from, read at startup (`surfaces`).
    pub surfaces: Vec<Surface>,
    /// Whether each of the surface's maps is used, to compare the model with and without it.
    pub use_color_map: LocalState<bool>,
    pub use_normal_map: LocalState<bool>,
    pub use_roughness_map: LocalState<bool>,
    pub use_occlusion_map: LocalState<bool>,
    pub use_metallic_map: LocalState<bool>,
    pub use_emission_map: LocalState<bool>,
    /// How many times the surface's maps repeat across each side or around the model.
    pub tiling: LocalState<Scalar>,
    /// Whether the maps are sampled smoothly or show their pixels.
    pub sampling: LocalState<SampleType>,
    pub roughness: LocalState<Scalar>,
    pub metallic: LocalState<Scalar>,
    pub reflectance: LocalState<Scalar>,
    pub clearcoat: LocalState<Scalar>,
    pub clearcoat_roughness: LocalState<Scalar>,
    pub ambient_occlusion: LocalState<Scalar>,
    /// The colour the model glows in; black for none.
    pub emissive: LocalState<Color>,
    pub emissive_strength: LocalState<Scalar>,
    /// Whether the model's colour is multiplied by its vertex colours, which paint it by position.
    pub vertex_colors: LocalState<bool>,
    /// Whether the render ball has the materials of its MTL file, instead of the side bar's.
    pub file_materials: LocalState<bool>,

    /// Whether the model spins on its own, or is turned by `turn`.
    pub spin: LocalState<bool>,
    /// How far the model is turned around the vertical axis, in degrees, while it isn't spinning.
    pub turn: LocalState<Scalar>,

    pub light_azimuth: LocalState<Scalar>,
    pub light_elevation: LocalState<Scalar>,
    pub light_intensity: LocalState<Scalar>,
    pub light_kind: LocalState<LightKind>,
    /// The spot light's outer cone angle, in degrees; the inner angle is 60% of it.
    pub spot_angle: LocalState<Scalar>,
    pub light_color: LocalState<LightColor>,
    /// The light's colour temperature in kelvin, used when the colour is `Temperature`.
    pub light_temperature: LocalState<Scalar>,
    /// Whether the directional and spot lights cast shadows.
    pub shadows: LocalState<bool>,

    /// The views' background.
    pub background: LocalState<Color>,
    /// The hemisphere's colour from below; the sky is white.
    pub ground: LocalState<Color>,
    pub ambient: LocalState<Scalar>,
    pub exposure: LocalState<Scalar>,
    pub tonemapping: LocalState<Tonemapping>,

    pub projection: LocalState<ProjectionKind>,
    pub fov: LocalState<Scalar>,
    pub height: LocalState<Scalar>,
    /// The main view's view matrix, moved by its orbit controller and reset from the side bar.
    pub view: LocalState<Matrix4<f32>>,
}

impl Options {
    pub fn new() -> Options {
        Options {
            model: LocalState::new(Model::Cube),
            views: LocalState::new(Views::One),
            renderer: LocalState::new(Renderer::Wgpu),
            axes: LocalState::new(false),
            floor: LocalState::new(false),
            interactive: LocalState::new(true),
            outlined: LocalState::new(false),
            hover: LocalState::new(true),
            hovered: LocalState::new(false),
            last_hit: LocalState::new(None),
            outline_thickness: LocalState::new(2.0),
            color: LocalState::new(RED),
            surface: LocalState::new(Surface::none()),
            surfaces: surfaces(),
            use_color_map: LocalState::new(true),
            use_normal_map: LocalState::new(true),
            use_roughness_map: LocalState::new(true),
            use_occlusion_map: LocalState::new(true),
            use_metallic_map: LocalState::new(true),
            use_emission_map: LocalState::new(true),
            tiling: LocalState::new(1.0),
            sampling: LocalState::new(SampleType::Linear),
            roughness: LocalState::new(0.5),
            metallic: LocalState::new(0.0),
            reflectance: LocalState::new(0.5),
            clearcoat: LocalState::new(0.0),
            clearcoat_roughness: LocalState::new(0.1),
            ambient_occlusion: LocalState::new(1.0),
            emissive: LocalState::new(BLACK),
            emissive_strength: LocalState::new(1.0),
            vertex_colors: LocalState::new(false),
            file_materials: LocalState::new(true),
            spin: LocalState::new(false),
            turn: LocalState::new(0.0),
            light_azimuth: LocalState::new(-35.0),
            light_elevation: LocalState::new(50.0),
            light_intensity: LocalState::new(5.0),
            light_kind: LocalState::new(LightKind::Directional),
            spot_angle: LocalState::new(20.0),
            shadows: LocalState::new(true),
            light_color: LocalState::new(LightColor::White),
            light_temperature: LocalState::new(6500.0),
            background: LocalState::new(TRANSPARENT),
            ground: LocalState::new(WHITE),
            ambient: LocalState::new(0.1),
            exposure: LocalState::new(1.0),
            tonemapping: LocalState::new(Tonemapping::PbrNeutral),
            projection: LocalState::new(ProjectionKind::Perspective),
            fov: LocalState::new(60.0),
            height: LocalState::new(2.0),
            view: LocalState::new(default_view()),
        }
    }

    /// The main view's camera. Its view matrix is the shared `view` state. Its projection is made
    /// from the side bar's projection, field of view and height, and writes back into them: a pinch
    /// on an orthographic view changes the height, and the height slider follows.
    fn main_camera(&self) -> Camera {
        let projection = Map3::map(
            self.projection.clone(), self.fov.clone(), self.height.clone(),
            |kind: &ProjectionKind, fov: &Scalar, height: &Scalar| match kind {
                ProjectionKind::Perspective => Projection::Perspective { vfov: *fov as f32, near: 0.05 },
                ProjectionKind::Orthographic => Projection::Orthographic { height: *height as f32 },
            },
            |projection: Projection, mut kind, mut fov, mut height| match projection {
                Projection::Perspective { vfov, .. } => {
                    *kind = ProjectionKind::Perspective;
                    *fov = vfov as Scalar;
                }
                Projection::Orthographic { height: new_height } => {
                    *kind = ProjectionKind::Orthographic;
                    *height = new_height as Scalar;
                }
                Projection::Raw(_) => {}
            },
        );

        Camera::perspective().projection(projection).view(self.view.clone())
    }

    /// Where the light comes from, as a unit vector from the model, from the side bar's angles:
    /// an azimuth of 0° is from in front of the scene (the camera's side at the start, +Z),
    /// positive to the right, and the height is degrees above the horizon.
    fn light_from(&self) -> impl ReadState<T=Vector3<f32>> + Clone + use<> {
        Map2::read_map(self.light_azimuth.clone(), self.light_elevation.clone(), |azimuth: &Scalar, elevation: &Scalar| {
            let (azimuth, elevation) = ((*azimuth as f32).to_radians(), (*elevation as f32).to_radians());
            Vector3::new(elevation.cos() * azimuth.sin(), elevation.sin(), elevation.cos() * azimuth.cos())
        })
    }

    /// The side bar's intensity for a light of `kind`, and none for the others. The side bar
    /// gives the illuminance at the model in lux, so point and spot lights get it times the
    /// distance squared, in candela: switching kinds keeps the model as bright.
    fn intensity_for(&self, kind: LightKind) -> impl ReadState<T=f32> + Clone + use<> {
        Map2::read_map(self.light_kind.clone(), self.light_intensity.clone(), move |current: &LightKind, lux: &Scalar| {
            match (*current == kind, kind) {
                (false, _) => 0.0,
                (true, LightKind::Directional) => *lux as f32,
                (true, _) => *lux as f32 * LIGHT_DISTANCE * LIGHT_DISTANCE,
            }
        })
    }

    /// The side bar's light colour: a fixed colour, or one from the temperature.
    fn light_color(&self) -> impl ReadState<T=Color> + Clone + use<> {
        Map2::read_map(self.light_color.clone(), self.light_temperature.clone(), |color: &LightColor, kelvin: &Scalar| {
            match color {
                LightColor::White => WHITE,
                LightColor::Warm => Color::temperature(2700.0),
                LightColor::Cool => Color::temperature(8000.0),
                LightColor::Red => RED,
                LightColor::Green => GREEN,
                LightColor::Blue => BLUE,
                LightColor::Temperature => Color::temperature(*kelvin),
            }
        })
    }

    /// The three lights, of which only the side bar's kind shines.
    fn lights(&self) -> Vec<Box<dyn AnyNode3d>> {
        let from = self.light_from();
        let position = Map1::read_map(from.clone(), |from: &Vector3<f32>| from * LIGHT_DISTANCE);
        // The spot light shines along its -Z: turn -Z to point back at the model.
        let aim = Map1::read_map(from.clone(), |from: &Vector3<f32>| Rotation(Quaternion::from_arc(-Vector3::unit_z(), -*from, None)));
        let outer = Map1::read_map(self.spot_angle.clone(), |angle: &Scalar| Deg(*angle as f32));
        let inner = Map1::read_map(self.spot_angle.clone(), |angle: &Scalar| Deg(*angle as f32 * 0.6));

        vec![
            DirectionalLight::new(Map1::read_map(from, |from: &Vector3<f32>| -*from))
                .color(self.light_color())
                .intensity(self.intensity_for(LightKind::Directional))
                .boxed(),
            PointLight::new()
                .color(self.light_color())
                .intensity(self.intensity_for(LightKind::Point))
                .position(position.clone())
                .boxed(),
            SpotLight::new(inner, outer)
                .color(self.light_color())
                .intensity(self.intensity_for(LightKind::Spot))
                .rotation(aim)
                .position(position)
                .boxed(),
        ]
    }

    /// The model's rotation around the vertical axis: a full turn every eight seconds while
    /// spinning, and the side bar's angle otherwise. The spin continues from where it stopped.
    fn rotation(&self) -> impl ReadState<T=Euler<Deg<f32>>> + Clone + use<> {
        // Paused while not spinning, so the views don't redraw for an animation nobody sees.
        let spinning = AnimatedState::linear()
            .duration(Duration::from_secs(8))
            .repeat()
            .running(self.spin.clone())
            .range(0.0, 360.0);

        Map3::read_map(self.spin.clone(), self.turn.clone(), spinning, |spin: &bool, turn: &Scalar, spinning: &f64| {
            let angle = if *spin { *spinning } else { *turn };
            Euler::new(Deg(0.0), Deg(angle as f32), Deg(0.0))
        })
    }

    /// The chosen surface, without the maps switched off in the side bar.
    fn active_surface(&self) -> impl ReadState<T=Surface> + Clone + use<> {
        let surface = Map2::read_map(self.surface.clone(), self.use_emission_map.clone(), |surface: &Surface, emission: &bool| Surface {
            emission: surface.emission.clone().filter(|_| *emission),
            ..surface.clone()
        });

        Map6::read_map(
            surface,
            self.use_color_map.clone(),
            self.use_normal_map.clone(),
            self.use_roughness_map.clone(),
            self.use_occlusion_map.clone(),
            self.use_metallic_map.clone(),
            |surface: &Surface, color: &bool, normal: &bool, roughness: &bool, occlusion: &bool, metallic: &bool| Surface {
                name: surface.name.clone(),
                albedo: surface.albedo.clone().filter(|_| *color),
                normal: surface.normal.clone().filter(|_| *normal),
                roughness: surface.roughness.clone().filter(|_| *roughness),
                ambient_occlusion: surface.ambient_occlusion.clone().filter(|_| *occlusion),
                metallic: surface.metallic.clone().filter(|_| *metallic),
                emission: surface.emission.clone(),
            },
        )
    }

    fn material(&self) -> PbrMaterial {
        let surface = self.active_surface();
        let f32_of = |state: &LocalState<Scalar>| Map1::read_map(state.clone(), |value: &Scalar| *value as f32);
        // The surface's colour map or the colour, alone or times the vertex colours.
        let albedo = Map3::read_map(self.color.clone(), surface.clone(), self.vertex_colors.clone(), |color: &Color, surface: &Surface, vertex: &bool| {
            match (surface.albedo.clone(), *vertex) {
                (Some(texture), true) => AlbedoComponent::TextureVertex { texture, srgb: true },
                (Some(texture), false) => AlbedoComponent::Texture(texture),
                (None, true) => AlbedoComponent::ValueVertex { value: *color, srgb: true },
                (None, false) => AlbedoComponent::Value(*color),
            }
        });

        PbrMaterial::new()
            .color(albedo)
            .roughness(Map1::read_map(self.roughness.clone(), |roughness: &Scalar| Some(*roughness as f32)))
            .metallic(f32_of(&self.metallic))
            .reflectance(f32_of(&self.reflectance))
            .clearcoat(f32_of(&self.clearcoat), f32_of(&self.clearcoat_roughness))
            .ambient_occlusion(f32_of(&self.ambient_occlusion))
            // A surface's emission map glows white while Glow is None, and in the Glow colour
            // otherwise.
            .emissive(Map2::read_map(self.emissive.clone(), surface.clone(), |glow: &Color, surface: &Surface| {
                if surface.emission.is_some() && *glow == BLACK { WHITE } else { *glow }
            }))
            .emissive_map(Map1::read_map(surface.clone(), |surface: &Surface| surface.emission.clone()))
            .tiling(Map1::read_map(self.tiling.clone(), |tiling: &Scalar| Vector2::new(*tiling as f32, *tiling as f32)))
            .sample_type(self.sampling.clone())
            .emissive_strength(f32_of(&self.emissive_strength))
            .normal(Map1::read_map(surface.clone(), |surface: &Surface| surface.normal_texture()))
            .aomr_textures(Map1::read_map(surface, |surface: &Surface| surface.aomr()))
    }

    /// `view` with the side bar's background, interactivity, exposure and tone mapping. A click on
    /// nothing in it clears the selection.
    fn look<N: Node3dSequence>(&self, view: Scene3d<N>) -> Scene3d<N> {
        let selected = self.outlined.clone();
        view.background(self.background.clone())
            .interactive(self.interactive.clone())
            .on_missed_click(move |_: &mut Environment| selected.clone().set_value(false))
            .exposure(Map1::read_map(self.exposure.clone(), |exposure: &Scalar| *exposure as f32))
            .tonemapping(self.tonemapping.clone())
    }

    /// The renderer's factory, put in the environment of the views.
    fn factory(&self) -> impl ReadState<T=ContextFactory3d> + Clone + use<> {
        Map1::read_map(self.renderer.clone(), |renderer: &Renderer| match renderer {
            Renderer::Wgpu => ContextFactory3d {
                id: RenderContext3dId::of::<WgpuRenderContext3d>(),
                render_context: |_| Box::new(WgpuRenderContext3d::new()),
            },
            Renderer::None => ContextFactory3d {
                id: RenderContext3dId::of::<NoopRenderContext3d>(),
                render_context: |_| Box::new(NoopRenderContext3d),
            },
        })
    }
}

/// The main camera's starting point, looking at the origin: 30° above the horizon and turned 35°
/// around the vertical axis towards +X, about 2.8 metres away, so it sees the front, right and top,
/// and all three positive axes.
fn default_view() -> Matrix4<f32> {
    look_at(Point3::new(1.4, 1.4, 2.0), Point3::new(0.0, 0.0, 0.0))
}

/// The nodes of `model`, all with the side bar's material, in a group turned by the side bar, and
/// the side bar's light, which stays where it is as the model turns.
fn nodes(model: Model, file_materials: bool, hover: bool, options: &Options) -> Vec<Box<dyn AnyNode3d>> {
    let mut parts: Vec<Box<dyn AnyNode3d>> = vec![];

    match model {
        Model::Empty => {}
        Model::Cube => {
            parts.push(Object::new(meshes::painted(Mesh::cube(1.0)), options.material()).boxed());
        }
        Model::Sphere => {
            parts.push(Object::new(meshes::painted(Mesh::icosphere(0.6, 3)), options.material()).boxed());
        }
        Model::Cubes => {
            for x in -1..=1 {
                for z in -1..=1 {
                    let cube = Object::new(meshes::painted(Mesh::cube(0.4)), options.material())
                        .position(Vector3::new(x as f32 * 0.6, 0.0, z as f32 * 0.6));
                    parts.push(cube.boxed());
                }
            }
        }
        Model::MaterialGrid => {
            // Facing +Z, centred on the origin: columns along X, rows down Y.
            let sphere = meshes::painted(Mesh::icosphere(GRID_RADIUS, 3));
            let mut spheres: Vec<Box<dyn AnyNode3d>> = vec![];
            for (row, metallic) in GRID_STEPS.iter().enumerate() {
                for (column, roughness) in GRID_STEPS.iter().enumerate() {
                    let material = options.material().roughness(Some(*roughness)).metallic(*metallic);
                    let position = Vector3::new((column as f32 - 2.0) * GRID_SPACING, (2.0 - row as f32) * GRID_SPACING, 0.0);
                    spheres.push(Object::new(sphere.clone(), material).position(position).boxed());
                }
            }
            // The spheres don't shadow each other, as in the charts; they still cast onto the floor.
            parts.push(Group::new(spheres).receives_shadows(false).boxed());
        }
        Model::RenderBall | Model::Wolf | Model::MoriKnob | Model::SportsCar => {
            // Loaded the first time the model is drawn: the views of every model are built up
            // front, but only the chosen one is drawn.
            let options = options.clone();
            parts.push(LazyObj::new(move || {
                let loaded = model.load().unwrap();
                loaded.parts.iter().cloned().map(|(mesh, material)| {
                    let object = if file_materials {
                        Object::new(mesh, material)
                    } else {
                        Object::new(meshes::painted(mesh), options.material())
                    };
                    object.transform(loaded.fit).boxed()
                }).collect()
            }).boxed());
        }
    }

    let mut nodes = options.lights();

    // The model: clicking it selects or deselects it. Selected, it has the side bar's outline;
    // hovered, a thin one, when hover is on. The selection is the app's own state, as the library
    // leaves it.
    let outlined = Map2::read_map(options.outlined.clone(), options.hovered.clone(), move |selected: &bool, hovered: &bool| *selected || (hover && *hovered));
    let thickness = Map2::read_map(options.outlined.clone(), options.outline_thickness.clone(), |selected: &bool, thickness: &Scalar| {
        if *selected { *thickness as f32 } else { 1.0 }
    });
    let select = {
        let (selected, last_hit) = (options.outlined.clone(), options.last_hit.clone());
        move |hit: &PickHit, _: &mut Environment| {
            let now = !*selected.value();
            selected.clone().set_value(now);
            last_hit.clone().set_value(Some(hit.position));
        }
    };
    let group = Group::new(parts).rotation(options.rotation()).on_click(select);
    // Without hover, no hover handler: the views then pick only on clicks.
    let group: Box<dyn AnyNode3d> = if hover {
        group.hovered(options.hovered.clone()).outlined(outlined).outline_thickness(thickness).boxed()
    } else {
        group.outlined(outlined).outline_thickness(thickness).boxed()
    };
    nodes.push(group);
    nodes.push(floor(model, options).boxed());
    // The scene's axes, which don't turn with the model.
    nodes.push(AxisHelper::new(1.0).shown(options.axes.clone()).boxed());
    nodes
}

/// A floor 6 metres square just under `model`, shown when the side bar says so: an empty mesh
/// otherwise. It doesn't turn with the model.
fn floor(model: Model, options: &Options) -> impl Node3dExt {
    // A model's height, read when the floor is drawn: for an OBJ model, its bottom, which loads it,
    // so only once the model is shown.
    let height = move || match model {
        Model::Empty => 0.0,
        Model::Cube => -0.5,
        Model::RenderBall | Model::Wolf | Model::MoriKnob | Model::SportsCar => model.load().unwrap().bottom,
        Model::Sphere => -0.6,
        Model::Cubes => -0.2,
        Model::MaterialGrid => -2.0 * GRID_SPACING - GRID_RADIUS - 0.05,
    };
    let position = Map1::read_map(options.floor.clone(), move |_: &bool| Vector3::new(0.0, height(), 0.0));

    let (plane, empty) = (Mesh::plane(6.0, 6.0), Mesh::new(vec![], vec![]));
    let mesh = Map1::read_map(options.floor.clone(), move |shown: &bool| if *shown { plane.clone() } else { empty.clone() });

    Object::new(mesh, PbrMaterial::new().color(rgba_bytes(200, 200, 200, 1.0)).roughness(Some(0.9f32)))
        .position(position)
}

/// An OBJ model's parts, built by `build` the first time they are drawn, then kept.
#[derive(Clone)]
struct LazyObj {
    id: NodeId,
    build: Rc<dyn Fn() -> Vec<Box<dyn AnyNode3d>>>,
    parts: Option<Vec<Box<dyn AnyNode3d>>>,
}

impl LazyObj {
    fn new(build: impl Fn() -> Vec<Box<dyn AnyNode3d>> + 'static) -> LazyObj {
        LazyObj { id: NodeId::new(), build: Rc::new(build), parts: None }
    }
}

impl std::fmt::Debug for LazyObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyObj").field("loaded", &self.parts.is_some()).finish()
    }
}

impl Render3d for LazyObj {
    fn render(&mut self, context: &mut RenderContext3d) {
        let parts = self.parts.get_or_insert_with(|| (self.build)());
        for part in parts {
            part.render(context);
        }
    }
}

impl CommonNode3d for LazyObj {
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
        for part in self.parts.iter().flatten() {
            f(part);
        }
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        for part in self.parts.iter_mut().flatten() {
            f(part);
        }
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        for part in self.parts.iter_mut().flatten().rev() {
            f(part);
        }
    }
}

/// The views of `model`: the main view with an orbit controller, and in two-view mode a front
/// orthographic view of the same scene, a clone with its own camera and controller.
fn views(model: Model, file_materials: bool, hover: bool, options: &Options) -> Box<dyn AnyWidget> {
    // The ambient light is the scene's, shared by both views.
    let scene = Scene3d::new(nodes(model, file_materials, hover, options))
        .hemisphere(WHITE, options.ground.clone(), Map1::read_map(options.ambient.clone(), |ambient: &Scalar| *ambient as f32));

    // The height limits match the side bar's slider, so a pinch stays within what it shows.
    let main = options.look(scene.clone())
        .camera(options.main_camera())
        .controller(OrbitController::new().height(0.1, 10.0));

    let front = options.look(scene)
        .camera(Camera::orthographic(2.0).look_at(Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 0.0, 0.0)))
        .controller(OrbitController::new());

    // The clone of `main` gets a copy of its camera; giving it the main camera again keeps it on
    // the shared view matrix, so "Reset view" and the position read-out follow it.
    let two = HStack::new((
        framed(main.clone().camera(options.main_camera())),
        framed(front),
    )).spacing(10.0);

    let main = framed(main);

    let views = options.views.clone();
    ui!(
        match views {
            Views::One => main,
            Views::Two => two,
        }
    ).boxed()
}

/// The views of `model`, rebuilt when the hover toggle changes: with or without a hover handler on
/// the model.
fn hover_views(model: Model, file_materials: bool, options: &Options) -> Box<dyn AnyWidget> {
    let o = options.clone();
    let hover = options.hover.clone();

    ui!(
        match hover {
            true => views(model, file_materials, true, &o),
            false => views(model, file_materials, false, &o),
        }
    ).boxed()
}

/// The views of an OBJ model, rebuilt when the toggle changes, with its file's materials or the
/// side bar's.
fn obj_views(model: Model, options: &Options) -> Box<dyn AnyWidget> {
    let o = options.clone();
    let file_materials = options.file_materials.clone();

    ui!(
        match file_materials {
            true => hover_views(model, true, &o),
            false => hover_views(model, false, &o),
        }
    ).boxed()
}

/// The playground: the views on the left, the options on the right.
pub fn playground(options: Options) -> impl Widget {
    let o = options.clone();
    let model = options.model.clone();

    let scene = ui!(
        match model {
            Model::Empty => hover_views(Model::Empty, false, &o),
            Model::Cube => hover_views(Model::Cube, false, &o),
            Model::Sphere => hover_views(Model::Sphere, false, &o),
            Model::Cubes => hover_views(Model::Cubes, false, &o),
            Model::RenderBall => obj_views(Model::RenderBall, &o),
            Model::Wolf => obj_views(Model::Wolf, &o),
            Model::MoriKnob => obj_views(Model::MoriKnob, &o),
            Model::SportsCar => obj_views(Model::SportsCar, &o),
            Model::MaterialGrid => hover_views(Model::MaterialGrid, false, &o),
        }
    );

    // The renderer's factory and the shadows are inherited, like styles: set here, they reach every
    // view. Set on a simple widget, the padding: a `ui!` match is a proxy that its parent sees
    // through, which would skip the modifiers.
    let scene = scene.padding(10.0);
    let scene = EnvUpdatingNew2::<_, ContextFactory3d, _>::new(options.factory(), scene)
        .casts_shadows(options.shadows.clone());

    HSplit::new(
        scene,
        ZStack::new((
            Rectangle::new().fill(EnvironmentColor::TertiarySystemFill),
            side_bar(&options)
                .padding(10.0)
                .picker_style(MenuStyle)
                .toggle_style(CheckboxStyle)
                .scroll()
                .with_scroll_direction(ScrollDirection::Vertical)
                .clip(),
        )),
    ).relative_to_end(300.0)
}

fn side_bar(options: &Options) -> impl Widget + use<> {
    let reset = options.view.clone();

    let scene = VStack::new((
        heading("Scene"),
        Picker::new("Model", options.model.clone(), (
            Text::new("Empty").tag(Model::Empty),
            Text::new("A cube").tag(Model::Cube),
            Text::new("A sphere").tag(Model::Sphere),
            Text::new("A grid of cubes").tag(Model::Cubes),
            Text::new("The render ball (OBJ)").tag(Model::RenderBall),
            Text::new("A wolf (OBJ, textured)").tag(Model::Wolf),
            Text::new("Mori's material knob (OBJ)").tag(Model::MoriKnob),
            Text::new("A sports car (OBJ)").tag(Model::SportsCar),
            Text::new("PBR grid: roughness × metallic").tag(Model::MaterialGrid),
        )),
        Picker::new("Views", options.views.clone(), (
            Text::new("One").tag(Views::One),
            Text::new("Two: orbit and front").tag(Views::Two),
        )),
        Picker::new("Renderer", options.renderer.clone(), (
            Text::new("wgpu").tag(Renderer::Wgpu),
            Text::new("None").tag(Renderer::None),
        )),
        Toggle::new("Show axes", options.axes.clone()),
        Toggle::new("Floor", options.floor.clone()),
        Toggle::new("Interactive", options.interactive.clone()),
        Toggle::new("Hover highlight", options.hover.clone()),
        Toggle::new("Selected (click the model)", options.outlined.clone()),
        shown_when(options.outlined.clone(), slider("Outline width", &options.outline_thickness, 1.0, 10.0, |value| format!("{value:.0} px"))),
        Text::new(Map3::read_map(options.hover.clone(), options.hovered.clone(), options.last_hit.clone(), |hover: &bool, hovered: &bool, hit: &Option<Vector3<f32>>| {
            let over = match (*hover, *hovered) {
                (false, _) => "not tracked",
                (true, true) => "over the model",
                (true, false) => "off the model",
            };
            match hit {
                Some(hit) => format!("Pointer {over}; last click at ({:.2}, {:.2}, {:.2})", hit.x, hit.y, hit.z),
                None => format!("Pointer {over}"),
            }
        })).font_size(EnvironmentFontSize::Footnote),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    let not_grid = Map1::read_map(options.model.clone(), |model: &Model| *model != Model::MaterialGrid);
    let render_ball = Map1::read_map(options.model.clone(), |model: &Model| model.obj_file().is_some());
    let using_file = Map2::read_map(options.model.clone(), options.file_materials.clone(), |model: &Model, file: &bool| {
        model.obj_file().is_some() && *file
    });
    let material = VStack::new((
        heading("Material"),
        shown_when(render_ball, Toggle::new("Materials from file", options.file_materials.clone())),
        shown_when(using_file, note("The model uses its MTL file's materials and maps; the values below apply to the other models.")),
        Picker::new(
            "Texture",
            options.surface.clone(),
            options.surfaces.iter().map(|surface| Text::new(surface.name.clone()).tag(surface.clone())).collect::<Vec<_>>(),
        ),
        // A checkbox for each map the surface has, to see the model with and without it.
        VStack::new((
            map_toggle(options, "Colour map", &options.use_color_map, |surface| surface.albedo.is_some()),
            map_toggle(options, "Normal map", &options.use_normal_map, |surface| surface.normal.is_some()),
            map_toggle(options, "Roughness map", &options.use_roughness_map, |surface| surface.roughness.is_some()),
            map_toggle(options, "Occlusion map", &options.use_occlusion_map, |surface| surface.ambient_occlusion.is_some()),
            map_toggle(options, "Metallic map", &options.use_metallic_map, |surface| surface.metallic.is_some()),
            map_toggle(options, "Emission map", &options.use_emission_map, |surface| surface.emission.is_some()),
            // How the maps are laid out and filtered, for any surface with an image.
            shown_when(
                Map1::read_map(options.surface.clone(), |surface: &Surface| surface.albedo.is_some() || surface.has_maps()),
                VStack::new((
                    slider("Tiling", &options.tiling, 1.0, 8.0, two_places),
                    Picker::new("Sampling", options.sampling.clone(), (
                        Text::new("Smooth (linear)").tag(SampleType::Linear),
                        Text::new("Pixels (nearest)").tag(SampleType::Nearest),
                    )),
                )).cross_axis_alignment(CrossAxisAlignment::Start),
            ),
            shown_when(
                Map1::read_map(options.surface.clone(), |surface: &Surface| surface.has_maps()),
                note("The roughness and metallic sliders scale the material's maps."),
            ),
        )).cross_axis_alignment(CrossAxisAlignment::Start),
        // A colour map takes the colour's place, so the colour picker gives way while one is used.
        shown_when(
            Map1::read_map(options.active_surface(), |surface: &Surface| surface.albedo.is_none()),
            Picker::new("Colour", options.color.clone(), (
                Text::new("Red").tag(RED),
                Text::new("Orange").tag(ORANGE),
                Text::new("Green").tag(GREEN),
                Text::new("Blue").tag(BLUE),
                Text::new("White").tag(WHITE),
            )),
        ),
        // The grid sets its own roughness and metallic, so their sliders give way to a note.
        shown_when(not_grid.clone(), slider("Roughness", &options.roughness, 0.0, 1.0, two_places)),
        shown_when(not_grid.clone(), slider("Metallic", &options.metallic, 0.0, 1.0, two_places)),
        shown_when(
            Map1::read_map(not_grid, |not_grid: &bool| !*not_grid),
            note("The grid: roughness 0 to 1 from left to right, metallic 0 to 1 from top to bottom."),
        ),
        VStack::new((
        slider("Reflectance", &options.reflectance, 0.0, 1.0, two_places),
        slider("Clear coat", &options.clearcoat, 0.0, 1.0, two_places),
        shown_when(
            Map1::read_map(options.clearcoat.clone(), |coat: &Scalar| *coat > 0.0),
            slider("Coat roughness", &options.clearcoat_roughness, 0.0, 1.0, two_places),
        ),
        slider("Ambient occl.", &options.ambient_occlusion, 0.0, 1.0, two_places),
        Picker::new("Glow", options.emissive.clone(), (
            Text::new("None").tag(BLACK),
            Text::new("Red").tag(RED),
            Text::new("Orange").tag(ORANGE),
            Text::new("Green").tag(GREEN),
            Text::new("Blue").tag(BLUE),
            Text::new("White").tag(WHITE),
        )),
        shown_when(
            Map1::read_map(options.emissive.clone(), |color: &Color| *color != BLACK),
            slider("Glow strength", &options.emissive_strength, 0.0, 5.0, two_places),
        ),
        Toggle::new("Vertex colours", options.vertex_colors.clone()),
        )).cross_axis_alignment(CrossAxisAlignment::Start),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    let spin_off = Map1::read_map(options.spin.clone(), |spin: &bool| !*spin);
    let motion = VStack::new((
        heading("Motion"),
        Toggle::new("Spin", options.spin.clone()),
        shown_when(spin_off, slider("Turn", &options.turn, -180.0, 180.0, degrees)),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    let environment = VStack::new((
        heading("Environment"),
        Picker::new("Background", options.background.clone(), (
            Text::new("None").tag(TRANSPARENT),
            Text::new("Night").tag(rgba_bytes(20, 30, 70, 1.0)),
            Text::new("Paper").tag(rgba_bytes(235, 232, 225, 1.0)),
        )),
        Picker::new("Ground light", options.ground.clone(), (
            Text::new("White, like the sky").tag(WHITE),
            Text::new("Warm").tag(rgba_bytes(160, 90, 40, 1.0)),
            Text::new("Cool").tag(rgba_bytes(60, 90, 160, 1.0)),
        )),
        slider("Ambient", &options.ambient, 0.0, 1.0, |value| format!("{value:.2}")),
        slider("Exposure", &options.exposure, 0.1, 3.0, |value| format!("{value:.2}")),
        Picker::new("Tone mapping", options.tonemapping.clone(), (
            Text::new("PBR Neutral").tag(Tonemapping::PbrNeutral),
            Text::new("ACES").tag(Tonemapping::Aces),
            Text::new("None").tag(Tonemapping::None),
        )),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    let light = VStack::new((
        heading("Light"),
        Picker::new("Kind", options.light_kind.clone(), (
            Text::new("Directional").tag(LightKind::Directional),
            Text::new("Point, 2.5 m away").tag(LightKind::Point),
            Text::new("Spot, 2.5 m away").tag(LightKind::Spot),
        )),
        slider("From", &options.light_azimuth, -180.0, 180.0, degrees),
        slider("Height", &options.light_elevation, -90.0, 90.0, degrees),
        slider("Lux at model", &options.light_intensity, 0.0, 20.0, |value| format!("{value:.1}")),
        Picker::new("Colour", options.light_color.clone(), (
            Text::new("White").tag(LightColor::White),
            Text::new("Warm white, 2700 K").tag(LightColor::Warm),
            Text::new("Cool white, 8000 K").tag(LightColor::Cool),
            Text::new("Red").tag(LightColor::Red),
            Text::new("Green").tag(LightColor::Green),
            Text::new("Blue").tag(LightColor::Blue),
            Text::new("Temperature").tag(LightColor::Temperature),
        )),
        shown_when(
            Map1::read_map(options.light_color.clone(), |color: &LightColor| *color == LightColor::Temperature),
            slider("Temperature", &options.light_temperature, 1000.0, 12000.0, |kelvin| format!("{kelvin:.0} K")),
        ),
        shown_when(
            Map1::read_map(options.light_kind.clone(), |kind: &LightKind| *kind == LightKind::Spot),
            slider("Spot cone", &options.spot_angle, 5.0, 60.0, degrees),
        ),
        // Point lights don't cast shadows yet.
        shown_when(
            Map1::read_map(options.light_kind.clone(), |kind: &LightKind| *kind != LightKind::Point),
            Toggle::new("Shadows", options.shadows.clone()),
        ),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    let camera = VStack::new((
        heading("Camera"),
        Picker::new("Projection", options.projection.clone(), (
            Text::new("Perspective").tag(ProjectionKind::Perspective),
            Text::new("Orthographic").tag(ProjectionKind::Orthographic),
        )),
        shown_when(
            Map1::read_map(options.projection.clone(), |kind: &ProjectionKind| *kind == ProjectionKind::Perspective),
            slider("Field of view", &options.fov, 10.0, 120.0, degrees),
        ),
        shown_when(
            Map1::read_map(options.projection.clone(), |kind: &ProjectionKind| *kind == ProjectionKind::Orthographic),
            slider("Ortho height", &options.height, 0.1, 10.0, |value| format!("{value:.1}")),
        ),
        Text::new(Map1::read_map(options.view.clone(), |view: &Matrix4<f32>| {
            let world = view.invert().unwrap_or(Matrix4::identity());
            format!("At ({:.2}, {:.2}, {:.2})", world.w.x, world.w.y, world.w.z)
        })),
        Button::new("Reset view", move |_| {
            let mut reset = reset.clone();
            *reset.value_mut() = default_view();
        }).button_style(BorderedProminentStyle).frame(110.0, 26.0),
        note("Drag or scroll in a view to orbit, pinch to zoom: closer in perspective, a smaller height in orthographic. The arrow keys turn the main view."),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    // With no renderer nothing is drawn, so the material, light and camera have nothing to show.
    let rendering = Map1::read_map(options.renderer.clone(), |renderer: &Renderer| *renderer != Renderer::None);
    let not_rendering = Map1::read_map(options.renderer.clone(), |renderer: &Renderer| *renderer == Renderer::None);

    VStack::new((
        scene,
        shown_when(not_rendering, note("With no renderer, the views draw nothing.")),
        shown_when(rendering, VStack::new((material, motion, environment, light, camera)).cross_axis_alignment(CrossAxisAlignment::Start)),
        Spacer::new(),
    ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
}

/// `widget`, shown only while `condition` holds: an option that can't apply while other options
/// are as they are is left out of the side bar, taking no room.
/// A checkbox for one of the surface's maps, shown only when the chosen surface has it.
fn map_toggle(options: &Options, label: &str, used: &LocalState<bool>, has: fn(&Surface) -> bool) -> impl Widget + use<> {
    shown_when(
        Map1::read_map(options.surface.clone(), move |surface: &Surface| has(surface)),
        Toggle::new(label.to_string(), used.clone()),
    )
}

fn shown_when(condition: impl ReadState<T=bool> + Clone + 'static, widget: impl Widget) -> impl Widget {
    IfElse::new(condition).when_true(widget)
}

/// A line of explanation in the side bar.
fn note(text: &str) -> impl Widget + use<> {
    Text::new(text.to_string()).frame_fixed_width(270.0).fit_height()
}

/// A view, with a border marking its bounds.
fn framed(view: impl Widget) -> Box<dyn AnyWidget> {
    view.border().color(EnvironmentColor::Separator).boxed()
}

fn heading(title: &str) -> impl Widget + use<> {
    Text::new(title.to_string())
        .font_size(EnvironmentFontSize::Title3)
        .padding(EdgeInsets::single(8.0, 0.0, 0.0, 0.0))
}

fn slider(label: &str, value: &LocalState<Scalar>, min: Scalar, max: Scalar, format: fn(&Scalar) -> String) -> impl Widget + use<> {
    HStack::new((
        Text::new(label.to_string()),
        Slider::new(value.clone(), min, max),
        Text::new(Map1::read_map(value.clone(), format)).frame_fixed_width(50.0),
    )).frame_fixed_height(26.0)
}

fn two_places(value: &Scalar) -> String {
    format!("{value:.2}")
}

fn degrees(degrees: &Scalar) -> String {
    format!("{degrees:.0}°")
}
