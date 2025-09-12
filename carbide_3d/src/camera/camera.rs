use crate::camera::camera_projection::CameraProjection;
use std::fmt::Debug;
use carbide_core::time::Instant;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::environment::Environment;
use carbide::event::EventHandler;
use carbide::lifecycle::Update;
use carbide::math::{Deg, Matrix4, Rad, Vector4};
use carbide::render::Layer;
use crate::camera::CameraSpec;

pub trait Camera: Debug + Clone + EventHandler + Update + 'static {
    fn view(&self) -> Matrix4<f32>;
    fn projection(&self) -> CameraProjection;
    fn view_projection(&self, aspect_ratio: f32) -> Matrix4<f32> {
        self.projection_matrix(aspect_ratio) * self.view()
    }

    fn projection_matrix(&self, aspect_ratio: f32) -> Matrix4<f32> {
        match self.projection() {
            CameraProjection::Orthographic { size } => {
                let half = size * 0.5;
                orthographic_lh(-half.x, half.x, -half.y, half.y, half.z, -half.z)
            }
            CameraProjection::Perspective { vfov, near } => {
                perspective_infinite_reverse_lh(Deg(vfov), aspect_ratio, near)
            }
            CameraProjection::Raw(proj) => proj,
        }
    }

    fn to_spec(&self, layer: &Layer) -> CameraSpec {

        let aspect_ratio = layer.dimensions().0 as f32 / layer.dimensions().1 as f32;

        CameraSpec {
            view: self.view(),
            view_projection: self.view_projection(aspect_ratio),
        }
    }
}

pub fn perspective_infinite_reverse_lh<T: Into<Rad<f32>>>(
    fov_y: T,
    aspect_ratio: f32,
    z_near: f32,
) -> Matrix4<f32> {
    assert!(z_near > 0.0);
    let fov_y_radians = fov_y.into().0;
    let (sin_fov, cos_fov) = f32::sin_cos(0.5 * fov_y_radians);
    let h = cos_fov / sin_fov;
    let w = h / aspect_ratio;
    Matrix4::from_cols(
        Vector4::new(w, 0.0, 0.0, 0.0),
        Vector4::new(0.0, h, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(0.0, 0.0, z_near, 0.0),
    )
}

pub fn perspective_infinite_lh<T: Into<Rad<f32>>>(fov_y: T, aspect_ratio: f32, z_near: f32) -> Matrix4<f32> {
    let fov_y_radians = fov_y.into().0;

    let (sin_fov, cos_fov) = f32::sin_cos(0.5 * fov_y_radians);
    let h = cos_fov / sin_fov;
    let w = h / aspect_ratio;
    Matrix4::from_cols(
        Vector4::new(w, 0.0, 0.0, 0.0),
        Vector4::new(0.0, h, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 1.0),
        Vector4::new(0.0, 0.0, -z_near, 0.0),
    )
}

pub fn perspective_infinite_rh<T: Into<Rad<f32>>>(fov_y: T, aspect_ratio: f32, z_near: f32) -> Matrix4<f32> {
    let fov_y_radians = fov_y.into().0;
    let f = 1.0 / f32::tan(0.5 * fov_y_radians);
    Matrix4::from_cols(
        Vector4::new(f / aspect_ratio, 0.0, 0.0, 0.0),
        Vector4::new(0.0, f, 0.0, 0.0),
        Vector4::new(0.0, 0.0, -1.0, -1.0),
        Vector4::new(0.0, 0.0, -z_near, 0.0),
    )
}

pub fn orthographic_lh(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Matrix4<f32> {
    let rcp_width = 1.0 / (right - left);
    let rcp_height = 1.0 / (top - bottom);
    let r = 1.0 / (far - near);
    Matrix4::<f32>::from_cols(
        Vector4::new(rcp_width + rcp_width, 0.0, 0.0, 0.0),
        Vector4::new(0.0, rcp_height + rcp_height, 0.0, 0.0),
        Vector4::new(0.0, 0.0, r, 0.0),
        Vector4::new(
            -(left + right) * rcp_width,
            -(top + bottom) * rcp_height,
            -r * near,
            1.0,
        ),
    )
}