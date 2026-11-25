//! Loading the playground's OBJ model.

use carbide::d3::Mesh;
use carbide::math::{Matrix4, Vector3};

/// The meshes of the OBJ file at `path`, and a transform that centres them on the origin and
/// scales them to fit in a unit cube.
pub fn load_obj(path: &str) -> (Vec<Mesh>, Matrix4<f32>) {
    let meshes = Mesh::load_obj(path)
        .unwrap_or_else(|error| panic!("the OBJ file at {path} to load: {error}"));

    let positions = meshes.iter().flat_map(|mesh| mesh.vertices().iter().map(|vertex| vertex.position));
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
