use crate::gradient::{Dashes, Gradient};
use carbide_core::draw::{ImageId, Rect};
use carbide_core::render::LayerId;
use carbide_core::widget::FilterId;

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
    Uniform {
        uniform_bind_group_index: usize,
    },
    Gradient(Gradient),
    StrokeDashing(Dashes),
    /// A new image requires drawing and in turn a new bind group requires setting.
    SetBindGroup {
        bind_group: WGPUBindGroup,
    },
    SetMaskBindGroup {
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
        mask_id: Option<usize>,
        initial_copy: bool,
    },
}

#[derive(PartialEq, Debug, Clone)]
pub enum WGPUBindGroup {
    // Default,
    Image(ImageId),
    Target(usize),
    Layer(LayerId)
}