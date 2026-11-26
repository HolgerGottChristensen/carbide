//! Turning shadows on and off for parts of a scene, inherited through the environment like the
//! styles of carbide's controls.
//!
//! [`CastsShadows`] and [`ReceivesShadows`] are environment keys. They can be set on any 3D node,
//! with `.casts_shadows(..)` and `.receives_shadows(..)` from [`Node3dExt`](crate::Node3dExt), for
//! the node and everything inside it, or on any widget, with [`ShadowsExt`], for every scene below
//! it. The nearest setting wins. Where nothing is set, directional and spot lights cast shadows,
//! and objects cast and receive them.
//!
//! So `.casts_shadows(false)` on a `Scene3d`, or on any widget around it, turns shadows off for
//! the scene; on a light, it saves that light's shadow pass; on a group, it stops the objects in it
//! from casting.

use std::marker::PhantomData;

use carbide::environment::{EnvironmentKey, Shared};
use carbide::math::{Matrix4, SquareMatrix};
use carbide::state::{AnyReadState, IntoReadState, ReadState, StateSync};
use carbide::widget::{EnvUpdatingNew2, Widget};

use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render::{Render3d, RenderContext3d};

/// Whether lights cast shadows, and objects cast them, in this part of the tree.
#[derive(Debug, Copy, Clone)]
pub struct CastsShadows;

impl EnvironmentKey for CastsShadows {
    type Value = bool;
    type Access = Shared;
}

/// Whether shadows fall on objects in this part of the tree.
#[derive(Debug, Copy, Clone)]
pub struct ReceivesShadows;

impl EnvironmentKey for ReceivesShadows {
    type Value = bool;
    type Access = Shared;
}

/// A node with a shadow setting for itself and everything inside it: what `.casts_shadows(..)`
/// and `.receives_shadows(..)` wrap a node in.
#[derive(Debug, Clone)]
pub struct WithShadowSetting<N, K> where N: AnyNode3d + Clone, K: EnvironmentKey<Value=bool, Access=Shared> + Clone + std::fmt::Debug {
    id: NodeId,
    child: N,
    value: Box<dyn AnyReadState<T=bool>>,
    key: PhantomData<K>,
}

impl<N: AnyNode3d + Clone, K: EnvironmentKey<Value=bool, Access=Shared> + Clone + std::fmt::Debug> WithShadowSetting<N, K> {
    pub fn new<S: IntoReadState<bool>>(child: N, value: S) -> WithShadowSetting<N, K> {
        WithShadowSetting {
            id: NodeId::new(),
            child,
            value: Box::new(value.into_read_state()),
            key: PhantomData,
        }
    }
}

impl<N: AnyNode3d + Clone, K: EnvironmentKey<Value=bool, Access=Shared> + Clone + std::fmt::Debug> Render3d for WithShadowSetting<N, K> {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.value.sync(context.env);
        let value = *self.value.value();

        let render = &mut *context.render;
        let child = &mut self.child;
        context.env.with::<K>(&value, |env| {
            child.render(&mut RenderContext3d { render, env });
        });
    }
}

impl<N: AnyNode3d + Clone, K: EnvironmentKey<Value=bool, Access=Shared> + Clone + std::fmt::Debug> CommonNode3d for WithShadowSetting<N, K> {
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
}

/// Shadow settings on widgets, for every 3D scene inside them.
pub trait ShadowsExt: Widget + Sized {
    /// Whether lights and objects in scenes inside this widget cast shadows. See the
    /// [module](crate::shadows) for the defaults.
    fn casts_shadows<S: IntoReadState<bool>>(self, casts: S) -> impl Widget<Kind=Self::Kind> {
        EnvUpdatingNew2::<Self, CastsShadows, S::Output>::new(casts.into_read_state(), self)
    }

    /// Whether shadows fall on objects in scenes inside this widget.
    fn receives_shadows<S: IntoReadState<bool>>(self, receives: S) -> impl Widget<Kind=Self::Kind> {
        EnvUpdatingNew2::<Self, ReceivesShadows, S::Output>::new(receives.into_read_state(), self)
    }
}

impl<W: Widget> ShadowsExt for W {}
