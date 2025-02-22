use crate::camera::Camera;
use crate::image_context3d::InnerImageContext3d;
use crate::material::Material;
use crate::mesh::Mesh;
use carbide::draw::Color;
use carbide::environment::Environment;
use carbide::render::matrix::{Matrix4, Vector3};
use carbide::render::Layer;
use dyn_clone::DynClone;
use std::fmt::Debug;

pub struct RenderContext3d<'a, 'b: 'a> {
    pub(crate) render: &'a mut dyn InnerRenderContext3d,
    pub(crate) image: &'a mut dyn InnerImageContext3d,
    pub env: &'a mut Environment<'b>,

}

impl<'a, 'b: 'a> RenderContext3d<'a, 'b> {
    pub fn render(&mut self, layer: Layer, camera: &dyn Camera) {
        self.render.render(layer, camera)
    }

    pub fn transform<R, F: Fn(&mut RenderContext3d)->R>(&mut self, transform: &Matrix4<f32>, f: F) -> R {
        self.render.transform(transform);
        let res = f(self);
        self.render.pop_transform();
        res
    }

    pub fn material<R, F: Fn(&mut RenderContext3d)->R>(&mut self, material: &Material, f: F) -> R {
        self.render.material(material);
        let res = f(self);
        self.render.pop_material();
        res
    }

    pub fn mesh(&mut self, mesh: &Mesh) {
        self.render.mesh(mesh)
    }

    pub fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>) {
        self.render.directional(color, intensity, direction)
    }
}

pub trait InnerRenderContext3d: Debug + DynClone + 'static {
    fn start(&mut self);

    fn render(&mut self, layer: Layer, camera: &dyn Camera);

    fn mesh(&mut self, mesh: &Mesh);

    fn material(&mut self, material: &Material);
    fn pop_material(&mut self);

    fn transform(&mut self, transform: &Matrix4<f32>);
    fn pop_transform(&mut self);

    fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>);
}

dyn_clone::clone_trait_object!(InnerRenderContext3d);