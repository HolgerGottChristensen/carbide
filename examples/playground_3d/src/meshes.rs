//! Loading the playground's OBJ model, and painting meshes with vertex colours.

use carbide::color::rgba;
use carbide::d3::material::pbr_material::PbrMaterial;
use carbide::d3::{Mesh, Vertex};
use carbide::math::{Matrix4, Vector3};

/// The meshes of the OBJ file at `path`, each with its material from the file's MTL library, and a
/// transform that centres them on the origin and scales them to fit in a unit cube.
pub fn load_obj(path: &str) -> (Vec<(Mesh, PbrMaterial)>, Matrix4<f32>) {
    let parts = Mesh::load_obj_with_materials(path)
        .unwrap_or_else(|error| panic!("the OBJ file at {path} to load: {error}"));

    let positions = parts.iter().flat_map(|(mesh, _)| mesh.vertices().iter().map(|vertex| vertex.position));
    let (min, max) = positions.fold(
        (Vector3::new(f32::MAX, f32::MAX, f32::MAX), Vector3::new(f32::MIN, f32::MIN, f32::MIN)),
        |(min, max), p| (
            Vector3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z)),
            Vector3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z)),
        ),
    );

    let center = (min + max) / 2.0;
    let size = (max - min).x.max((max - min).y).max((max - min).z).max(f32::EPSILON);

    (parts, Matrix4::from_scale(1.0 / size) * Matrix4::from_translation(-center))
}

/// `mesh` with its vertices coloured by where they are in its bounds: red along X, green along Y
/// and blue along Z. The colours only show with a material that uses vertex colours.
pub fn painted(mesh: Mesh) -> Mesh {
    let (min, max) = mesh.vertices().iter().fold(
        (Vector3::new(f32::MAX, f32::MAX, f32::MAX), Vector3::new(f32::MIN, f32::MIN, f32::MIN)),
        |(min, max), v| {
            let p = v.position;
            (Vector3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z)), Vector3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z)))
        },
    );
    let size = max - min;
    let fraction = |value: f32, min: f32, size: f32| if size > f32::EPSILON { (value - min) / size } else { 0.5 };

    let vertices = mesh.vertices().iter().map(|vertex| {
        let p = vertex.position;
        Vertex {
            color_0: rgba(fraction(p.x, min.x, size.x), fraction(p.y, min.y, size.y), fraction(p.z, min.z, size.z), 1.0),
            ..*vertex
        }
    }).collect();

    Mesh::new(vertices, mesh.indices().to_vec())
}
