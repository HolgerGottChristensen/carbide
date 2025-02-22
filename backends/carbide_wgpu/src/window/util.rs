use carbide_core::draw::{Dimension, Scalar};
use carbide_core::math::{ortho, Matrix4, Vector3};

pub fn calculate_carbide_to_wgpu_matrix(
    dimension: Dimension,
    scale_factor: Scalar,
) -> Matrix4<f32> {
    let half_height = dimension.height / 2.0;
    let scale = (scale_factor / half_height) as f32;

    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );

    let pixel_to_points: [[f32; 4]; 4] = [
        [scale, 0.0, 0.0, 0.0],
        [0.0, -scale, 0.0, 0.0],
        [0.0, 0.0, scale, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    let aspect_ratio = (dimension.width / dimension.height) as f32;

    let ortho = ortho(
        -1.0 * aspect_ratio,
        1.0 * aspect_ratio,
        -1.0,
        1.0,
        1.0,
        -1.0,
    );
    let res = OPENGL_TO_WGPU_MATRIX
        * ortho
        * Matrix4::from_translation(Vector3::new(-aspect_ratio, 1.0, 0.0))
        * Matrix4::from(pixel_to_points);
    res
}