use carbide::draw::Color;
use carbide::color::WHITE;
use carbide::math::{Vector2, Vector3, Vector4, Zero};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    /// The direction of increasing u along the surface (x, y, z), and the bitangent's sign (w): the
    /// image's up, towards decreasing v, is `cross(normal, tangent.xyz) · w`, as glTF defines it.
    /// w is -1 where a model's texture is mirrored. Zero (with w 1) when the mesh has no
    /// tangents.
    pub tangent: Vector4<f32>,
    pub texture_coords_0: Vector2<f32>,
    pub texture_coords_1: Vector2<f32>,
    pub color_0: Color,
    pub color_1: Color,
}

impl Vertex {
    /// A vertex at `position`, with no normal, tangent or texture coordinates, and white vertex
    /// colours.
    pub fn new(position: Vector3<f32>) -> Vertex {
        Vertex {
            position,
            normal: Vector3::zero(),
            tangent: Vector4::new(0.0, 0.0, 0.0, 1.0),
            texture_coords_0: Vector2::zero(),
            texture_coords_1: Vector2::zero(),
            color_0: WHITE,
            color_1: WHITE,
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