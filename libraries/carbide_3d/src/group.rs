use carbide::math::{Matrix4, SquareMatrix};

use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::node3d_sequence::Node3dSequence;
use crate::render::{Render3d, RenderContext3d};

/// Nodes rendered together, in order. A group has no transform of its own: moving, turning or
/// scaling it, with `.position()`, `.rotation()` and the like, moves all of its nodes together.
///
/// ```ignore
/// Group::new((cube, ball)).rotation(spin)
/// ```
#[derive(Debug, Clone)]
pub struct Group<N> where N: Node3dSequence {
    id: NodeId,
    nodes: N,
}

impl<N: Node3dSequence> Group<N> {
    pub fn new(nodes: N) -> Group<N> {
        Group {
            id: NodeId::new(),
            nodes,
        }
    }
}

impl<N: Node3dSequence> Render3d for Group<N> {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.nodes.foreach_mut(&mut |node| node.render(context));
    }
}

impl<N: Node3dSequence> CommonNode3d for Group<N> {
    fn id(&self) -> NodeId {
        self.id
    }

    fn transform(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    fn visible(&self) -> bool {
        true
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d)) {
        self.nodes.foreach(f)
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        self.nodes.foreach_mut(f)
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        self.nodes.foreach_rev(f)
    }
}
