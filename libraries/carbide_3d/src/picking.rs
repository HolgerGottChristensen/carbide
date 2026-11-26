//! Picking: which node is under the pointer, read from the renderer's picking buffer.
//!
//! Picking is opt-in. A subtree is pickable when [`Pickable`] is set for it, with
//! `.pickable(true)` from [`Node3dExt`](crate::Node3dExt), or when it has a pointer handler:
//! `.on_click(..)`, `.on_hover(..)` or `.hovered(..)`. `.pickable(false)` inside excludes parts.
//!
//! A [`Scene3d`](crate::Scene3d) with something to pick renders a picking buffer with each frame:
//! for each pixel, which mesh is nearest, and its depth. On pointer events inside the view it
//! reads the pixels around the pointer and finds the node hit:
//! - **Hover:** `.on_hover(..)` handlers and `.hovered(..)` states hear when the pointer enters and
//!   leaves their subtree.
//! - **Clicks:** `.on_click(..)` handlers hear a click that was pressed and released on the same
//!   node without dragging.
//!
//! Events bubble: every handler between the node hit and the scene hears them, innermost first.
//! If the pixel under the pointer is empty, the nearest pickable pixel within
//! [`PickTolerance`] (2 logical pixels by default, at most 7) counts.

use std::fmt::{Debug, Formatter};

use carbide::environment::{Environment, EnvironmentKey, Shared};
use carbide::math::{Matrix4, SquareMatrix, Vector3};
use carbide::state::{AnyState, IntoReadState, State};
use carbide::widget::{EnvUpdatingNew2, Widget};

use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render::{Render3d, RenderContext3d};

/// Whether the meshes in this part of the tree can be picked. Off where nothing is set, except
/// inside pointer handlers.
#[derive(Debug, Copy, Clone)]
pub struct Pickable;

impl EnvironmentKey for Pickable {
    type Value = bool;
    type Access = Shared;
}

/// How far from the pointer, in logical pixels, a pick finds the nearest pickable pixel when the
/// pixel under it is empty. 2 by default; at most 7.
#[derive(Debug, Copy, Clone)]
pub struct PickTolerance;

impl EnvironmentKey for PickTolerance {
    type Value = f32;
    type Access = Shared;
}

/// What a pick found.
#[derive(Debug, Clone, PartialEq)]
pub struct PickHit {
    /// The node that drew the mesh hit, such as an [`Object`](crate::Object).
    pub node: NodeId,
    /// The nodes from the scene down to `node` that take part in picking: the pointer handlers
    /// around it, outermost first, and `node` last.
    pub path: Vec<NodeId>,
    /// Where the pointer hit, in world space.
    pub position: Vector3<f32>,
    /// The surface's normal there, in world space, facing the camera; `None` where the pixels
    /// around are too few to tell.
    pub normal: Option<Vector3<f32>>,
    /// The distance from the camera to `position`.
    pub distance: f32,
}

/// A pointer event, as delivered to the nodes on a hit's path.
#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent3d {
    /// The pointer moved onto the node's subtree.
    Enter(PickHit),
    /// The pointer left the node's subtree.
    Leave,
    /// The node's subtree was clicked: pressed and released on the same node without dragging.
    Click(PickHit),
}

/// What a frame recorded for picking: for each pick id, the path of the meshes drawn with it.
#[derive(Debug, Default)]
pub struct PickRecorder {
    /// The paths, indexed by pick id - 1.
    paths: Vec<Vec<NodeId>>,
    /// The current path while rendering.
    stack: Vec<NodeId>,
    /// How many pointer handlers rendered this frame.
    pub(crate) listeners: usize,
    /// How many of them listen for hover: without any, the view doesn't pick on pointer moves or
    /// each frame, only on presses and clicks.
    pub(crate) hover_listeners: usize,
}

impl PickRecorder {
    pub(crate) fn clear(&mut self) {
        self.paths.clear();
        self.stack.clear();
        self.listeners = 0;
        self.hover_listeners = 0;
    }

    pub(crate) fn push(&mut self, node: NodeId) {
        self.stack.push(node);
    }

    pub(crate) fn pop(&mut self) {
        self.stack.pop();
    }

    /// The pick id for a mesh drawn now: 1 and up.
    pub(crate) fn record(&mut self) -> u32 {
        if self.paths.last() != Some(&self.stack) {
            self.paths.push(self.stack.clone());
        }
        self.paths.len() as u32
    }

    /// The path recorded for `pick_id`.
    pub(crate) fn path(&self, pick_id: u32) -> Option<&Vec<NodeId>> {
        self.paths.get(pick_id.checked_sub(1)? as usize)
    }
}

/// Picking settings on widgets, for every 3D scene inside them.
pub trait PickingExt: Widget + Sized {
    /// How far from the pointer, in logical pixels, picks in scenes inside this widget reach when
    /// the pixel under it is empty. 2 by default; at most 7.
    fn pick_tolerance<S: IntoReadState<f32>>(self, tolerance: S) -> impl Widget<Kind=Self::Kind> {
        EnvUpdatingNew2::<Self, PickTolerance, S::Output>::new(tolerance.into_read_state(), self)
    }
}

impl<W: Widget> PickingExt for W {}

/// What a pointer handler node does with the events that reach it.
pub trait PointerHandler: Clone + 'static {
    fn handle(&mut self, event: &PointerEvent3d, env: &mut Environment);

    /// Whether the handler listens for hover: only then does the view pick as the pointer moves.
    fn hears_hover(&self) -> bool;
}

/// Calls a closure with each click's hit.
#[derive(Clone)]
pub struct ClickHandler<F: FnMut(&PickHit, &mut Environment) + Clone + 'static>(pub(crate) F);

impl<F: FnMut(&PickHit, &mut Environment) + Clone + 'static> PointerHandler for ClickHandler<F> {
    fn handle(&mut self, event: &PointerEvent3d, env: &mut Environment) {
        if let PointerEvent3d::Click(hit) = event {
            (self.0)(hit, env);
        }
    }

    fn hears_hover(&self) -> bool {
        false
    }
}

/// Calls a closure when the pointer enters, with the hit, and leaves, with `None`.
#[derive(Clone)]
pub struct HoverHandler<F: FnMut(Option<&PickHit>, &mut Environment) + Clone + 'static>(pub(crate) F);

impl<F: FnMut(Option<&PickHit>, &mut Environment) + Clone + 'static> PointerHandler for HoverHandler<F> {
    fn handle(&mut self, event: &PointerEvent3d, env: &mut Environment) {
        match event {
            PointerEvent3d::Enter(hit) => (self.0)(Some(hit), env),
            PointerEvent3d::Leave => (self.0)(None, env),
            PointerEvent3d::Click(_) => {}
        }
    }

    fn hears_hover(&self) -> bool {
        true
    }
}

/// Sets a state to whether the pointer is over the subtree.
#[derive(Clone)]
pub struct HoveredState(pub(crate) Box<dyn AnyState<T=bool>>);

impl PointerHandler for HoveredState {
    fn handle(&mut self, event: &PointerEvent3d, env: &mut Environment) {
        match event {
            PointerEvent3d::Enter(_) => {
                self.0.sync(env);
                self.0.set_value(true);
            }
            PointerEvent3d::Leave => {
                self.0.sync(env);
                self.0.set_value(false);
            }
            PointerEvent3d::Click(_) => {}
        }
    }

    fn hears_hover(&self) -> bool {
        true
    }
}

impl<F: FnMut(&PickHit, &mut Environment) + Clone + 'static> Debug for ClickHandler<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("ClickHandler")
    }
}

impl<F: FnMut(Option<&PickHit>, &mut Environment) + Clone + 'static> Debug for HoverHandler<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("HoverHandler")
    }
}

impl Debug for HoveredState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("HoveredState")
    }
}

/// A node with a pointer handler: what `.on_click(..)`, `.on_hover(..)` and `.hovered(..)` wrap a
/// node in. Its subtree is pickable, and the events whose path goes through it reach the handler.
#[derive(Debug, Clone)]
pub struct WithPointerHandler<N, H> where N: AnyNode3d + Clone, H: PointerHandler + Debug {
    id: NodeId,
    child: N,
    handler: H,
}

impl<N: AnyNode3d + Clone, H: PointerHandler + Debug> WithPointerHandler<N, H> {
    pub fn new(child: N, handler: H) -> WithPointerHandler<N, H> {
        WithPointerHandler { id: NodeId::new(), child, handler }
    }
}

impl<N: AnyNode3d + Clone, H: PointerHandler + Debug> Render3d for WithPointerHandler<N, H> {
    fn render(&mut self, context: &mut RenderContext3d) {
        context.picking.listeners += 1;
        if self.handler.hears_hover() {
            context.picking.hover_listeners += 1;
        }
        let child = &mut self.child;
        context.pick_node(self.id, |context| {
            // A handler makes its subtree pickable, unless something inside says otherwise.
            context.with_env::<Pickable, _>(&true, |context| child.render(context));
        });
    }
}

impl<N: AnyNode3d + Clone, H: PointerHandler + Debug> CommonNode3d for WithPointerHandler<N, H> {
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
        f(&self.child)
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        f(&mut self.child)
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        f(&mut self.child)
    }

    fn process_pointer_event(&mut self, event: &PointerEvent3d, env: &mut Environment) {
        self.handler.handle(event, env);
    }
}
