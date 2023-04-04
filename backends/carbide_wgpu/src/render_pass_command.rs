use std::collections::HashMap;

use cgmath::Matrix4;
use wgpu::{BindGroupLayout, Device};
use wgpu::util::DeviceExt;

use carbide_core::draw::image::ImageId;
use carbide_core::draw::Rect;
use carbide_core::mesh::DrawCommand;
use carbide_core::widget::FilterId;

use crate::bind_groups::{gradient_buffer_bind_group, matrix_to_uniform_bind_group};
use crate::diffuse_bind_group::DiffuseBindGroup;
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
    /// A new image requires drawing and in turn a new bind group requires setting.
    SetBindGroup {
        bind_group: WGPUBindGroup,
    },
}

#[derive(Debug)]
pub enum RenderPass {
    Normal(Vec<RenderPassCommand>),
    Gradient(std::ops::Range<u32>, usize),
    Filter(std::ops::Range<u32>, FilterId),
    FilterSplitPt1(std::ops::Range<u32>, FilterId),
    FilterSplitPt2(std::ops::Range<u32>, FilterId),
}

#[derive(PartialEq, Debug)]
pub enum WGPUBindGroup {
    Default,
    Image(ImageId),
}

impl WGPUBindGroup {
    pub fn get(&self) -> ImageId {
        match self {
            WGPUBindGroup::Default => ImageId::default(),
            WGPUBindGroup::Image(id) => id.clone(),
        }
    }
}

pub fn draw_commands_to_render_pass_commands<'a>(
    draw_commands: &[DrawCommand],
    uniform_bind_groups: &mut Vec<wgpu::BindGroup>,
    device: &Device,
    uniform_bind_group_layout: &BindGroupLayout,
    gradient_bind_group_layout: &BindGroupLayout,
    carbide_to_wgpu_matrix: Matrix4<f32>,
) -> Vec<RenderPass> {

    let mut commands = vec![];
    let mut inner_commands = vec![];

    let mut current_bind_group = None;

    for command in draw_commands {
        match command {
            // Update the `scissor` before continuing to draw.
            DrawCommand::Scissor(scissor_rect) => {

                let cmd = RenderPassCommand::SetScissor {
                    rect: *scissor_rect
                };

                inner_commands.push(cmd);
            }

            DrawCommand::Filter(vertex_range, filter_id) => {
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: WGPUBindGroup::Default,
                    };
                    inner_commands.push(cmd);
                }

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::Filter(range, *filter_id));
                current_bind_group = None;
            }
            DrawCommand::FilterSplitPt1(vertex_range, filter_id) => {
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: WGPUBindGroup::Default,
                    };
                    inner_commands.push(cmd);
                }

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::FilterSplitPt1(range, *filter_id));
                current_bind_group = None;
            }
            DrawCommand::FilterSplitPt2(vertex_range, filter_id) => {
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: WGPUBindGroup::Default,
                    };
                    inner_commands.push(cmd);
                }

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::FilterSplitPt2(range, *filter_id));
                current_bind_group = None;
            }

            DrawCommand::Stencil(vertex_range) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    current_bind_group = Some(WGPUBindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: WGPUBindGroup::Default,
                    };
                    inner_commands.push(cmd);
                }
                let cmd = RenderPassCommand::Stencil {
                    vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                };
                inner_commands.push(cmd);
            }

            DrawCommand::DeStencil(vertex_range) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    current_bind_group = Some(WGPUBindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: WGPUBindGroup::Default,
                    };
                    inner_commands.push(cmd);
                }
                let cmd = RenderPassCommand::DeStencil {
                    vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                };
                inner_commands.push(cmd);
            }

            DrawCommand::Transform(matrix) => {
                let transformed_matrix = carbide_to_wgpu_matrix * matrix;
                let new_bind_group = matrix_to_uniform_bind_group(
                    device,
                    uniform_bind_group_layout,
                    transformed_matrix,
                );

                inner_commands.push(RenderPassCommand::Transform {
                    uniform_bind_group_index: uniform_bind_groups.len(),
                });
                uniform_bind_groups.push(new_bind_group);
            }
            DrawCommand::Geometry(vertex_range) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    current_bind_group = Some(WGPUBindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: WGPUBindGroup::Default,
                    };
                    inner_commands.push(cmd);
                }
                let cmd = RenderPassCommand::Draw {
                    vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                };
                inner_commands.push(cmd);
            }
            DrawCommand::Image(vertex_range, image_id) => {
                let vertex_count = vertex_range.len();
                if vertex_count == 0 {
                    continue;
                }

                // Ensure the bind group matches this image.
                let new_group = WGPUBindGroup::Image(image_id.clone());
                let expected_bind_group = Some(WGPUBindGroup::Image(image_id.clone()));
                if current_bind_group != expected_bind_group {
                    // Now update the bind group and add the new bind group command.
                    current_bind_group = expected_bind_group;
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: new_group,
                    };
                    inner_commands.push(cmd);
                }
                let cmd = RenderPassCommand::Draw {
                    vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                };
                inner_commands.push(cmd);
            }
            DrawCommand::Gradient(vertex_range, gradient) => {
                // If there is no vertices continue
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }

                let gradient = Gradient::convert(gradient);
                let gradient_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Gradient Buffer"),
                        contents: &*gradient.as_bytes(),
                        usage: wgpu::BufferUsages::STORAGE,
                    });
                let gradient_buffer_bind_group = gradient_buffer_bind_group(
                    &device,
                    &gradient_bind_group_layout,
                    &gradient_buffer,
                );

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::Gradient(range, uniform_bind_groups.len()));
                uniform_bind_groups.push(gradient_buffer_bind_group);
                current_bind_group = None;
            }
        }
    }

    commands.push(RenderPass::Normal(inner_commands));

    commands
}
