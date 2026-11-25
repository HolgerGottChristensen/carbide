use crate::camera::CameraSpec;
use crate::material::Material;
use crate::mesh::Mesh;
use carbide::draw::Color;
use carbide::environment::Environment;
use carbide::render::Layer;
use carbide::math::{Deg, Matrix4, Vector3};
use crate::render::{InnerRenderContext3d, ViewSpec};

pub struct RenderContext3d<'a, 'b: 'a> {
    pub(crate) render: &'a mut dyn InnerRenderContext3d,
    //pub(crate) image: &'a mut dyn InnerImageContext3d,
    pub env: &'a mut Environment<'b>,
}

impl<'a, 'b: 'a> RenderContext3d<'a, 'b> {
    pub fn render(&mut self, layer: Layer, camera: CameraSpec, view: ViewSpec) {
        self.render.render(layer, camera, view, self.env)
    }

    pub fn transform<R, F: FnOnce(&mut RenderContext3d)->R>(&mut self, transform: &Matrix4<f32>, f: F) -> R {
        self.render.transform(transform);
        let res = f(self);
        self.render.pop_transform();
        res
    }

    pub fn material<R, F: FnOnce(&mut RenderContext3d)->R>(&mut self, material: &Material, f: F) -> R {
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

    pub fn point(&mut self, color: Color, intensity: f32, range: Option<f32>) {
        self.render.point(color, intensity, range)
    }

    pub fn spot(&mut self, color: Color, intensity: f32, range: Option<f32>, inner: Deg<f32>, outer: Deg<f32>) {
        self.render.spot(color, intensity, range, inner, outer)
    }
}


