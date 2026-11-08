mod test_scene;
mod harness;
mod snapshot;

pub use harness::Harness;
pub use snapshot::SnapshotOptions;
// Re-exported here (its own module is crate-private) since it's a parameter of the public
// `Harness::new_with_options`.
pub use crate::wgpu_msaa::WgpuMsaa;
