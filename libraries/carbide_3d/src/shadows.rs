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

use carbide::environment::{EnvironmentKey, Shared};
use carbide::state::IntoReadState;
use carbide::widget::{EnvUpdatingNew2, Widget};

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
