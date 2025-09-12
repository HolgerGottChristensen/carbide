use std::any::TypeId;
use carbide::color::Color;
use carbide::environment::Environment;
use carbide::math::{Matrix4, Vector3};
use carbide::render::Layer;
use crate::camera::{Camera, CameraSpec};
use crate::material::Material;
use crate::{InnerImageContext3d, Mesh};
use crate::render::InnerRenderContext3d;

#[derive(Clone, Debug)]
pub struct NoopRenderContext3d;

impl InnerRenderContext3d for NoopRenderContext3d {
    fn start(&mut self) {
    }

    fn render(&mut self, layer: Layer, camera: CameraSpec, env: &mut Environment) {
    }

    fn mesh(&mut self, mesh: &Mesh) {
    }

    fn material(&mut self, material: &Material, env: &mut Environment) {
    }

    fn pop_material(&mut self) {
    }

    fn transform(&mut self, transform: &Matrix4<f32>) {
    }

    fn pop_transform(&mut self) {
    }

    fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>) {
    }

    fn image_context(&mut self) -> &mut dyn InnerImageContext3d {
        todo!()
    }
}