use dyn_clone::DynClone;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};
use carbide::math::{Matrix4, Vector3};
use carbide::state::IntoReadState;
use crate::transform::{Rotation, Scale, Transformed};
use crate::shadows::{CastsShadows, ReceivesShadows, WithShadowSetting};
use crate::render::{Render3d, RenderContext3d};

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
    fn casts_shadows<S: IntoReadState<bool>>(self, casts: S) -> WithShadowSetting<Self, CastsShadows> {
        WithShadowSetting::new(self, casts)
    }

    /// Whether shadows fall on the objects in this node. Inherited, like a style; see
    /// [`shadows`](crate::shadows).
    fn receives_shadows<S: IntoReadState<bool>>(self, receives: S) -> WithShadowSetting<Self, ReceivesShadows> {
        WithShadowSetting::new(self, receives)
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
}