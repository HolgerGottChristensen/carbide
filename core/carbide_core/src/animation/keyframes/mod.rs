//! Composable keyframe/timeline animation engine, additive to the rest of
//! [`crate::animation`]. Not re-exported into `animation::*` — opt in via
//! `carbide_core::animation::keyframes::{...}` to minimize collision surface
//! (see the naming note on [`Chain`] in particular).

pub use timeline::{ResolveMode, Timeline, TimelineStatus};
pub use keyframes::Keyframes;
pub use delay::Delay;
pub use chain::Chain;
pub use tracks::Tracks;
pub use repeat::Repeat;
pub use controller::{RunState, TimelineController, TimelineHandle};

mod timeline;
mod keyframes;
mod delay;
mod chain;
mod tracks;
mod repeat;
mod controller;
