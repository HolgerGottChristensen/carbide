use carbide::math::Matrix4;

#[derive(Clone, Debug)]
pub struct CameraSpec {
    pub view: Matrix4<f32>,
    pub view_projection: Matrix4<f32>,
}