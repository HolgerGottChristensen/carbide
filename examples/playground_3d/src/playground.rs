//! The playground: the scene, shown in one or two views, with its options in a side bar.

use carbide::color::{Color, BLUE, GREEN, ORANGE, RED, WHITE};
use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::picker::{MenuStyle, Picker};
use carbide::controls::slider::Slider;
use carbide::controls::toggle::{CheckboxStyle, Toggle};
use carbide::controls::ControlsExt;
use carbide::d3::camera::{look_at, Camera, OrbitController, Projection};
use carbide::d3::light::DirectionalLight;
use carbide::d3::material::pbr_material::PbrMaterial;
use carbide::d3::node3d::AnyNode3d;
use carbide::d3::render::{ContextFactory3d, NoopRenderContext3d, RenderContext3dId};
use carbide::d3::wgpu::WgpuRenderContext3d;
use carbide::d3::{Group, Mesh, Node3dExt, Object, Scene3d};
use carbide::draw::Scalar;
use carbide::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide::math::{Deg, Euler, Matrix4, Point3, SquareMatrix, Vector3};
use carbide::state::{AnimatedState, LocalState, Map1, Map3, ReadState, State};
use carbide::time::Duration;
use carbide::ui;
use carbide::widget::*;

use crate::meshes;

/// The models to show.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Model {
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

    pub color: LocalState<Color>,
    pub roughness: LocalState<Scalar>,

    /// Whether the model spins on its own, or is turned by `turn`.
    pub spin: LocalState<bool>,
    /// How far the model is turned around the vertical axis, in degrees, while it isn't spinning.
    pub turn: LocalState<Scalar>,

    pub light_azimuth: LocalState<Scalar>,
    pub light_elevation: LocalState<Scalar>,
    pub light_intensity: LocalState<Scalar>,

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
            color: LocalState::new(RED),
            roughness: LocalState::new(0.5),
            spin: LocalState::new(false),
            turn: LocalState::new(0.0),
            light_azimuth: LocalState::new(-35.0),
            light_elevation: LocalState::new(50.0),
            light_intensity: LocalState::new(5.0),
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

    /// The direction the light shines in, from the side bar's angles of where it comes from: an
    /// azimuth of 0° is from in front of the scene (the camera's side at the start, -Z), positive
    /// to the right, and the height is degrees above the horizon.
    fn light(&self) -> DirectionalLight<impl ReadState<T=Vector3<f32>> + Clone + use<>, Color, impl ReadState<T=f32> + Clone + use<>> {
        let direction = carbide::state::Map2::read_map(self.light_azimuth.clone(), self.light_elevation.clone(), |azimuth: &Scalar, elevation: &Scalar| {
            let (azimuth, elevation) = ((*azimuth as f32).to_radians(), (*elevation as f32).to_radians());
            let from = Vector3::new(elevation.cos() * azimuth.sin(), elevation.sin(), -elevation.cos() * azimuth.cos());
            -from
        });

        DirectionalLight::new(direction)
            .intensity(Map1::read_map(self.light_intensity.clone(), |intensity: &Scalar| *intensity as f32))
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

/// The main camera's starting point: above and in front of the origin, looking at it.
fn default_view() -> Matrix4<f32> {
    look_at(Point3::new(0.0, 1.2, -2.5), Point3::new(0.0, 0.0, 0.0))
}

/// The nodes of `model`, all with the side bar's material, in a group turned by the side bar, and
/// the side bar's light, which stays where it is as the model turns.
fn nodes(model: Model, options: &Options) -> Vec<Box<dyn AnyNode3d>> {
    let mut parts: Vec<Box<dyn AnyNode3d>> = vec![];

    match model {
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

    vec![
        options.light().boxed(),
        Group::new(parts).rotation(options.rotation()).boxed(),
    ]
}

/// The views of `model`: the main view with an orbit controller, and in two-view mode a front
/// orthographic view of the same scene, a clone with its own camera and controller.
fn views(model: Model, options: &Options) -> Box<dyn AnyWidget> {
    let scene = Scene3d::new(nodes(model, options));

    // The height limits match the side bar's slider, so a pinch stays within what it shows.
    let main = scene.clone()
        .camera(options.main_camera())
        .controller(OrbitController::new().height(0.1, 10.0));

    let front = scene
        .camera(Camera::orthographic(2.0).look_at(Point3::new(0.0, 0.0, -5.0), Point3::new(0.0, 0.0, 0.0)))
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
            Model::Cube => views(Model::Cube, &o),
            Model::Sphere => views(Model::Sphere, &o),
            Model::Cubes => views(Model::Cubes, &o),
            Model::RenderBall => views(Model::RenderBall, &o),
        }
    );

    let scene = EnvUpdatingNew2::<_, ContextFactory3d, _>::new(options.factory(), scene);

    HSplit::new(
        scene.padding(10.0),
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

    let light = VStack::new((
        heading("Light"),
        slider("From", &options.light_azimuth, -180.0, 180.0, degrees),
        slider("Height", &options.light_elevation, -90.0, 90.0, degrees),
        slider("Intensity", &options.light_intensity, 0.0, 20.0, |value| format!("{value:.1}")),
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
        shown_when(rendering, VStack::new((material, motion, light, camera)).cross_axis_alignment(CrossAxisAlignment::Start)),
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
