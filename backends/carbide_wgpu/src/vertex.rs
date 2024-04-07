use wgpu::VertexFormat;
use carbide_winit::dpi::PhysicalSize;

use carbide_core::draw::Scalar;
use carbide_core::draw::MODE_IMAGE;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub rgba: [f32; 4],
    pub mode: u32,
    pub line_coords: [f32; 4],
}

impl Vertex {

    pub fn rect(size: PhysicalSize<u32>, scale_factor: Scalar, zoom: f32) -> Vec<Vertex> {
        let half_width = size.width as f32 / 2.0 / zoom;
        let half_height = size.height as f32 / 2.0 / zoom;
        let offset_x = half_width - half_width * zoom;
        let offset_y = half_height - half_height * zoom;

        let total_scale_factor = scale_factor as f32 / zoom;

        vec![
            Vertex::new_from_2d(
                offset_x / total_scale_factor as f32,
                offset_y / total_scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0],
                MODE_IMAGE
            ),
            Vertex::new_from_2d(
                (size.width as f32 + offset_x) / total_scale_factor as f32,
                offset_y / total_scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                offset_x / total_scale_factor as f32,
                (size.height as f32 + offset_y) / total_scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                (size.width as f32 + offset_x) / total_scale_factor as f32,
                offset_y / total_scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                (size.width as f32 + offset_x) / total_scale_factor as f32,
                (size.height as f32 + offset_y) / total_scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 1.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                offset_x / total_scale_factor as f32,
                (size.height as f32 + offset_y) / total_scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0],
                MODE_IMAGE,
            ),
        ]
    }

    pub fn new_from_2d(x: f32, y: f32, color: [f32; 4], tex_coords: [f32; 2], mode: u32) -> Vertex {
        Vertex {
            position: [x, y, 0.0],
            tex_coords,
            rgba: color,
            mode,
            line_coords: [0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // 1.
            step_mode: wgpu::VertexStepMode::Vertex,                            // 2.
            attributes: &[
                // 3.
                wgpu::VertexAttribute {
                    offset: 0,                       // 4.
                    shader_location: 0,              // 5.
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
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>()
                        + std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<[f32; 4]>()
                        + std::mem::size_of::<[u32; 1]>())
                        as wgpu::BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}