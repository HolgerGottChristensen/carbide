use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use carbide::impl_state_value;

use crate::vertex::Vertex;
use tobj::Model;
use carbide::math::{InnerSpace, Vector2, Vector3, Zero};

/// Identifies a [`Mesh`]'s data: clones of a mesh share it, and every new or changed mesh gets a
/// new one.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MeshId(u64);

impl MeshId {
    fn new() -> MeshId {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        MeshId(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// Triangles, as vertices and indices into them, three per triangle.
///
/// The renderer draws the side a triangle's vertices go round counter-clockwise, seen from that
/// side, in carbide_3d's right-handed space (+X right, +Y up, +Z towards the viewer), as OBJ and
/// glTF do. Vertex normals point out of that side.
///
/// A mesh is a cheap handle to data that never changes: cloning it shares the data, and a renderer
/// uploads the data once for every clone, in every view. A changed mesh is a new mesh, with a new
/// [`MeshId`]: the methods that change one, like [`calculate_normals`](Mesh::calculate_normals),
/// return a new mesh. To change the geometry of an [`Object`](crate::Object), give it a state of a
/// mesh and set the state to the new mesh. A renderer frees its copy once every handle to the
/// data is gone.
#[derive(Clone)]
pub struct Mesh {
    id: MeshId,
    data: Arc<MeshData>,
}

#[derive(Clone)]
struct MeshData {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

/// A reference to a [`Mesh`]'s data that doesn't keep it alive, for a renderer's cache.
#[derive(Clone)]
pub struct WeakMesh(Weak<MeshData>);

impl WeakMesh {
    /// Whether a handle to the mesh's data still exists.
    pub fn is_alive(&self) -> bool {
        self.0.strong_count() > 0
    }
}

impl Debug for WeakMesh {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakMesh").field("alive", &self.is_alive()).finish()
    }
}

impl Debug for Mesh {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("id", &self.id)
            .field("vertices", &self.data.vertices.len())
            .field("indices", &self.data.indices.len())
            .finish()
    }
}

impl_state_value!(Mesh);

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
        Mesh::from_data(MeshData {
            vertices,
            indices,
        })
    }

    fn from_data(data: MeshData) -> Mesh {
        Mesh {
            id: MeshId::new(),
            data: Arc::new(data),
        }
    }

    /// The data, to change into a new mesh: taken if this is the only handle, copied otherwise.
    fn into_data(self) -> MeshData {
        Arc::unwrap_or_clone(self.data)
    }

    /// Identifies this mesh's data: the same for all its clones.
    pub fn id(&self) -> MeshId {
        self.id
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.data.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.data.indices
    }

    /// A reference to the data that doesn't keep it alive.
    pub fn downgrade(&self) -> WeakMesh {
        WeakMesh(Arc::downgrade(&self.data))
    }

    /// A cube centred on the origin with sides of `size`, with flat normals: four vertices per
    /// face, so each face is shaded on its own.
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
            // Seen from outside, looking along -normal: pick an up direction, and the right
            // direction follows (right-handed: right = view direction × up). Texture coordinates
            // run along right (u) and down (v), so they read correctly from outside.
            let up = if normal.y.abs() > 0.5 { -Vector3::unit_z() } else { Vector3::unit_y() };
            let right = (-normal).cross(up).normalize();

            let center = normal * half;
            let corners = [
                (center + (-right + up) * half, Vector2::new(0.0, 0.0)),
                (center + (right + up) * half, Vector2::new(1.0, 0.0)),
                (center + (right - up) * half, Vector2::new(1.0, 1.0)),
                (center + (-right - up) * half, Vector2::new(0.0, 1.0)),
            ];

            let first = vertices.len() as u32;
            for (corner, coords) in corners {
                vertices.push(Vertex::new(corner).normal(normal).texture_coords_0(coords));
            }

            // Top left, bottom right, top right; top left, bottom left, bottom right: counter-
            // clockwise seen from outside.
            indices.extend([first, first + 2, first + 1, first, first + 3, first + 2]);
        }

        Mesh::new(vertices, indices).calculate_tangents()
    }

    /// A flat rectangle on the ground: `width` along X and `depth` along Z, centred on the origin,
    /// facing up (+Y).
    pub fn plane(width: f32, depth: f32) -> Mesh {
        let (x, z) = (width / 2.0, depth / 2.0);
        let up = Vector3::unit_y();

        // Counter-clockwise seen from above, with +X right and +Z towards the viewer.
        let vertices = vec![
            Vertex::new(Vector3::new(-x, 0.0, z)).normal(up).texture_coords_0(Vector2::new(0.0, 0.0)),
            Vertex::new(Vector3::new(x, 0.0, z)).normal(up).texture_coords_0(Vector2::new(1.0, 0.0)),
            Vertex::new(Vector3::new(x, 0.0, -z)).normal(up).texture_coords_0(Vector2::new(1.0, 1.0)),
            Vertex::new(Vector3::new(-x, 0.0, -z)).normal(up).texture_coords_0(Vector2::new(0.0, 1.0)),
        ];

        Mesh::new(vertices, vec![0, 1, 2, 0, 2, 3]).calculate_tangents()
    }

    /// A sphere centred on the origin with the given `radius`, made by splitting each triangle of
    /// an icosahedron into four `subdivisions` times: 20 triangles at 0, 20 · 4ⁿ at n. Each
    /// triangle has vertices and a normal of its own, so it is shaded flat.
    ///
    /// After <https://schneide.blog/2016/07/15/generating-an-icosphere-in-c/>.
    pub fn icosphere(radius: f32, subdivisions: u32) -> Mesh {
        const X: f32 = 0.525731112119133606;
        const Z: f32 = 0.850650808352039932;
        const N: f32 = 0.0;

        const VERTICES: [[f32; 3]; 12] = [
            [-X, N, Z], [X, N, Z], [-X, N, -Z], [X, N, -Z],
            [N, Z, X], [N, Z, -X], [N, -Z, X], [N, -Z, -X],
            [Z, X, N], [-Z, X, N], [Z, -X, N], [-Z, -X, N],
        ];

        // Wound the other way round from the source, so they are counter-clockwise seen from
        // outside.
        const INDICES: [[u32; 3]; 20] = [
            [0, 1, 4], [0, 4, 9], [9, 4, 5], [4, 8, 5], [4, 1, 8],
            [8, 1, 10], [8, 10, 3], [5, 8, 3], [5, 3, 2], [2, 3, 7],
            [7, 3, 10], [7, 10, 6], [7, 6, 11], [11, 6, 0], [0, 6, 1],
            [6, 10, 1], [9, 11, 0], [9, 2, 11], [9, 5, 2], [7, 11, 2],
        ];

        fn vertex_for_edge(map: &mut HashMap<(u32, u32), u32>, positions: &mut Vec<Vector3<f32>>, first: u32, second: u32) -> u32 {
            // The same midpoint for an edge in either direction.
            let key = (first.min(second), first.max(second));

            if let Some(index) = map.get(&key) {
                return *index;
            }

            let index = positions.len() as u32;
            let position = (positions[first as usize] + positions[second as usize]).normalize();
            positions.push(position);
            map.insert(key, index);

            index
        }

        let mut lookup = HashMap::new();
        let mut positions = VERTICES.iter().map(|p| Vector3::new(p[0], p[1], p[2])).collect::<Vec<_>>();
        let mut triangles = INDICES.to_vec();

        for _ in 0..subdivisions {
            let mut split = Vec::with_capacity(triangles.len() * 4);

            for triangle in triangles {
                let mut mid = [0u32; 3];

                for i in 0..3 {
                    mid[i] = vertex_for_edge(&mut lookup, &mut positions, triangle[i], triangle[(i + 1) % 3]);
                }

                split.push([triangle[0], mid[0], mid[2]]);
                split.push([triangle[1], mid[1], mid[0]]);
                split.push([triangle[2], mid[2], mid[1]]);
                split.push([mid[0], mid[1], mid[2]]);
            }

            triangles = split;
        }

        // Three vertices of its own for each triangle, so the normals are flat.
        let vertices = triangles.iter()
            .flatten()
            .map(|index| Vertex::new(positions[*index as usize] * radius))
            .collect::<Vec<_>>();
        let indices = (0..vertices.len() as u32).collect::<Vec<u32>>();

        Mesh::new(vertices, indices).calculate_normals()
    }

    /// The meshes of the OBJ file at `path`, one per object or group in it, triangulated with one
    /// index per vertex. Missing normals are calculated (flat), and missing texture coordinates
    /// are zero. OBJ materials are not loaded.
    pub fn load_obj<P: AsRef<Path>>(path: P) -> Result<Vec<Mesh>, tobj::LoadError> {
        let (models, _materials) = tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS)?;
        Ok(models.into_iter().map(Mesh::from).collect())
    }

    /// Calculate each vertex's normal from the triangles it is part of, pointing out of the side
    /// they go round counter-clockwise, and normalise it. Replaces the normals the vertices had.
    pub fn calculate_normals(self) -> Self {
        let mut data = self.into_data();

        for vertex in &mut data.vertices {
            vertex.normal = Vector3::zero();
        }

        for chunk in data.indices.chunks_exact(3) {
            let pos0 = data.vertices[chunk[0] as usize].position;
            let pos1 = data.vertices[chunk[1] as usize].position;
            let pos2 = data.vertices[chunk[2] as usize].position;

            let edge1 = pos1 - pos0;
            let edge2 = pos2 - pos0;

            let normal = edge1.cross(edge2);

            data.vertices[chunk[0] as usize].normal += normal;
            data.vertices[chunk[1] as usize].normal += normal;
            data.vertices[chunk[2] as usize].normal += normal;
        }

        for vertex in &mut data.vertices {
            if vertex.normal.magnitude2() > 0.0 {
                vertex.normal = vertex.normal.normalize();
            }
        }

        Mesh::from_data(data)
    }

    /// Calculate each vertex's tangent from its triangles' positions and first texture
    /// coordinates, made perpendicular to its normal. Triangles whose texture coordinates don't
    /// span an area add nothing; a vertex with no tangent gets zero.
    pub fn calculate_tangents(self) -> Self {
        let mut data = self.into_data();

        for vertex in &mut data.vertices {
            vertex.tangent = Vector3::zero();
        }

        for chunk in data.indices.chunks_exact(3) {
            let pos0 = data.vertices[chunk[0] as usize].position;
            let pos1 = data.vertices[chunk[1] as usize].position;
            let pos2 = data.vertices[chunk[2] as usize].position;

            let tex0 = data.vertices[chunk[0] as usize].texture_coords_0;
            let tex1 = data.vertices[chunk[1] as usize].texture_coords_0;
            let tex2 = data.vertices[chunk[2] as usize].texture_coords_0;

            let uv1 = tex1 - tex0;
            let uv2 = tex2 - tex0;

            let edge1 = pos1 - pos0;
            let edge2 = pos2 - pos0;

            let determinant = uv1.x * uv2.y - uv1.y * uv2.x;
            if determinant.abs() <= f32::EPSILON {
                continue;
            }

            let tangent = (edge1 * uv2.y - edge2 * uv1.y) / determinant;

            data.vertices[chunk[0] as usize].tangent += tangent;
            data.vertices[chunk[1] as usize].tangent += tangent;
            data.vertices[chunk[2] as usize].tangent += tangent;
        }

        for vertex in &mut data.vertices {
            let t = vertex.tangent - (vertex.normal * vertex.normal.dot(vertex.tangent));
            vertex.tangent = if t.magnitude2() > 0.0 { t.normalize() } else { Vector3::zero() };
        }

        Mesh::from_data(data)
    }
}

/// A mesh from a model loaded with tobj's `GPU_LOAD_OPTIONS` (triangulated, one index per
/// vertex). Normals the model doesn't have are calculated, and texture coordinates it doesn't
/// have are zero.
impl From<Model> for Mesh {
    fn from(value: Model) -> Self {
        let mesh = value.mesh;
        let count = mesh.positions.len() / 3;
        let has_normals = mesh.normals.len() == mesh.positions.len();
        let has_coords = mesh.texcoords.len() / 2 == count;

        let vertices = (0..count).map(|i| {
            let position = Vector3::new(mesh.positions[3 * i], mesh.positions[3 * i + 1], mesh.positions[3 * i + 2]);
            let mut vertex = Vertex::new(position);

            if has_normals {
                vertex = vertex.normal(Vector3::new(mesh.normals[3 * i], mesh.normals[3 * i + 1], mesh.normals[3 * i + 2]));
            }

            if has_coords {
                vertex = vertex.texture_coords_0(Vector2::new(mesh.texcoords[2 * i], mesh.texcoords[2 * i + 1]));
            }

            vertex
        }).collect::<Vec<_>>();

        let mesh = Mesh::new(vertices, mesh.indices);

        // OBJ files are right-handed and wind counter-clockwise, as carbide_3d does.
        let mesh = if has_normals { mesh } else { mesh.calculate_normals() };

        mesh.calculate_tangents()
    }
}

#[cfg(test)]
mod tests {
    use carbide::math::{InnerSpace, Vector3};
    use crate::Mesh;

    /// Every normal of `mesh` has length 1 and points away from the origin, as for a closed
    /// shape around it.
    fn assert_outward_unit_normals(mesh: &Mesh) {
        for vertex in mesh.vertices() {
            assert!((vertex.normal.magnitude() - 1.0).abs() < 1e-4, "normal {:?} is not unit length", vertex.normal);
            assert!(vertex.normal.dot(vertex.position) > 0.0, "normal {:?} at {:?} points inwards", vertex.normal, vertex.position);
        }
    }

    #[test]
    fn clones_share_their_data_and_changes_make_a_new_mesh() {
        let mesh = Mesh::cube(1.0);
        let clone = mesh.clone();
        assert_eq!(mesh.id(), clone.id());

        let weak = mesh.downgrade();
        let changed = clone.calculate_tangents();
        assert_ne!(changed.id(), mesh.id());

        // The original is still alive through `mesh`, and gone once that is dropped.
        assert!(weak.is_alive());
        drop(mesh);
        assert!(!weak.is_alive());
    }

    #[test]
    fn calculated_normals_are_unit_length() {
        let cube = Mesh::cube(2.0);
        let mut vertices = cube.vertices().to_vec();
        for vertex in &mut vertices {
            vertex.normal = Vector3::new(0.0, 0.0, 0.0);
        }

        let mesh = Mesh::new(vertices, cube.indices().to_vec()).calculate_normals();
        assert_outward_unit_normals(&mesh);
    }

    #[test]
    fn cube_and_icosphere_have_outward_normals() {
        assert_outward_unit_normals(&Mesh::cube(1.0));
        assert_outward_unit_normals(&Mesh::icosphere(1.0, 2));
        assert_eq!(Mesh::icosphere(1.0, 2).indices().len(), 20 * 16 * 3);
    }

    #[test]
    fn tangents_follow_the_texture_coordinates() {
        // The cube's faces have u along their right direction, so the tangent is that direction:
        // for the front face (normal +Z, facing the default camera), +X.
        let cube = Mesh::cube(1.0);
        let front = cube.vertices().iter().find(|vertex| vertex.normal.z > 0.5).unwrap();
        assert!((front.tangent - Vector3::new(1.0, 0.0, 0.0)).magnitude() < 1e-4, "tangent {:?}", front.tangent);
    }

    #[test]
    fn an_obj_without_normals_gets_outward_normals() {
        // A tetrahedron around the origin, counter-clockwise seen from outside, with no normals
        // or texture coordinates.
        let obj = "v 0 1 0\nv -1 -1 1\nv 1 -1 1\nv 0 -1 -1\nf 1 2 3\nf 1 3 4\nf 1 4 2\nf 2 4 3\n";
        let (models, _) = tobj::load_obj_buf(&mut obj.as_bytes(), &tobj::GPU_LOAD_OPTIONS, |_| Ok(Default::default())).unwrap();

        let mesh = Mesh::from(models.into_iter().next().unwrap());
        assert_eq!(mesh.vertices().len(), 4);
        assert_eq!(mesh.indices().len(), 12);
        assert_outward_unit_normals(&mesh);
    }
}
