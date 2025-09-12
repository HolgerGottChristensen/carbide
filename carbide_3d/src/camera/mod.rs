mod camera;
pub mod camera_projection;
mod simple_camera;
mod camera_spec;

#[cfg(feature = "dolly")]
mod dolly_camera;
mod orbit_camera;

pub use camera::Camera;
//pub use simple_camera::SimpleCamera;
pub use orbit_camera::OrbitCamera;
pub use camera_spec::CameraSpec;

//#[cfg(feature = "dolly")]
//pub use dolly_camera::DollyCamera;

#[cfg(feature = "dolly")]
pub mod dolly {
    pub use dolly::*;
}