use carbide::render::matrix::{Matrix4, Vector3};

/// Describes how the world should be projected into the camera.
#[derive(Debug, Copy, Clone)]
pub enum CameraProjection {
    Orthographic {
        /// Size assumes the location is at the center of the camera area.
        size: Vector3<f32>,
    },
    Perspective {
        /// Vertical field of view in degrees.
        vfov: f32,
        /// Near plane distance. All projection uses a infinite far plane.
        near: f32,
    },
    Raw(Matrix4<f32>),
}

impl Default for CameraProjection {
    fn default() -> Self {
        Self::Perspective { vfov: 60.0, near: 0.1 }
    }
}