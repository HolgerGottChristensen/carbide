//! A node that sets an environment value for everything inside it: what the shadow and outline
//! modifiers wrap a node in, so their settings are inherited like styles.

use std::fmt::Debug;
use std::marker::PhantomData;

use carbide::environment::{EnvironmentKey, Shared};
use carbide::math::{Matrix4, SquareMatrix};
use carbide::state::{AnyReadState, IntoReadState, ReadState, StateContract};

use crate::node3d::{AnyNode3d, CommonNode3d, NodeId};
use crate::render::{Render3d, RenderContext3d};

/// A node with the environment key `K` set to a state's value, for itself and everything inside
/// it. The nearest setting wins.
#[derive(Debug, Clone)]
pub struct WithEnvironment<N, K> where N: AnyNode3d + Clone, K: EnvironmentKey<Access=Shared> + Clone + Debug, K::Value: StateContract {
    id: NodeId,
    child: N,
    value: Box<dyn AnyReadState<T=K::Value>>,
    key: PhantomData<K>,
}

impl<N: AnyNode3d + Clone, K: EnvironmentKey<Access=Shared> + Clone + Debug> WithEnvironment<N, K> where K::Value: StateContract {
    pub fn new<S: IntoReadState<K::Value>>(child: N, value: S) -> WithEnvironment<N, K> {
        WithEnvironment {
            id: NodeId::new(),
            child,
            value: Box::new(value.into_read_state()),
            key: PhantomData,
        }
    }
}

impl<N: AnyNode3d + Clone, K: EnvironmentKey<Access=Shared> + Clone + Debug> Render3d for WithEnvironment<N, K> where K::Value: StateContract {
    fn render(&mut self, context: &mut RenderContext3d) {
        self.value.sync(context.env);
        let value = self.value.value().clone();

        let render = &mut *context.render;
        let child = &mut self.child;
        context.env.with::<K>(&value, |env| {
            child.render(&mut RenderContext3d { render, env });
        });
    }
}

impl<N: AnyNode3d + Clone, K: EnvironmentKey<Access=Shared> + Clone + Debug> CommonNode3d for WithEnvironment<N, K> where K::Value: StateContract {
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
