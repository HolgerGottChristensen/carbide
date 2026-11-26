use dyn_clone::DynClone;
use carbide::any_debug::AnyDebug;
use carbide::color::Color;
use carbide::environment::Environment;
use carbide::math::{Deg, Matrix4, Vector3};
use carbide::render::Layer;
use crate::camera::CameraSpec;
use crate::render::{LightShadow, ViewSpec, MeshSettings};
use crate::material::Material;
use crate::{InnerImageContext3d, Mesh};

pub trait InnerRenderContext3d: AnyDebug + DynClone + 'static {
    fn start(&mut self);

    /// Draw what the calls since [`start`](Self::start) described into `layer`, through `camera`,
    /// looking as `view` says.
    fn render(&mut self, layer: Layer, camera: CameraSpec, view: ViewSpec, env: &mut Environment);

    /// Draw `mesh` with the current material and transform. `casts_shadows` says whether it
    /// casts shadows from lights that cast them, and `receives_shadows` whether shadows fall on it.
    fn mesh(&mut self, mesh: &Mesh, settings: MeshSettings);

    fn material(&mut self, material: &Material, env: &mut Environment);
    fn pop_material(&mut self);

    /// Push `transform`, composed with the current one (`current * transform`). Meshes and lights
    /// until the matching [`pop_transform`](Self::pop_transform) are in its space.
    fn transform(&mut self, transform: &Matrix4<f32>);
    fn pop_transform(&mut self);

    /// A directional light. `direction` is the way the light shines, in the space of the current
    /// transform: a light inside a rotated group turns with it. `intensity` is in lux. With
    /// `shadow`, it casts shadows.
    fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>, shadow: Option<LightShadow>);

    /// A point light at the origin of the current transform. `intensity` is in candela, `range`
    /// (in metres) is where it has faded to nothing, and `color` is in sRGB.
    fn point(&mut self, color: Color, intensity: f32, range: Option<f32>);

    /// A spot light at the origin of the current transform, shining along its -Z: full brightness
    /// within `inner` of its axis, dark beyond `outer`. Otherwise as [`point`](Self::point).
    fn spot(&mut self, color: Color, intensity: f32, range: Option<f32>, inner: Deg<f32>, outer: Deg<f32>, shadow: Option<LightShadow>);

    /// The scene's ambient light: `sky` lights surfaces facing up, `ground` surfaces facing down,
    /// blended by each surface's normal, both times `intensity`. Colours are in sRGB.
    fn hemisphere(&mut self, sky: Color, ground: Color, intensity: f32);

    fn image_context(&mut self) -> &mut dyn InnerImageContext3d;
}

dyn_clone::clone_trait_object!(InnerRenderContext3d);