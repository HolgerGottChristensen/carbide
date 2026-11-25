pub use carbide_core::*;

#[cfg(feature = "carbide_macro")]
pub use carbide_macro::*;

#[cfg(feature = "carbide_winit")]
pub use carbide_winit::{Application, Window};

// Also links carbide_wgpu, which registers itself as the window renderer through its plugin.
#[cfg(feature = "carbide_wgpu")]
pub use carbide_wgpu::*;

#[cfg(feature = "carbide_controls")]
pub mod controls {
    pub use carbide_controls::*;
}

#[cfg(feature = "carbide_3d")]
pub mod d3 {
    pub use carbide_3d::*;

    // Also links carbide_wgpu_3d, which registers itself as the 3D renderer through its plugin.
    #[cfg(feature = "carbide_wgpu_3d")]
    pub use carbide_wgpu_3d as wgpu;
}

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

#[cfg(feature = "carbide_table")]
pub mod table {
    pub use carbide_table::*;
}

#[cfg(feature = "carbide_icons")]
pub mod icons {
    pub use carbide_icons::*;
}

#[cfg(feature = "carbide_testing")]
pub mod testing {
    pub use carbide_testing::*;
}
