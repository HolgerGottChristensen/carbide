//! Loading the playground's OBJ models, and painting meshes with vertex colours.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use carbide::color::rgba;
use carbide::d3::material::pbr_material::PbrMaterial;
use carbide::d3::{Mesh, Vertex};
use carbide::math::{Matrix4, Vector3};

/// An OBJ model, fitted to the playground: its meshes with their materials from the MTL library,
/// a transform that centres it on the origin and scales it to fit in a unit cube, and the height
/// of its bottom after that transform, for the floor.
#[derive(Clone)]
pub struct FittedObj {
    pub parts: Vec<(Mesh, PbrMaterial)>,
    pub fit: Matrix4<f32>,
    pub bottom: f32,
}

thread_local! {
    /// The models loaded so far, by path. Meshes are shared handles and their images load once,
    /// so a cached model costs nothing to show again, when the views are rebuilt.
    static LOADED: RefCell<HashMap<String, Rc<FittedObj>>> = RefCell::new(HashMap::new());
}

/// The bounds of `mesh`'s vertices: the smallest and largest x, y and z.
fn bounds(mesh: &Mesh) -> (Vector3<f32>, Vector3<f32>) {
    mesh.vertices().iter().map(|vertex| vertex.position).fold(
        (Vector3::new(f32::MAX, f32::MAX, f32::MAX), Vector3::new(f32::MIN, f32::MIN, f32::MIN)),
        |(min, max), p| (
            Vector3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z)),
            Vector3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z)),
        ),
    )
}

/// The largest side of `bounds`.
fn extent((min, max): &(Vector3<f32>, Vector3<f32>)) -> f32 {
    let size = max - min;
    size.x.max(size.y).max(size.z)
}

/// The OBJ model at `path`, loaded the first time and cached after. With `fit_without_studio`,
/// the fit leaves out parts more than four times as large as the median part, such as the
/// backdrop and light panels of a studio scene, so the model itself fills the view; they are
/// still drawn. Only for such scenes: in a model of one large part among many small ones, the
/// large part would be left out.
pub fn load_obj(path: &str, fit_without_studio: bool) -> Rc<FittedObj> {
    if let Some(loaded) = LOADED.with(|loaded| loaded.borrow().get(path).cloned()) {
        return loaded;
    }

    let parts = Mesh::load_obj_with_materials(path)
        .unwrap_or_else(|error| panic!("the OBJ file at {path} to load: {error}"));

    let mut part_bounds: Vec<_> = parts.iter().map(|(mesh, _)| bounds(mesh)).filter(|(min, max)| min.x <= max.x).collect();
    if fit_without_studio && !part_bounds.is_empty() {
        let mut extents: Vec<f32> = part_bounds.iter().map(extent).collect();
        extents.sort_by(f32::total_cmp);
        let median = extents[extents.len() / 2];
        part_bounds.retain(|bounds| extent(bounds) <= 4.0 * median);
    }
    let (min, max) = part_bounds.iter().fold(
        (Vector3::new(f32::MAX, f32::MAX, f32::MAX), Vector3::new(f32::MIN, f32::MIN, f32::MIN)),
        |(min, max), (a, b)| (
            Vector3::new(min.x.min(a.x), min.y.min(a.y), min.z.min(a.z)),
            Vector3::new(max.x.max(b.x), max.y.max(b.y), max.z.max(b.z)),
        ),
    );

    let center = (min + max) / 2.0;
    let size = (max - min).x.max((max - min).y).max((max - min).z).max(f32::EPSILON);

    let fitted = Rc::new(FittedObj {
        parts,
        fit: Matrix4::from_scale(1.0 / size) * Matrix4::from_translation(-center),
        bottom: (min.y - center.y) / size,
    });
    LOADED.with(|loaded| loaded.borrow_mut().insert(path.to_string(), fitted.clone()));
    fitted
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
