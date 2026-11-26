//! The playground: the scene, shown in one or two views, with its options in a side bar.

use carbide::color::{rgba_bytes, Color, BLUE, GREEN, ORANGE, RED, TRANSPARENT, WHITE};
use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::picker::{MenuStyle, Picker};
use carbide::controls::slider::Slider;
use carbide::controls::toggle::{CheckboxStyle, Toggle};
use carbide::controls::ControlsExt;
use carbide::d3::camera::{look_at, Camera, OrbitController, Projection};
use carbide::d3::light::{DirectionalLight, PointLight, SpotLight};
use carbide::d3::material::pbr_material::PbrMaterial;
use carbide::d3::node3d::AnyNode3d;
use carbide::d3::render::{ContextFactory3d, NoopRenderContext3d, RenderContext3dId, Tonemapping};
use carbide::d3::wgpu::WgpuRenderContext3d;
use carbide::d3::{AxisHelper, Group, Mesh, Node3dExt, Node3dSequence, Object, Rotation, Scene3d, ShadowsExt};
use carbide::draw::Scalar;
use carbide::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide::math::{Deg, Euler, Matrix4, Point3, Quaternion, SquareMatrix, Vector3};
use carbide::state::{AnimatedState, LocalState, Map1, Map2, Map3, ReadState, State};
use carbide::time::Duration;
use carbide::ui;
use carbide::widget::*;

use crate::meshes;

/// The models to show.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Model {
    /// No model: only the lights, and the axes when shown.
    Empty,
    Cube,
    Sphere,
    Cubes,
    RenderBall,
}

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

    pub color: LocalState<Color>,
    pub roughness: LocalState<Scalar>,

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
            color: LocalState::new(RED),
            roughness: LocalState::new(0.5),
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

    fn material(&self) -> PbrMaterial {
        PbrMaterial::new()
            .color(self.color.clone())
            .roughness(Map1::read_map(self.roughness.clone(), |roughness: &Scalar| Some(*roughness as f32)))
    }

    /// `view` with the side bar's background, exposure and tone mapping.
    fn look<N: Node3dSequence>(&self, view: Scene3d<N>) -> Scene3d<N> {
        view.background(self.background.clone())
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
fn nodes(model: Model, options: &Options) -> Vec<Box<dyn AnyNode3d>> {
    let mut parts: Vec<Box<dyn AnyNode3d>> = vec![];

    match model {
        Model::Empty => {}
        Model::Cube => {
            parts.push(Object::new(Mesh::cube(1.0), options.material()).boxed());
        }
        Model::Sphere => {
            parts.push(Object::new(Mesh::icosphere(0.6, 3), options.material()).boxed());
        }
        Model::Cubes => {
            for x in -1..=1 {
                for z in -1..=1 {
                    let cube = Object::new(Mesh::cube(0.4), options.material())
                        .position(Vector3::new(x as f32 * 0.6, 0.0, z as f32 * 0.6));
                    parts.push(cube.boxed());
                }
            }
        }
        Model::RenderBall => {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../libraries/carbide_3d/ressources/renderball.obj");
            let (meshes, fit) = meshes::load_obj(path);
            for mesh in meshes {
                parts.push(Object::new(mesh, options.material()).transform(fit).boxed());
            }
        }
    }

    let mut nodes = options.lights();
    nodes.push(Group::new(parts).rotation(options.rotation()).boxed());
    nodes.push(floor(model, options).boxed());
    // The scene's axes, which don't turn with the model.
    nodes.push(AxisHelper::new(1.0).shown(options.axes.clone()).boxed());
    nodes
}

/// A floor 6 metres square just under `model`, shown when the side bar says so: an empty mesh
/// otherwise. It doesn't turn with the model.
fn floor(model: Model, options: &Options) -> impl Node3dExt {
    let height = match model {
        Model::Empty => 0.0,
        Model::Cube | Model::RenderBall => -0.5,
        Model::Sphere => -0.6,
        Model::Cubes => -0.2,
    };

    let (plane, empty) = (Mesh::plane(6.0, 6.0), Mesh::new(vec![], vec![]));
    let mesh = Map1::read_map(options.floor.clone(), move |shown: &bool| if *shown { plane.clone() } else { empty.clone() });

    Object::new(mesh, PbrMaterial::new().color(rgba_bytes(200, 200, 200, 1.0)).roughness(Some(0.9f32)))
        .position(Vector3::new(0.0, height, 0.0))
}

/// The views of `model`: the main view with an orbit controller, and in two-view mode a front
/// orthographic view of the same scene, a clone with its own camera and controller.
fn views(model: Model, options: &Options) -> Box<dyn AnyWidget> {
    // The ambient light is the scene's, shared by both views.
    let scene = Scene3d::new(nodes(model, options))
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

/// The playground: the views on the left, the options on the right.
pub fn playground(options: Options) -> impl Widget {
    let o = options.clone();
    let model = options.model.clone();

    let scene = ui!(
        match model {
            Model::Empty => views(Model::Empty, &o),
            Model::Cube => views(Model::Cube, &o),
            Model::Sphere => views(Model::Sphere, &o),
            Model::Cubes => views(Model::Cubes, &o),
            Model::RenderBall => views(Model::RenderBall, &o),
        }
    );

    let scene = EnvUpdatingNew2::<_, ContextFactory3d, _>::new(options.factory(), scene);

    // Shadows are inherited, like styles: set here, they reach every node of every view. Set on a
    // simple widget: a `ui!` match is a proxy that its parent sees through, which would skip the
    // modifier.
    let scene = scene.padding(10.0).casts_shadows(options.shadows.clone());

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
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    let material = VStack::new((
        heading("Material"),
        Picker::new("Colour", options.color.clone(), (
            Text::new("Red").tag(RED),
            Text::new("Orange").tag(ORANGE),
            Text::new("Green").tag(GREEN),
            Text::new("Blue").tag(BLUE),
            Text::new("White").tag(WHITE),
        )),
        slider("Roughness", &options.roughness, 0.0, 1.0, |value| format!("{value:.2}")),
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

fn degrees(degrees: &Scalar) -> String {
    format!("{degrees:.0}°")
}
