pub use carbide_core::*;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "carbide_macro")]
pub use carbide_macro::*;

#[cfg(feature = "default")]
pub use carbide_wgpu::*;

#[cfg(feature = "carbide_controls")]
pub mod controls {
    pub use carbide_controls::*;
}

#[cfg(feature = "carbide_3d")]
pub use carbide_3d::*;

#[cfg(feature = "carbide_chart")]
pub mod chart {
    pub use carbide_chart::*;
}

#[cfg(feature = "i18n")]
pub mod i18n {
    pub use carbide_fluent::*;
}

#[cfg(feature = "carbide_dialogs")]
pub mod dialogs {
    pub use carbide_dialogs::*;
}

#[cfg(feature = "carbide_media")]
pub mod media {
    pub use carbide_media::*;
}

pub fn init() {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);
    if !INITIALIZED.swap(true, Ordering::Relaxed) {
        #[cfg(feature = "carbide_wgpu_3d")]
        carbide_wgpu_3d::init()
    }
}