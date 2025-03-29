use carbide::math::Matrix4;
use crate::camera::camera_projection::CameraProjection;
use crate::camera::Camera;
use carbide::state::ReadState;

#[derive(Debug, Clone)]
pub struct SimpleCamera<V> where V: ReadState<T=Matrix4<f32>> {
    pub projection: CameraProjection,
    pub view: V,
}

impl<V: ReadState<T=Matrix4<f32>>> Camera for SimpleCamera<V> {
    fn view(&self) -> Matrix4<f32> {
        *self.view.value()
    }

    fn projection(&self) -> CameraProjection {
        self.projection
    }
}