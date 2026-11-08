use carbide::color::Color;
use carbide::environment::Environment;
use carbide::math::{Matrix4, Vector3};
use carbide::render::Layer;
use crate::camera::CameraSpec;
use crate::material::Material;
use crate::{InnerImageContext3d, Mesh};
use crate::render::InnerRenderContext3d;

#[derive(Clone, Debug)]
pub struct NoopRenderContext3d;

impl InnerRenderContext3d for NoopRenderContext3d {
    fn start(&mut self) {
    }

    fn render(&mut self, _layer: Layer, _camera: CameraSpec, _env: &mut Environment) {
    }

    fn mesh(&mut self, _mesh: &Mesh) {
    }

    fn material(&mut self, _material: &Material, _env: &mut Environment) {
    }

    fn pop_material(&mut self) {
    }

    fn transform(&mut self, _transform: &Matrix4<f32>) {
    }

    fn pop_transform(&mut self) {
    }

    fn directional(&mut self, _color: Color, _intensity: f32, _direction: Vector3<f32>) {
    }

    fn image_context(&mut self) -> &mut dyn InnerImageContext3d {
        todo!()
    }
}