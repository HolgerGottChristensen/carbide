use std::any::{Any, TypeId};
use crate::camera::{Camera, CameraSpec};
use crate::image_context3d::InnerImageContext3d;
use crate::material::Material;
use crate::mesh::Mesh;
use carbide::draw::Color;
use carbide::environment::{Environment, EnvironmentKey};
use carbide::render::Layer;
use dyn_clone::DynClone;
use std::fmt::Debug;
use carbide::math::{Matrix4, Vector3};
use crate::render::InnerRenderContext3d;

pub struct RenderContext3d<'a, 'b: 'a> {
    pub(crate) render: &'a mut dyn InnerRenderContext3d,
    //pub(crate) image: &'a mut dyn InnerImageContext3d,
    pub env: &'a mut Environment<'b>,
}

impl<'a, 'b: 'a> RenderContext3d<'a, 'b> {
    pub fn render(&mut self, layer: Layer, camera: CameraSpec) {
        self.render.render(layer, camera, self.env)
    }

    pub fn transform<R, F: Fn(&mut RenderContext3d)->R>(&mut self, transform: &Matrix4<f32>, f: F) -> R {
        self.render.transform(transform);
        let res = f(self);
        self.render.pop_transform();
        res
    }

    pub fn material<R, F: Fn(&mut RenderContext3d)->R>(&mut self, material: &Material, f: F) -> R {
        self.render.material(material, self.env);
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


