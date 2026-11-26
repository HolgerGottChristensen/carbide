use crate::node3d::NodeId;
use crate::picking::{PickRecorder, Pickable};
use carbide::environment::{EnvironmentKey, Shared};
use crate::outline::outline_in;
use crate::render::MeshSettings;
use crate::shadows::{CastsShadows, ReceivesShadows};
use crate::camera::CameraSpec;
use crate::material::Material;
use crate::mesh::Mesh;
use carbide::draw::Color;
use carbide::environment::Environment;
use carbide::render::Layer;
use carbide::math::{Deg, Matrix4, Vector3};
use crate::render::{InnerRenderContext3d, LightShadow, ViewSpec};

pub struct RenderContext3d<'a, 'b: 'a> {
    pub(crate) render: &'a mut dyn InnerRenderContext3d,
    //pub(crate) image: &'a mut dyn InnerImageContext3d,
    pub env: &'a mut Environment<'b>,
    /// What this frame records for picking.
    pub(crate) picking: &'a mut PickRecorder,
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

    /// Render `f` as part of the node `id`, for picking: meshes drawn in it record it in their
    /// path. Objects and pointer handlers call it.
    pub fn pick_node<R, F: FnOnce(&mut RenderContext3d)->R>(&mut self, id: NodeId, f: F) -> R {
        self.picking.push(id);
        let res = f(self);
        self.picking.pop();
        res
    }

    /// Render `f` with the environment key `K` set to `value`.
    pub fn with_env<K: EnvironmentKey<Access=Shared>, R>(&mut self, value: &K::Value, f: impl FnOnce(&mut RenderContext3d) -> R) -> R where K::Value: Clone {
        let render = &mut *self.render;
        let picking = &mut *self.picking;
        let mut result = None;
        self.env.with::<K>(value, |env| {
            result = Some(f(&mut RenderContext3d { render, env, picking }));
        });
        result.unwrap()
    }

    /// Draw `mesh` with the settings in the environment: it casts and receives shadows unless
    /// [`CastsShadows`](crate::CastsShadows) or [`ReceivesShadows`](crate::ReceivesShadows) say
    /// otherwise, and is outlined where [`Outlined`](crate::outline::Outlined) is set.
    pub fn mesh(&mut self, mesh: &Mesh) {
        let settings = MeshSettings {
            casts_shadows: self.env.get::<CastsShadows>().copied().unwrap_or(true),
            receives_shadows: self.env.get::<ReceivesShadows>().copied().unwrap_or(true),
            outline: outline_in(self.env),
            pick_id: if self.env.get::<Pickable>().copied().unwrap_or(false) { self.picking.record() } else { 0 },
        };

        self.render.mesh(mesh, settings)
    }

    /// Draw `mesh` with `settings`, whatever the environment says.
    pub fn mesh_with(&mut self, mesh: &Mesh, settings: MeshSettings) {
        self.render.mesh(mesh, settings)
    }

    pub fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>, shadow: Option<LightShadow>) {
        self.render.directional(color, intensity, direction, shadow)
    }

    pub fn point(&mut self, color: Color, intensity: f32, range: Option<f32>) {
        self.render.point(color, intensity, range)
    }

    pub fn spot(&mut self, color: Color, intensity: f32, range: Option<f32>, inner: Deg<f32>, outer: Deg<f32>, shadow: Option<LightShadow>) {
        self.render.spot(color, intensity, range, inner, outer, shadow)
    }
}


