use std::collections::HashMap;

use cgmath::Matrix4;
use wgpu::{BindGroupLayout, Device, Texture};

use carbide_core::image_map::{Id, ImageMap};
use carbide_core::mesh::mesh;
use carbide_core::mesh::mesh::Mesh;

use crate::bind_groups::matrix_to_uniform_bind_group;
use crate::diffuse_bind_group::{DiffuseBindGroup, new_diffuse};
use crate::image::Image;
use crate::window::Window;

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
    Filter(std::ops::Range<u32>, u32),
    FilterSplitPt1(std::ops::Range<u32>, u32),
    FilterSplitPt2(std::ops::Range<u32>, u32),
}

#[derive(PartialEq)]
enum BindGroup {
    Default,
    Image(carbide_core::image_map::Id),
}

pub fn create_render_pass_commands<'a>(
    default_bind_group: &'a wgpu::BindGroup,
    bind_groups: &'a mut HashMap<Id, DiffuseBindGroup>,
    uniform_bind_groups: &mut Vec<wgpu::BindGroup>,
    image_map: &'a ImageMap<Image>,
    mesh: &'a Mesh,
    device: &'a Device,
    glyph_texture: &'a Texture,
    atlas_tex: &'a Texture,
    bind_group_layout: &'a BindGroupLayout,
    uniform_bind_group_layout: &'a BindGroupLayout,
    carbide_to_wgpu_matrix: Matrix4<f32>,
) -> Vec<RenderPass<'a>> {
    bind_groups.retain(|k, _| image_map.contains_key(k));

    for (id, img) in image_map.iter() {
        // If we already have a bind group for this image move on.
        if bind_groups.contains_key(id) {
            continue;
        }

        // Create the bind
        let bind_group = new_diffuse(
            &device,
            &img,
            &atlas_tex,
            &bind_group_layout,
        );
        bind_groups.insert(*id, bind_group);
    }

    let mut commands = vec![];
    let mut inner_commands = vec![];

    let mut bind_group = None;

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
                if bind_group.is_none() {
                    bind_group = Some(BindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: default_bind_group,
                    };
                    inner_commands.push(cmd);
                }

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::Filter(range, filter_id));
                bind_group = None;
            }
            mesh::Command::FilterSplitPt1(vertex_range, filter_id) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if bind_group.is_none() {
                    bind_group = Some(BindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: default_bind_group,
                    };
                    inner_commands.push(cmd);
                }

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::FilterSplitPt1(range, filter_id));
                bind_group = None;
            }
            mesh::Command::FilterSplitPt2(vertex_range, filter_id) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if bind_group.is_none() {
                    bind_group = Some(BindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: default_bind_group,
                    };
                    inner_commands.push(cmd);
                }

                let range = vertex_range.start as u32..vertex_range.end as u32;
                let mut new_inner_commands = vec![];
                std::mem::swap(&mut new_inner_commands, &mut inner_commands);
                commands.push(RenderPass::Normal(new_inner_commands));
                commands.push(RenderPass::FilterSplitPt2(range, filter_id));
                bind_group = None;
            }

            mesh::Command::Stencil(vertex_range) => {
                let vertex_count = vertex_range.len();
                if vertex_count <= 0 {
                    continue;
                }
                // Ensure a render pipeline and bind group is set.
                if bind_group.is_none() {
                    bind_group = Some(BindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: default_bind_group,
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
                if bind_group.is_none() {
                    bind_group = Some(BindGroup::Default);
                    let cmd = RenderPassCommand::SetBindGroup {
                        bind_group: default_bind_group,
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
                let new_bind_group = matrix_to_uniform_bind_group(device, uniform_bind_group_layout, transformed_matrix);

                inner_commands.push(RenderPassCommand::Transform { uniform_bind_group_index: uniform_bind_groups.len() });
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
                    if bind_group.is_none() {
                        bind_group = Some(BindGroup::Default);
                        let cmd = RenderPassCommand::SetBindGroup {
                            bind_group: default_bind_group,
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
                    let expected_bind_group = Some(BindGroup::Image(image_id));
                    if bind_group != expected_bind_group {
                        // Now update the bind group and add the new bind group command.
                        bind_group = expected_bind_group;
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
            },
        }
    }

    commands.push(RenderPass::Normal(inner_commands));

    commands
}
