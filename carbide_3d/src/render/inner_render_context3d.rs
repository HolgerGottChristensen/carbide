use std::any::TypeId;
use std::fmt::Debug;
use dyn_clone::DynClone;
use carbide::any_debug::AnyDebug;
use carbide::color::Color;
use carbide::environment::Environment;
use carbide::math::{Matrix4, Vector3};
use carbide::render::Layer;
use crate::camera::{Camera, CameraSpec};
use crate::material::Material;
use crate::{InnerImageContext3d, Mesh};

pub trait InnerRenderContext3d: AnyDebug + DynClone + 'static {
    fn start(&mut self);

    fn render(&mut self, layer: Layer, camera_spec: CameraSpec, env: &mut Environment);

    fn mesh(&mut self, mesh: &Mesh);

    fn material(&mut self, material: &Material, env: &mut Environment);
    fn pop_material(&mut self);

    fn transform(&mut self, transform: &Matrix4<f32>);
    fn pop_transform(&mut self);

    fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>);

    fn image_context(&mut self) -> &mut dyn InnerImageContext3d;
}

dyn_clone::clone_trait_object!(InnerRenderContext3d);