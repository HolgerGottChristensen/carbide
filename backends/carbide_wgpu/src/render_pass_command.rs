use cgmath::Matrix4;
use wgpu::{BindGroupLayout, Device};

use carbide_core::draw::image::ImageId;
use carbide_core::draw::Rect;
use carbide_core::mesh::DrawCommand;
use carbide_core::widget::FilterId;

use crate::bind_groups::matrix_to_uniform_bind_group;
use crate::gradient::Gradient;

/// A draw command that maps directly to the `wgpu::CommandEncoder` method. By returning
/// `RenderPassCommand`s, we can avoid consuming the entire `AutoCommandBufferBuilder` itself which might
/// not always be available from APIs that wrap Vulkan.
#[derive(Debug)]
pub enum RenderPassCommand {
    /// Specify the rectangle to which drawing should be cropped.
    SetScissor {
        rect: Rect,
    },
    /// Draw the specified range of vertices.
    Draw {
        vertex_range: std::ops::Range<u32>,
    },
    Stencil {
        vertex_range: std::ops::Range<u32>,
    },
    DeStencil {
        vertex_range: std::ops::Range<u32>,
    },
    Transform {
        uniform_bind_group_index: usize,
    },
    Gradient {
        index: usize,
    },
    /// A new image requires drawing and in turn a new bind group requires setting.
    SetBindGroup {
        bind_group: WGPUBindGroup,
    },
}

#[derive(Debug)]
pub enum RenderPass {
    Normal {
        commands: Vec<RenderPassCommand>,
        target_index: usize
    },
    Clear {
        target_index: usize
    },
    Filter {
        vertex_range: std::ops::Range<u32>,
        filter_id: FilterId,
        source_id: usize,
        target_id: usize,
        initial_copy: bool,
    },
}

#[derive(PartialEq, Debug)]
pub enum WGPUBindGroup {
    Default,
    Image(ImageId),
    Target(usize),
}