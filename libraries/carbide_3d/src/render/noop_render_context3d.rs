use carbide::color::Color;
use carbide::environment::Environment;
use carbide::math::{Deg, Matrix4, Vector3};
use carbide::render::Layer;
use crate::camera::CameraSpec;
use crate::material::Material;
use crate::Mesh;
use crate::render::{InnerRenderContext3d, LightShadow, MeshSettings, ViewSpec};

/// The render context of a scene without a 3D renderer, used until a
/// [`ContextFactory3d`](crate::render::ContextFactory3d) is found in the environment. It draws
/// nothing, so the scene's layer stays empty and transparent.
#[derive(Clone, Debug)]
pub struct NoopRenderContext3d;

impl InnerRenderContext3d for NoopRenderContext3d {
    fn start(&mut self) {
    }

    fn render(&mut self, _layer: Layer, _camera: CameraSpec, _view: ViewSpec, _env: &mut Environment) {
    }

    fn mesh(&mut self, _mesh: &Mesh, _settings: MeshSettings) {
    }

    fn material(&mut self, _material: &Material, _env: &mut Environment) {
    }

    fn pop_material(&mut self) {
    }

    fn transform(&mut self, _transform: &Matrix4<f32>) {
    }

    fn pop_transform(&mut self) {
    }

    fn directional(&mut self, _color: Color, _intensity: f32, _direction: Vector3<f32>, _shadow: Option<LightShadow>) {
    }

    fn point(&mut self, _color: Color, _intensity: f32, _range: Option<f32>) {
    }

    fn spot(&mut self, _color: Color, _intensity: f32, _range: Option<f32>, _inner: Deg<f32>, _outer: Deg<f32>, _shadow: Option<LightShadow>) {
    }

    fn hemisphere(&mut self, _sky: Color, _ground: Color, _intensity: f32) {
    }
}
