use carbide::draw::Color;
use carbide::render::matrix::{Vector2, Vector3, Vector4, Zero};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub texture_coords_0: Vector2<f32>,
    pub texture_coords_1: Vector2<f32>,
    pub color_0: Color,
    pub color_1: Color,
}

impl Vertex {
    pub fn new(position: Vector3<f32>) -> Vertex {
        Vertex {
            position,
            normal: Vector3::zero(),
            tangent: Vector3::zero(),
            texture_coords_0: Vector2::zero(),
            texture_coords_1: Vector2::zero(),
            color_0: Color::random(),
            color_1: Color::random(),
        }
    }

    pub fn normal(mut self, normal: Vector3<f32>) -> Vertex {
        self.normal = normal;
        self
    }

    pub fn texture_coords_0(mut self, coords: Vector2<f32>) -> Vertex {
        self.texture_coords_0 = coords;
        self
    }
}