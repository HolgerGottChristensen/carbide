use std::collections::HashMap;

use carbide_core::draw::image::{ImageId, ImageMap};
use cgmath::Matrix4;
use wgpu::util::DeviceExt;
use wgpu::{BindGroupLayout, Device, Texture};

use carbide_core::mesh::mesh;
use carbide_core::mesh::mesh::{Draw, Mesh};
use carbide_core::widget::FilterId;

use crate::bind_groups::{gradient_buffer_bind_group, matrix_to_uniform_bind_group};
use crate::diffuse_bind_group::{new_diffuse, DiffuseBindGroup};
use crate::gradient::Gradient;
use crate::image::Image;

/// A draw command that maps directly to the `wgpu::CommandEncoder` method. By returning
/// `RenderPassCommand`s, we can avoid consuming the entire `AutoCommandBufferBuilder` itself which might
/// not always be available from APIs that wrap Vulkan.
pub enum RenderPassCommand<'a> {
    /// Specify the rectangle to which drawing should be cropped.
    SetScissor {
        top_left: [u32; 2],
        dimensions: [u32; 2],
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
        bind_group: &'a wgpu::BindGroup,
    },
}

pub enum RenderPass<'a> {
    Normal(Vec<RenderPassCommand<'a>>),
    Gradient(std::ops::Range<u32>, usize),
    Filter(std::ops::Range<u32>, FilterId),
    FilterSplitPt1(std::ops::Range<u32>, FilterId),
    FilterSplitPt2(std::ops::Range<u32>, FilterId),
}

#[derive(PartialEq)]
enum BindGroup {
    Default,
    Image(ImageId),
}

pub fn create_render_pass_commands<'a>(
    bind_groups: &'a HashMap<ImageId, DiffuseBindGroup>,
    uniform_bind_groups: &mut Vec<wgpu::BindGroup>,
    mesh: &Mesh,
    device: &Device,
    uniform_bind_group_layout: &BindGroupLayout,
    gradient_bind_group_layout: &BindGroupLayout,
    carbide_to_wgpu_matrix: Matrix4<f32>,
) -> Vec<RenderPass<'a>> {

    let mut commands = vec![];
    let mut inner_commands = vec![];

    let mut current_bind_group = None;

    for command in mesh.commands() {
        match command {
            // Update the `scissor` before continuing to draw.
            mesh::Command::Scissor(s) => {
                let top_left = [s.top_left[0] as u32, s.top_left[1] as u32];
                let dimensions = s.dimensions;
                let cmd = RenderPassCommand::SetScissor {
                    top_left,
                    dimensions,
                };
                inner_commands.push(cmd);
            }

            mesh::Command::Filter(vertex_range, filter_id) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: &bind_groups[&ImageId::default()],
                    };
                    inner_commands.push(cmd);
                }

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::Filter(range, filter_id));
                current_bind_group = None;
            }
            mesh::Command::FilterSplitPt1(vertex_range, filter_id) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: &bind_groups[&ImageId::default()],
                    };
                    inner_commands.push(cmd);
                }

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::FilterSplitPt1(range, filter_id));
                current_bind_group = None;
            }
            mesh::Command::FilterSplitPt2(vertex_range, filter_id) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: &bind_groups[&ImageId::default()],
                    };
                    inner_commands.push(cmd);
                }

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::FilterSplitPt2(range, filter_id));
                current_bind_group = None;
            }

            mesh::Command::Stencil(vertex_range) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    current_bind_group = Some(BindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: &bind_groups[&ImageId::default()],
                    };
                    inner_commands.push(cmd);
                }
                let cmd = RenderPassCommand::Stencil {
                    vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                };
                inner_commands.push(cmd);
            }

            mesh::Command::DeStencil(vertex_range) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if current_bind_group.is_none() {
                    current_bind_group = Some(BindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: &bind_groups[&ImageId::default()],
                    };
                    inner_commands.push(cmd);
                }
                let cmd = RenderPassCommand::DeStencil {
                    vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                };
                inner_commands.push(cmd);
            }

            mesh::Command::Transform(matrix) => {
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
            // Draw to the target with the given `draw` command.
            mesh::Command::Draw(draw) => match draw {
                // Draw text and plain 2D geometry.
                mesh::Draw::Plain(vertex_range) => {
                    let vertex_count = vertex_range.len();
                    if vertex_count <= 0 {
                        continue;
                    }
                    // Ensure a render pipeline and bind group is set.
                    if current_bind_group.is_none() {
                        current_bind_group = Some(BindGroup::Default);
                        let cmd = RenderPassCommand::SetBindGroup {
                            bind_group: &bind_groups[&ImageId::default()],
                        };
                        inner_commands.push(cmd);
                    }
                    let cmd = RenderPassCommand::Draw {
                        vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                    };
                    inner_commands.push(cmd);
                }

                // Draw an image whose texture data lies within the `image_map` at the
                // given `id`.
                mesh::Draw::Image(image_id, vertex_range) => {
                    let vertex_count = vertex_range.len();
                    if vertex_count == 0 {
                        continue;
                    }

                    // Ensure the bind group matches this image.
                    let expected_bind_group = Some(BindGroup::Image(image_id.clone()));
                    if current_bind_group != expected_bind_group {
                        // Now update the bind group and add the new bind group command.
                        current_bind_group = expected_bind_group;
                        let cmd = RenderPassCommand::SetBindGroup {
                            bind_group: &bind_groups[&image_id],
                        };
                        inner_commands.push(cmd);
                    }
                    let cmd = RenderPassCommand::Draw {
                        vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                    };
                    inner_commands.push(cmd);
                }
                Draw::Gradient(vertex_range, gradient) => {
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
            },
        }
    }

    commands.push(RenderPass::Normal(inner_commands));

    commands
}
