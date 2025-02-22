extern crate carbide_core as carbide;
mod scene3d;
mod render_context3d;
mod vertex;
mod handedness;
mod mesh;
mod object;
pub mod sorting;
pub mod material;
mod render3d;
pub mod camera;
mod transform;
pub mod light;
pub mod node3d;
mod node3d_sequence;
mod image_context3d;

use dashmap::DashMap;
pub use handedness::*;
pub use image_context3d::*;
pub use mesh::*;
pub use object::*;
use once_cell::sync::Lazy;
pub use render_context3d::*;
pub use scene3d::*;
pub use vertex::*;


pub(crate) static AVAILABLE_CONTEXT3D_INITIALIZERS: Lazy<DashMap<&'static str, fn()->Box<dyn InnerRenderContext3d>>> = Lazy::new(|| {
    DashMap::new()
});

pub fn register_render_context3d_initializer(identifier: &'static str, initializer: fn()->Box<dyn InnerRenderContext3d>) {
    AVAILABLE_CONTEXT3D_INITIALIZERS.insert(identifier, initializer);
}

pub(crate) fn render_context3d() -> Box<dyn InnerRenderContext3d> {
    let initializer = AVAILABLE_CONTEXT3D_INITIALIZERS.iter().next().expect("No initializers for render context 3d present");
    initializer()
}

pub(crate) static AVAILABLE_IMAGE_CONTEXT3D_INITIALIZERS: Lazy<DashMap<&'static str, fn()->Box<dyn InnerImageContext3d>>> = Lazy::new(|| {
    DashMap::new()
});

pub fn register_image_context3d_initializer(identifier: &'static str, initializer: fn()->Box<dyn InnerImageContext3d>) {
    AVAILABLE_IMAGE_CONTEXT3D_INITIALIZERS.insert(identifier, initializer);
}

pub(crate) fn image_context3d() -> Box<dyn InnerImageContext3d> {
    let initializer = AVAILABLE_IMAGE_CONTEXT3D_INITIALIZERS.iter().next().expect("No initializers for image context 3d present");
    initializer()
}