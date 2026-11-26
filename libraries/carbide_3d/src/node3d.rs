use carbide::color::Color;
use dyn_clone::DynClone;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};
use carbide::math::{Matrix4, Vector3};
use carbide::state::IntoReadState;
use crate::transform::{Rotation, Scale, Transformed};
use crate::environment_node::WithEnvironment;
use crate::outline::{OutlineColor, OutlineThickness, Outlined};
use crate::shadows::{CastsShadows, ReceivesShadows};
use crate::render::{Render3d, RenderContext3d};
use crate::picking::{ClickHandler, HoverHandler, HoveredState, PickHit, Pickable, PointerEvent3d, WithPointerHandler};
use carbide::environment::Environment;
use carbide::state::AnyState;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct NodeId(u32);

impl NodeId {
    /// Generate a new widget ID.
    pub fn new() -> Self {
        static NODE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
        NodeId(NODE_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for NodeId {
    fn default() -> Self {
        NodeId::new()
    }
}

pub trait AnyNode3d: Render3d + DynClone + Debug + 'static {}

dyn_clone::clone_trait_object!(AnyNode3d);

impl<T: Render3d + DynClone + Debug + 'static> AnyNode3d for T {}


impl Render3d for Box<dyn AnyNode3d> {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.deref_mut().render(context);
    }
}

impl CommonNode3d for Box<dyn AnyNode3d> {
    fn id(&self) -> NodeId {
        self.deref().id()
    }

    fn transform(&self) -> Matrix4<f32> {
        self.deref().transform()
    }

    fn visible(&self) -> bool {
        self.deref().visible()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d)) {
        self.deref().foreach_child(f)
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        self.deref_mut().foreach_child_mut(f)
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        self.deref_mut().foreach_child_rev(f)
    }

    fn process_pointer_event(&mut self, event: &PointerEvent3d, env: &mut Environment) {
        self.deref_mut().process_pointer_event(event, env)
    }
}

pub trait Node3d: AnyNode3d + Node3dExt + Clone + private::Sealed {}

impl<T> Node3d for T where T: AnyNode3d + Node3dExt + Clone {}

mod private {
    use crate::node3d::AnyNode3d;

    // This disallows implementing Widget manually, and requires something to implement
    // AnyWidget to implement Widget.
    pub trait Sealed {}

    impl<T> Sealed for T where T: AnyNode3d {}
}

/// Methods every node has. The transform methods wrap the node in a [`Transformed`]; called on a
/// `Transformed`, its own methods set that part of it instead.
pub trait Node3dExt: AnyNode3d + Clone + Sized {
    fn boxed(self) -> Box<dyn AnyNode3d> {
        Box::new(self)
    }

    /// Move the node by `position`. See [`Transformed::position`].
    fn position<P: IntoReadState<Vector3<f32>>>(self, position: P) -> Transformed<Self> {
        Transformed::new(self).position(position)
    }

    /// Turn the node. See [`Transformed::rotation`].
    fn rotation<R: IntoReadState<Rotation>>(self, rotation: R) -> Transformed<Self> {
        Transformed::new(self).rotation(rotation)
    }

    /// Scale the node. See [`Transformed::scale`].
    fn scale<S: IntoReadState<Scale>>(self, scale: S) -> Transformed<Self> {
        Transformed::new(self).scale(scale)
    }

    /// The point the node is turned and scaled around. See [`Transformed::pivot`].
    fn pivot<P: IntoReadState<Vector3<f32>>>(self, pivot: P) -> Transformed<Self> {
        Transformed::new(self).pivot(pivot)
    }

    /// Whether this node and everything inside it cast shadows: lights from their shadow maps,
    /// objects into them. Inherited, like a style; see [`shadows`](crate::shadows).
    fn casts_shadows<S: IntoReadState<bool>>(self, casts: S) -> WithEnvironment<Self, CastsShadows> {
        WithEnvironment::new(self, casts)
    }

    /// Whether shadows fall on the objects in this node. Inherited, like a style; see
    /// [`shadows`](crate::shadows).
    fn receives_shadows<S: IntoReadState<bool>>(self, receives: S) -> WithEnvironment<Self, ReceivesShadows> {
        WithEnvironment::new(self, receives)
    }

    /// Whether the meshes in this node can be picked. Picking is opt-in; pointer handlers make
    /// their subtree pickable by themselves. See [`picking`](crate::picking).
    fn pickable<S: IntoReadState<bool>>(self, pickable: S) -> WithEnvironment<Self, Pickable> {
        WithEnvironment::new(self, pickable)
    }

    /// Call `f` with the hit when this node's subtree is clicked: pressed and released on the
    /// same node without dragging. Makes the subtree pickable.
    fn on_click<F: FnMut(&PickHit, &mut Environment) + Clone + 'static>(self, f: F) -> WithPointerHandler<Self, ClickHandler<F>> {
        WithPointerHandler::new(self, ClickHandler(f))
    }

    /// Call `f` when the pointer moves onto this node's subtree, with the hit, and off it, with
    /// `None`. Makes the subtree pickable.
    fn on_hover<F: FnMut(Option<&PickHit>, &mut Environment) + Clone + 'static>(self, f: F) -> WithPointerHandler<Self, HoverHandler<F>> {
        WithPointerHandler::new(self, HoverHandler(f))
    }

    /// Set `hovered` to whether the pointer is over this node's subtree. Makes the subtree
    /// pickable.
    fn hovered<S: AnyState<T=bool> + Clone + 'static>(self, hovered: S) -> WithPointerHandler<Self, HoveredState> {
        WithPointerHandler::new(self, HoveredState(hovered.as_dyn()))
    }

    /// Whether this node and everything inside it are outlined: a band around their silhouette,
    /// drawn on top of the scene. Inherited, like a style; see [`outline`](crate::outline).
    fn outlined<S: IntoReadState<bool>>(self, outlined: S) -> WithEnvironment<Self, Outlined> {
        WithEnvironment::new(self, outlined)
    }

    /// The colour of outlines in this node. The environment's accent colour by default.
    fn outline_color<S: IntoReadState<Color>>(self, color: S) -> WithEnvironment<Self, OutlineColor> {
        WithEnvironment::new(self, color)
    }

    /// The thickness of outlines in this node, in logical pixels. 2 by default.
    fn outline_thickness<S: IntoReadState<f32>>(self, thickness: S) -> WithEnvironment<Self, OutlineThickness> {
        WithEnvironment::new(self, thickness)
    }

    /// Transform the node by a whole matrix. See [`Transformed::transform`].
    fn transform<M: IntoReadState<Matrix4<f32>>>(self, matrix: M) -> Transformed<Self> {
        Transformed::new(self).transform(matrix)
    }
}

impl<T: AnyNode3d + Clone + Sized> Node3dExt for T {}

pub trait CommonNode3d {
    fn id(&self) -> NodeId;

    fn transform(&self) -> Matrix4<f32>;

    fn visible(&self) -> bool;

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d));
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d));
    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d));

    /// Handle a pointer event whose path goes through this node. Only pointer handlers do
    /// anything with them; see [`picking`](crate::picking).
    fn process_pointer_event(&mut self, _event: &PointerEvent3d, _env: &mut Environment) {}
}