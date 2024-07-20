
struct Camera {
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    orig_view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    aspect_ratio: f32,
}