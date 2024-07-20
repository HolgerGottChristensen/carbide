use carbide::render::matrix::Matrix4;
use crate::camera::Camera;
use crate::camera::camera_projection::CameraProjection;

#[derive(Debug, Clone)]
pub struct SimpleCamera {
    pub projection: CameraProjection,
    pub view: Matrix4<f32>,
}

impl Camera for SimpleCamera {
    fn view(&self) -> Matrix4<f32> {
        self.view
    }

    fn projection(&self) -> CameraProjection {
        self.projection
    }
}