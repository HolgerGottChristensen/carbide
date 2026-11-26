//! Outlining parts of a scene, for example to show what is selected, inherited through the
//! environment like the shadow settings and carbide's control styles.
//!
//! [`Outlined`] turns outlines on for the node it is set on and everything inside it, with
//! `.outlined(..)` from [`Node3dExt`](crate::Node3dExt). [`OutlineColor`] and
//! [`OutlineThickness`] style them, with `.outline_color(..)` and `.outline_thickness(..)`. All three
//! can also be set on any widget, with [`OutlineExt`], for every scene inside it. The nearest
//! setting wins. Where nothing is set, nothing is outlined; the colour is the environment's accent
//! colour, and the thickness 2 logical pixels.
//!
//! An outline is a band of the colour and thickness around the silhouette of everything outlined,
//! as seen by the camera, so a group of touching objects gets one outline around them all. It is
//! drawn on top of the scene: it shows even where the outlined objects are behind others.

use carbide::color::{Color, BLUE};
use carbide::environment::{EnvironmentColor, EnvironmentKey, EnvironmentKeyable, Environment, Shared};
use carbide::state::IntoReadState;
use carbide::widget::{EnvUpdatingNew2, Widget};

use crate::render::Outline;

/// Whether this part of the tree is outlined.
#[derive(Debug, Copy, Clone)]
pub struct Outlined;

impl EnvironmentKey for Outlined {
    type Value = bool;
    type Access = Shared;
}

/// The colour of outlines in this part of the tree.
#[derive(Debug, Copy, Clone)]
pub struct OutlineColor;

impl EnvironmentKey for OutlineColor {
    type Value = Color;
    type Access = Shared;
}

/// The thickness of outlines in this part of the tree, in logical pixels.
#[derive(Debug, Copy, Clone)]
pub struct OutlineThickness;

impl EnvironmentKey for OutlineThickness {
    type Value = f32;
    type Access = Shared;
}

/// The outline for meshes rendered in `env`, or `None` where they aren't outlined.
pub(crate) fn outline_in(env: &Environment) -> Option<Outline> {
    if !env.get::<Outlined>().copied().unwrap_or(false) {
        return None;
    }

    let color = env.get::<OutlineColor>().copied()
        .or_else(|| EnvironmentColor::Accent.get(env))
        .unwrap_or(BLUE);
    let thickness = env.get::<OutlineThickness>().copied().unwrap_or(2.0).max(0.0);

    Some(Outline { color, thickness })
}

/// Outline settings on widgets, for every 3D scene inside them.
pub trait OutlineExt: Widget + Sized {
    /// Whether the nodes of scenes inside this widget are outlined.
    fn outlined<S: IntoReadState<bool>>(self, outlined: S) -> impl Widget<Kind=Self::Kind> {
        EnvUpdatingNew2::<Self, Outlined, S::Output>::new(outlined.into_read_state(), self)
    }

    /// The colour of outlines in scenes inside this widget.
    fn outline_color<S: IntoReadState<Color>>(self, color: S) -> impl Widget<Kind=Self::Kind> {
        EnvUpdatingNew2::<Self, OutlineColor, S::Output>::new(color.into_read_state(), self)
    }

    /// The thickness of outlines in scenes inside this widget, in logical pixels.
    fn outline_thickness<S: IntoReadState<f32>>(self, thickness: S) -> impl Widget<Kind=Self::Kind> {
        EnvUpdatingNew2::<Self, OutlineThickness, S::Output>::new(thickness.into_read_state(), self)
    }
}

impl<W: Widget> OutlineExt for W {}
