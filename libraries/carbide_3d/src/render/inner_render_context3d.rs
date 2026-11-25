use dyn_clone::DynClone;
use carbide::any_debug::AnyDebug;
use carbide::color::Color;
use carbide::environment::Environment;
use carbide::math::{Matrix4, Vector3};
use carbide::render::Layer;
use crate::camera::CameraSpec;
use crate::render::ViewSpec;
use crate::material::Material;
use crate::{InnerImageContext3d, Mesh};

pub trait InnerRenderContext3d: AnyDebug + DynClone + 'static {
    fn start(&mut self);

    /// Draw what the calls since [`start`](Self::start) described into `layer`, through `camera`,
    /// looking as `view` says.
    fn render(&mut self, layer: Layer, camera: CameraSpec, view: ViewSpec, env: &mut Environment);

    fn mesh(&mut self, mesh: &Mesh);

    fn material(&mut self, material: &Material, env: &mut Environment);
    fn pop_material(&mut self);

    /// Push `transform`, composed with the current one (`current * transform`). Meshes and lights
    /// until the matching [`pop_transform`](Self::pop_transform) are in its space.
    fn transform(&mut self, transform: &Matrix4<f32>);
    fn pop_transform(&mut self);

    /// A directional light. `direction` is the way the light shines, in the space of the current
    /// transform: a light inside a rotated group turns with it.
    fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>);

    /// The scene's ambient light: `sky` lights surfaces facing up, `ground` surfaces facing down,
    /// blended by each surface's normal, both times `intensity`. Colours are in sRGB.
    fn hemisphere(&mut self, sky: Color, ground: Color, intensity: f32);

    fn image_context(&mut self) -> &mut dyn InnerImageContext3d;
}

dyn_clone::clone_trait_object!(InnerRenderContext3d);