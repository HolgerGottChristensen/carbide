use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use carbide::environment::{Environment, EnvironmentKey, Shared};
use crate::render::InnerRenderContext3d;

/// Identifies a [`ContextFactory3d`]. Two factories with the same id must create equivalent
/// render contexts.
///
/// Built from a type with [`RenderContext3dId::of`], usually the render context the factory
/// creates. The factory's function can't be compared instead: the same function can have
/// different addresses, and different functions the same address.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderContext3dId(TypeId);

impl RenderContext3dId {
    /// The id for factories identified by `T`.
    pub fn of<T: ?Sized + 'static>() -> RenderContext3dId {
        RenderContext3dId(TypeId::of::<T>())
    }
}

/// Registered into the environment by a 3D renderer plugin, so a [`Scene3d`](crate::Scene3d) can
/// create a render context without depending on the renderer.
///
/// A `Scene3d` recreates its render context when the `id` of the factory in the environment
/// differs from the one that created its current context, so the renderer can be switched while
/// the application runs.
#[derive(Copy, Clone)]
pub struct ContextFactory3d {
    /// Identifies the factory. See [`RenderContext3dId`].
    pub id: RenderContext3dId,
    /// Create a render context for one `Scene3d`.
    pub render_context: fn(&mut Environment) -> Box<dyn InnerRenderContext3d>,
}

impl Debug for ContextFactory3d {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextFactory3d")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl EnvironmentKey for ContextFactory3d {
    type Value = ContextFactory3d;
    type Access = Shared;
}
