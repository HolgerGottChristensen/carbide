use tobj::Model;
use carbide::render::matrix::{InnerSpace, Vector2, Vector3, Zero};
use crate::handedness::Handedness;
use crate::vertex::Vertex;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub handedness: Handedness,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(mut vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
        Mesh {
            handedness: Default::default(),
            vertices,
            indices,
        }
    }

    pub fn calculate_normals(mut self, handedness: Handedness) -> Self {
        for chunk in self.indices.chunks(3) {
            let pos0 = self.vertices[chunk[0] as usize].position;
            let pos1 = self.vertices[chunk[1] as usize].position;
            let pos2 = self.vertices[chunk[2] as usize].position;

            let edge1 = pos1 - pos0;
            let edge2 = pos2 - pos0;

            let normal = if handedness == Handedness::Left { edge1.cross(edge2) } else { edge2.cross(edge1) };

            self.vertices[chunk[0] as usize].normal += normal;
            self.vertices[chunk[1] as usize].normal += normal;
            self.vertices[chunk[2] as usize].normal += normal;
        }

        for vertex in &mut self.vertices {
            if vertex.normal.magnitude() != 0.0 {
                vertex.normal.normalize();
            }
        }

        self
    }

    pub fn calculate_tangents(mut self) -> Self {
        for chunk in self.indices.chunks(3) {
            let pos0 = self.vertices[chunk[0] as usize].position;
            let pos1 = self.vertices[chunk[1] as usize].position;
            let pos2 = self.vertices[chunk[2] as usize].position;

            let tex0 = self.vertices[chunk[0] as usize].texture_coords_0;
            let tex1 = self.vertices[chunk[1] as usize].texture_coords_0;
            let tex2 = self.vertices[chunk[2] as usize].texture_coords_0;

            let uv1 = tex1 - tex0;
            let uv2 = tex2 - tex0;

            let edge1 = pos1 - pos0;
            let edge2 = pos2 - pos0;

            let r = 1.0 / (uv1.x * uv2.y - uv1.y * uv2.x);

            let tangent = (edge1 * uv2.y) - (edge2 * uv1.y) * r;


            self.vertices[chunk[0] as usize].tangent += tangent;
            self.vertices[chunk[1] as usize].tangent += tangent;
            self.vertices[chunk[2] as usize].tangent += tangent;
        }

        for vertex in &mut self.vertices {
            let t = vertex.tangent - (vertex.normal * vertex.normal.dot(vertex.tangent));
            if t.magnitude() != 0.0 {
                vertex.tangent = t.normalize();
            } else {
                vertex.tangent = Vector3::zero();
            }
        }

        self
    }
}

impl From<Model> for Mesh {
    fn from(value: Model) -> Self {
        println!("{:?}", value.mesh.positions.len());
        println!("{:?}", value.mesh.normals.len());
        println!("{:?}", value.mesh.texcoords.len());
        println!("{:?}", value.mesh.vertex_color.len());
        let vertices = value.mesh.positions.chunks(3)
            .zip(value.mesh.normals.chunks(3))
            .zip(value.mesh.texcoords.chunks(2)).map(|((pos, normal), (tex))| {
            Vertex::new(Vector3::new(pos[0], pos[1], pos[2]))
                .normal(Vector3::new(normal[0], normal[1], normal[2]))
                .texture_coords_0(Vector2::new(tex[0], tex[1]))
        }).collect::<Vec<_>>();

        let indices = value.mesh.indices;

        Mesh::new(vertices, indices)
            .calculate_tangents()

    }
}