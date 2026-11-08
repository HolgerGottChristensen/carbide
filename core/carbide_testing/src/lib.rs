//! Headless fixture for rendering-regression tests. A [`Harness`] builds a widget tree without
//! an OS window or event loop, drives it with synthetic input through carbide_winit's event
//! handler, renders it offscreen with carbide_wgpu, and compares the output against committed
//! snapshots.

extern crate carbide_core as carbide;

mod harness;
mod snapshot;
mod test_scene;

pub use harness::Harness;
pub use snapshot::SnapshotOptions;
pub use carbide_wgpu::testing::WgpuMsaa;
