use carbide::render::matrix::{Deg, Euler, Matrix4, SquareMatrix};
use carbide::state::StateSync;
use carbide::widget::Empty;
use crate::material::Material;
use crate::material::pbr_material::PbrMaterial;
use crate::mesh::Mesh;
use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render3d::Render3d;
use crate::RenderContext3d;

#[derive(Debug, Clone)]
pub struct Object {
    id: NodeId,
    mesh: Mesh,
    pub(crate) transform: Matrix4<f32>,
    material: Material,
}

impl Object {
    pub fn new(mesh: Mesh, material: impl Into<Material>) -> Object {
        Object {
            id: Default::default(),
            mesh,
            transform: Matrix4::identity(),
            material: material.into(),
        }
    }
}

impl Render3d for Object {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.material.sync(context.env_stack);
        //self.transform = Matrix4::from(Euler::new(Deg(0.0), Deg(0.5), Deg(0.0))) * self.transform;

        context.material(&self.material, |context| {
            context.transform(&self.transform, |context| {
                context.mesh(&self.mesh)
            })
        })
    }
}

impl CommonNode3d for Object {
    fn id(&self) -> NodeId {
        self.id
    }

    fn transform(&self) -> Matrix4<f32> {
        self.transform
    }

    fn visible(&self) -> bool {
        true
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d)) {}

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}
}