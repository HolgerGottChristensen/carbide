use bytemuck::{Pod, Zeroable};
use wgpu::VertexFormat;
use carbide_3d::Vertex;
use carbide_core::color::ColorExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct WgpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 4],
    texture_coords_0: [f32; 2],
    texture_coords_1: [f32; 2],
    color_0: [f32; 4],
    color_1: [f32; 4],
}

impl WgpuVertex {
    pub fn from_vertex(vertex: &Vertex) -> WgpuVertex {
        WgpuVertex {
            position: [vertex.position.x, vertex.position.y, vertex.position.z],
            normal: [vertex.normal.x, vertex.normal.y, vertex.normal.z],
            tangent: [vertex.tangent.x, vertex.tangent.y, vertex.tangent.z, vertex.tangent.w],
            texture_coords_0: [vertex.texture_coords_0.x, vertex.texture_coords_0.y],
            texture_coords_1: [vertex.texture_coords_1.x, vertex.texture_coords_1.y],
            color_0: [vertex.color_0.red(), vertex.color_0.green(), vertex.color_0.blue(), vertex.color_0.opacity()],
            color_1: [vertex.color_1.red(), vertex.color_1.green(), vertex.color_1.blue(), vertex.color_1.opacity()],
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<WgpuVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            // Byte offsets of WgpuVertex's fields, in order.
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 40,
                    shader_location: 3,
                    format: VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 56,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 72,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}