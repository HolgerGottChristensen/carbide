use std::sync::atomic::{AtomicBool, Ordering};
use ctor::ctor;
pub use carbide_core::*;

#[cfg(feature = "carbide_macro")]
pub use carbide_macro::*;

#[cfg(feature = "default")]
pub use carbide_wgpu::*;

#[cfg(feature = "media")]
pub use carbide_media::*;

#[cfg(feature = "i18n")]
pub use carbide_fluent::*;

#[cfg(feature = "carbide_controls")]
pub mod controls {
    pub use carbide_controls::*;
}

#[cfg(feature = "carbide_3d")]
pub use carbide_3d::*;

pub fn init() {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);
    if !INITIALIZED.swap(true, Ordering::Relaxed) {
        #[cfg(feature = "carbide_wgpu_3d")]
        carbide_wgpu_3d::init()
    }
}