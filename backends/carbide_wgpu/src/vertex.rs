use wgpu::VertexFormat;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub rgba: [f32; 4],
    pub mode: u32,
}

impl Vertex {
    pub fn new_from_2d(x: f32, y: f32, color: [f32; 4], tex_coords: [f32; 2], mode: u32) -> Vertex {
        Vertex {
            position: [x, y, 0.0],
            tex_coords,
            rgba: color,
            mode,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // 1.
            step_mode: wgpu::InputStepMode::Vertex,                       // 2.
            attributes: &[
                // 3.
                wgpu::VertexAttribute {
                    offset: 0,                          // 4.
                    shader_location: 0,                 // 5.
                    format: VertexFormat::Float32x3, // 6.
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>()
                        + std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Uint32,
                },
            ],
        }
    }
}

impl From<carbide_core::mesh::vertex::Vertex> for Vertex {
    fn from(v: carbide_core::mesh::vertex::Vertex) -> Self {
        Vertex {
            position: v.position,
            tex_coords: v.tex_coords,
            rgba: v.rgba,
            mode: v.mode,
        }
    }
}
