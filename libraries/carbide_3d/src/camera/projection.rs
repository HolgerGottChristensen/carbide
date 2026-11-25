use carbide::impl_state_value;
use carbide::math::{Deg, Matrix4, Rad, Vector4};

/// How a [`Camera`](crate::camera::Camera) projects the view onto the layer.
///
/// Depth is reverse-Z: near things get greater depth than far things.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Projection {
    /// A perspective projection with an infinite far plane.
    Perspective {
        /// Vertical field of view in degrees.
        vfov: f32,
        /// Distance to the near plane. Nothing nearer is drawn.
        near: f32,
    },
    /// An orthographic projection of `height` world units, with the width following the layer's
    /// aspect ratio. It draws what is in front of the camera, up to [`ORTHOGRAPHIC_DEPTH`] away.
    Orthographic {
        height: f32,
    },
    /// A projection matrix used as it is.
    Raw(Matrix4<f32>),
}

/// How far in front of an orthographic camera things are drawn.
pub const ORTHOGRAPHIC_DEPTH: f32 = 1000.0;

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective { vfov: 60.0, near: 0.1 }
    }
}

impl_state_value!(Projection);

impl Projection {
    /// The projection matrix for a layer with the given width divided by height.
    pub fn matrix(&self, aspect_ratio: f32) -> Matrix4<f32> {
        match *self {
            Projection::Perspective { vfov, near } => {
                perspective_infinite_reverse_lh(Deg(vfov), aspect_ratio, near)
            }
            Projection::Orthographic { height } => {
                let half_height = height * 0.5;
                let half_width = half_height * aspect_ratio;
                // Near and far swapped, so the projection is reverse-Z like the perspective one:
                // depth 1 at the camera, 0 at `ORTHOGRAPHIC_DEPTH` in front of it.
                orthographic_lh(-half_width, half_width, -half_height, half_height, ORTHOGRAPHIC_DEPTH, 0.0)
            }
            Projection::Raw(matrix) => matrix,
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
