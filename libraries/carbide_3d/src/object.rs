use carbide::math::{Matrix4, SquareMatrix};
use crate::material::Material;
use crate::mesh::Mesh;
use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use carbide::state::{AnyReadState, IntoReadState, ReadState, StateSync};
use crate::render::{Render3d, RenderContext3d};

/// A mesh drawn with a material. It sits at its own origin: move, turn or scale it with
/// `.position()`, `.rotation()`, `.scale()` and the like, which wrap it in a
/// [`Transformed`](crate::Transformed).
///
/// The mesh can be a state of a [`Mesh`]: setting the state to another mesh changes the object's
/// geometry.
#[derive(Debug, Clone)]
pub struct Object {
    id: NodeId,
    mesh: Box<dyn AnyReadState<T=Mesh>>,
    material: Material,
}

impl Object {
    pub fn new<M: IntoReadState<Mesh>>(mesh: M, material: impl Into<Material>) -> Object {
        Object {
            id: Default::default(),
            mesh: Box::new(mesh.into_read_state()),
            material: material.into(),
        }
    }
}

impl Render3d for Object {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.material.sync(context.env);
        self.mesh.sync(context.env);

        let mesh = self.mesh.value().clone();
        context.material(&self.material, |context| context.mesh(&mesh))
    }
}

impl CommonNode3d for Object {
    fn id(&self) -> NodeId {
        self.id
    }

    fn transform(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    fn visible(&self) -> bool {
        true
    }

    fn foreach_child<'a>(&'a self, _f: &mut dyn FnMut(&'a dyn AnyNode3d)) {}

    fn foreach_child_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}

    fn foreach_child_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}
}