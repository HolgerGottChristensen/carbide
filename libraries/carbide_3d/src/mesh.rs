use std::collections::HashMap;
use std::f32::consts::{PI, TAU};
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use carbide::impl_state_value;

use crate::vertex::Vertex;
use tobj::Model;
use carbide::math::{InnerSpace, Vector2, Vector3, Vector4, Zero};

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

        // Counter-clockwise seen from above, with +X right and +Z towards the viewer. The image's
        // top edge (v = 0) is the far edge (-Z), as the cube's top face has it, so the image reads
        // the right way round from the front.
        let vertices = vec![
            Vertex::new(Vector3::new(-x, 0.0, z)).normal(up).texture_coords_0(Vector2::new(0.0, 1.0)),
            Vertex::new(Vector3::new(x, 0.0, z)).normal(up).texture_coords_0(Vector2::new(1.0, 1.0)),
            Vertex::new(Vector3::new(x, 0.0, -z)).normal(up).texture_coords_0(Vector2::new(1.0, 0.0)),
            Vertex::new(Vector3::new(-x, 0.0, -z)).normal(up).texture_coords_0(Vector2::new(0.0, 0.0)),
        ];

        Mesh::new(vertices, vec![0, 1, 2, 0, 2, 3]).calculate_tangents()
    }

    /// A sphere centred on the origin with the given `radius`, made by splitting each triangle of
    /// an icosahedron into four `subdivisions` times: 20 triangles at 0, 20 · 4ⁿ at n. Triangles
    /// share their vertices, whose normals point straight out from the centre, so it is shaded
    /// smooth, as a round sphere. Use [`flat`](Mesh::flat) to shade each triangle flat.
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

        // Shared vertices, with the exact normals of a sphere: the direction from the centre.
        // Texture coordinates are longitude and latitude, as an equirectangular image wraps a
        // globe: u runs once around, from the back (-Z) through the front (+Z, the image's
        // centre), reading left to right from outside; v runs from the top pole (0) to the bottom
        // pole (1).
        let is_pole = |p: Vector3<f32>| p.x.abs() < 1e-6 && p.z.abs() < 1e-6;
        let coords = |p: Vector3<f32>| {
            // Exactly 0 or 1 at the poles, where rounding in y would otherwise show in acos.
            let v = if is_pole(p) { if p.y > 0.0 { 0.0 } else { 1.0 } } else { p.y.clamp(-1.0, 1.0).acos() / PI };
            Vector2::new(0.5 + p.x.atan2(p.z) / TAU, v)
        };

        let mut vertices = positions.iter()
            .map(|position| {
                let mut vertex = Vertex::new(position * radius).texture_coords_0(coords(*position));
                vertex.normal = *position;
                vertex
            })
            .collect::<Vec<_>>();

        // Copies of vertices on the low side of the seam, with u + 1, made once per vertex.
        let mut seam_copies: HashMap<u32, u32> = HashMap::new();
        let mut indices = Vec::with_capacity(triangles.len() * 3);

        for triangle in triangles {
            let mut corners = triangle;

            // A triangle across the seam has corners near u = 1 and near u = 0. Its low corners
            // use copies past 1, so it covers the narrow strip, not the whole texture backwards.
            // Poles have no longitude, so they don't count.
            let us = triangle.iter()
                .filter(|index| !is_pole(positions[**index as usize]))
                .map(|index| vertices[*index as usize].texture_coords_0.x)
                .collect::<Vec<_>>();
            let spread = us.iter().cloned().fold(f32::MIN, f32::max) - us.iter().cloned().fold(f32::MAX, f32::min);

            if spread > 0.5 {
                for corner in &mut corners {
                    if !is_pole(positions[*corner as usize]) && vertices[*corner as usize].texture_coords_0.x < 0.5 {
                        *corner = *seam_copies.entry(*corner).or_insert_with(|| {
                            let mut copy = vertices[*corner as usize];
                            copy.texture_coords_0.x += 1.0;
                            vertices.push(copy);
                            vertices.len() as u32 - 1
                        });
                    }
                }
            }

            // A pole gets a copy per triangle, with u between the triangle's other two corners,
            // so the texture meets at the pole instead of twisting around it.
            for i in 0..3 {
                if is_pole(positions[triangle[i] as usize]) {
                    let others = [corners[(i + 1) % 3], corners[(i + 2) % 3]];
                    let mut copy = vertices[triangle[i] as usize];
                    copy.texture_coords_0.x = (vertices[others[0] as usize].texture_coords_0.x + vertices[others[1] as usize].texture_coords_0.x) / 2.0;
                    vertices.push(copy);
                    corners[i] = vertices.len() as u32 - 1;
                }
            }

            indices.extend(corners);
        }

        // Tangents along increasing u: east, around the vertical axis.
        for vertex in &mut vertices {
            let longitude = (vertex.texture_coords_0.x - 0.5) * TAU;
            vertex.tangent = Vector4::new(longitude.cos(), 0.0, -longitude.sin(), 1.0);
        }

        Mesh::new(vertices, indices)
    }

    /// The meshes of the OBJ file at `path`, one per object or group in it, triangulated with one
    /// index per vertex. Missing normals are calculated (flat), and missing texture coordinates
    /// are zero. OBJ materials are not loaded.
    pub fn load_obj<P: AsRef<Path>>(path: P) -> Result<Vec<Mesh>, tobj::LoadError> {
        let (models, _materials) = tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS)?;
        Ok(models.into_iter().map(Mesh::from).collect())
    }

    /// The same triangles, each with three vertices of its own and its own normal, so each is
    /// shaded flat. Other vertex data is kept.
    pub fn flat(self) -> Mesh {
        let data = self.into_data();

        let vertices = data.indices.iter()
            .map(|index| data.vertices[*index as usize])
            .collect::<Vec<_>>();
        let indices = (0..vertices.len() as u32).collect::<Vec<u32>>();

        Mesh::new(vertices, indices).calculate_normals()
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

    /// Calculate each vertex's tangent with MikkTSpace, from its triangles' positions, normals
    /// and first texture coordinates, as glTF requires for meshes without tangents, so normal maps
    /// baked by other tools light the same here. The tangent points along increasing u, and its w
    /// is the sign that makes `cross(normal, tangent) · w` the image's up, so mirrored halves of a
    /// texture light the right way. Vertices whose triangles' texture coordinates don't span an
    /// area get a zero tangent, which draws them with their vertex normal.
    pub fn calculate_tangents(self) -> Self {
        let mut data = self.into_data();

        let mut geometry = TangentGeometry {
            vertices: &data.vertices,
            indices: &data.indices,
            tangents: vec![Vector4::new(0.0, 0.0, 0.0, 1.0); data.vertices.len()],
        };
        // The error type has no variants yet; on one, the mesh keeps no tangents.
        let generated = bevy_mikktspace::generate_tangents(&mut geometry).is_ok();
        let tangents = geometry.tangents;
        if generated {
            for (vertex, tangent) in data.vertices.iter_mut().zip(tangents) {
                vertex.tangent = tangent;
            }
        }

        Mesh::from_data(data)
    }
}

/// A mesh as MikkTSpace reads it: triangles of indexed vertices. MikkTSpace gives one tangent
/// per triangle corner, which for an indexed mesh agree on every corner that shares a vertex,
/// unless the vertex is shared across a texture seam, where the last one wins.
struct TangentGeometry<'a> {
    vertices: &'a [Vertex],
    indices: &'a [u32],
    tangents: Vec<Vector4<f32>>,
}

impl TangentGeometry<'_> {
    fn vertex(&self, face: usize, vert: usize) -> &Vertex {
        &self.vertices[self.indices[face * 3 + vert] as usize]
    }
}

impl bevy_mikktspace::Geometry for TangentGeometry<'_> {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertex(face, vert).position.into()
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.vertex(face, vert).normal.into()
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.vertex(face, vert).texture_coords_0.into()
    }

    fn set_tangent(&mut self, tangent_space: Option<bevy_mikktspace::TangentSpace>, face: usize, vert: usize) {
        let index = self.indices[face * 3 + vert] as usize;
        self.tangents[index] = match tangent_space {
            // MikkTSpace's bitangent follows increasing v, which is down the image in carbide's
            // (and glTF's) texture coordinates, so the sign is flipped to point up the image, as
            // Bevy does (`bevy_mesh/src/mikktspace.rs`).
            Some(space) => {
                let [x, y, z, w] = space.tangent_encoded();
                Vector4::new(x, y, z, -w)
            }
            None => Vector4::new(0.0, 0.0, 0.0, 1.0),
        };
    }
}

/// A mesh from a model loaded with tobj's `GPU_LOAD_OPTIONS` (triangulated, one index per
/// vertex). Normals the model doesn't have are calculated, and texture coordinates it doesn't
/// have are zero. v is flipped, from OBJ's convention (up) to carbide's (down).
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

            // OBJ's v points up, from the image's bottom row; carbide's points down, from its top
            // row, as glTF's and the images' own rows do.
            if has_coords {
                vertex = vertex.texture_coords_0(Vector2::new(mesh.texcoords[2 * i], 1.0 - mesh.texcoords[2 * i + 1]));
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
    use carbide::math::{InnerSpace, Vector2, Vector3, Vector4};
    use crate::{Mesh, Vertex};

    /// Every normal of `mesh` has length 1 and points away from the origin, as for a closed
    /// shape around it.
    fn assert_outward_unit_normals(mesh: &Mesh) {
        for vertex in mesh.vertices() {
            assert!((vertex.normal.magnitude() - 1.0).abs() < 1e-4, "normal {:?} is not unit length", vertex.normal);
            assert!(vertex.normal.dot(vertex.position) > 0.0, "normal {:?} at {:?} points inwards", vertex.normal, vertex.position);
        }
    }

    /// The icosphere's texture coordinates: in range, no triangle stretched across the seam or
    /// twisted at a pole, and tangents along the surface, pointing the way u increases.
    #[test]
    fn icosphere_texture_coordinates() {
        let mesh = Mesh::icosphere(1.0, 3);
        let vertices = mesh.vertices();

        for vertex in vertices {
            let uv = vertex.texture_coords_0;
            assert!((0.0..=2.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y), "{uv:?} out of range");
            assert!((vertex.tangent.truncate().magnitude() - 1.0).abs() < 1e-4);
            assert!(vertex.tangent.truncate().dot(vertex.normal).abs() < 1e-4, "tangent not along the surface");
            assert_eq!(vertex.tangent.w, 1.0);
        }

        for triangle in mesh.indices().chunks_exact(3) {
            let us = triangle.iter().map(|index| vertices[*index as usize].texture_coords_0.x).collect::<Vec<_>>();
            let spread = us.iter().cloned().fold(f32::MIN, f32::max) - us.iter().cloned().fold(f32::MAX, f32::min);
            assert!(spread < 0.25, "a triangle spans {spread} of the texture: {us:?}");

            // Moving from corner 0 towards a corner with larger u moves east, along the tangent.
            let (a, b) = (vertices[triangle[0] as usize], vertices[triangle[1] as usize]);
            let du = b.texture_coords_0.x - a.texture_coords_0.x;
            if du.abs() > 1e-3 && a.normal.y.abs() < 0.9 {
                assert!((b.position - a.position).dot(a.tangent.truncate()) * du > 0.0, "tangent points against u");
            }
        }

        // The front of the sphere is the middle of the image, its top the top row.
        let front = vertices.iter().find(|v| (v.normal - Vector3::unit_z()).magnitude() < 1e-4).unwrap();
        assert!((front.texture_coords_0.x - 0.5).abs() < 1e-4 && (front.texture_coords_0.y - 0.5).abs() < 1e-4);
        let top = vertices.iter().find(|v| (v.normal - Vector3::unit_y()).magnitude() < 1e-4).unwrap();
        assert!(top.texture_coords_0.y.abs() < 1e-4);
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
        assert!((front.tangent - Vector4::new(1.0, 0.0, 0.0, 1.0)).magnitude() < 1e-4, "tangent {:?}", front.tangent);

        // Every built-in mesh has the image's up at cross(normal, tangent), so w is 1.
        for mesh in [Mesh::cube(1.0), Mesh::plane(1.0, 1.0)] {
            assert!(mesh.vertices().iter().all(|vertex| vertex.tangent.w == 1.0));
        }
    }

    /// Two quads side by side facing +Z, the right one with its texture mirrored: u runs right on
    /// the left quad and left on the right one, v down on both. The tangent follows u, so it points
    /// left on the mirrored quad, and w turns to -1 so that `cross(normal, tangent) · w` still points
    /// up the image, +Y.
    #[test]
    fn a_mirrored_texture_flips_the_tangent_sign() {
        let normal = Vector3::unit_z();
        let quad = |x0: f32, u0: f32, u1: f32| [
            Vertex::new(Vector3::new(x0, 1.0, 0.0)).normal(normal).texture_coords_0(Vector2::new(u0, 0.0)),
            Vertex::new(Vector3::new(x0, 0.0, 0.0)).normal(normal).texture_coords_0(Vector2::new(u0, 1.0)),
            Vertex::new(Vector3::new(x0 + 1.0, 0.0, 0.0)).normal(normal).texture_coords_0(Vector2::new(u1, 1.0)),
            Vertex::new(Vector3::new(x0 + 1.0, 1.0, 0.0)).normal(normal).texture_coords_0(Vector2::new(u1, 0.0)),
        ];
        let vertices = [quad(-1.0, 0.0, 1.0), quad(0.0, 1.0, 0.0)].concat();
        let indices = vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];
        let mesh = Mesh::new(vertices, indices).calculate_tangents();

        for (index, vertex) in mesh.vertices().iter().enumerate() {
            let (expected, sign) = if index < 4 { (Vector3::unit_x(), 1.0) } else { (-Vector3::unit_x(), -1.0) };
            assert!((vertex.tangent.truncate() - expected).magnitude() < 1e-4, "vertex {index}: tangent {:?}", vertex.tangent);
            assert_eq!(vertex.tangent.w, sign, "vertex {index}");
            let image_up = vertex.normal.cross(vertex.tangent.truncate()) * vertex.tangent.w;
            assert!((image_up - Vector3::unit_y()).magnitude() < 1e-4, "vertex {index}: image up {image_up:?}");
        }
    }

    /// A mesh without texture coordinates has no tangents: zero, so it is drawn with its vertex
    /// normals.
    #[test]
    fn no_texture_coordinates_give_no_tangents() {
        let cube = Mesh::cube(1.0);
        let mut vertices = cube.vertices().to_vec();
        for vertex in &mut vertices {
            vertex.texture_coords_0 = Vector2::new(0.0, 0.0);
        }
        let mesh = Mesh::new(vertices, cube.indices().to_vec()).calculate_tangents();
        for vertex in mesh.vertices() {
            assert_eq!(vertex.tangent.truncate().magnitude(), 0.0, "tangent {:?}", vertex.tangent);
        }
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
