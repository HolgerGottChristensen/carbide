//! The models the playground shows.

use carbide::color::WHITE;
use carbide::d3::{Mesh, Vertex};
use carbide::math::{InnerSpace, Matrix4, Vector3};

/// A cube centred on the origin with sides of `size`, with flat normals: four vertices per face,
/// wound clockwise as seen from outside the cube. Vertex colours are white.
pub fn cube(size: f32) -> Mesh {
    let half = size / 2.0;
    let normals: [Vector3<f32>; 6] = [
        Vector3::unit_x(), -Vector3::unit_x(),
        Vector3::unit_y(), -Vector3::unit_y(),
        Vector3::unit_z(), -Vector3::unit_z(),
    ];

    let mut vertices = vec![];
    let mut indices = vec![];

    for normal in normals {
        // Seen from outside, looking along -normal (left-handed: right = up × view direction).
        let up = if normal.y.abs() > 0.5 { Vector3::unit_z() } else { Vector3::unit_y() };
        let right = up.cross(-normal).normalize();

        let center = normal * half;
        let corners = [
            center + (-right + up) * half,
            center + (right + up) * half,
            center + (right - up) * half,
            center + (-right - up) * half,
        ];

        let first = vertices.len() as u32;
        for corner in corners {
            let mut vertex = Vertex::new(corner).normal(normal);
            vertex.color_0 = WHITE;
            vertex.color_1 = WHITE;
            vertices.push(vertex);
        }

        indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    }

    Mesh::new(vertices, indices)
}

/// The meshes of the OBJ file at `path`, with white vertex colours, and a transform that centres
/// them on the origin and scales them to fit in a unit cube.
pub fn load_obj(path: &str) -> (Vec<Mesh>, Matrix4<f32>) {
    let (models, _materials) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)
        .unwrap_or_else(|error| panic!("the OBJ file at {path} to load: {error}"));

    let meshes = models.into_iter().map(|model| {
        let mut mesh = Mesh::from(model);
        for vertex in &mut mesh.vertices {
            vertex.color_0 = WHITE;
            vertex.color_1 = WHITE;
        }
        mesh
    }).collect::<Vec<_>>();

    let positions = meshes.iter().flat_map(|mesh| mesh.vertices.iter().map(|vertex| vertex.position));
    let (min, max) = positions.fold(
        (Vector3::new(f32::MAX, f32::MAX, f32::MAX), Vector3::new(f32::MIN, f32::MIN, f32::MIN)),
        |(min, max), p| (
            Vector3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z)),
            Vector3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z)),
        ),
    );

    let center = (min + max) / 2.0;
    let size = (max - min).x.max((max - min).y).max((max - min).z).max(f32::EPSILON);

    (meshes, Matrix4::from_scale(1.0 / size) * Matrix4::from_translation(-center))
}
