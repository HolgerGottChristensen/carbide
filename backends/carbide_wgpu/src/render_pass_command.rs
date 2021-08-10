use std::collections::HashMap;

use wgpu::{BindGroupLayout, Device, Texture};

use carbide_core::image_map::{Id, ImageMap};
use carbide_core::mesh::mesh;
use carbide_core::mesh::mesh::Mesh;

use crate::diffuse_bind_group::{DiffuseBindGroup, new_diffuse};
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
    /// A new image requires drawing and in turn a new bind group requires setting.
    SetBindGroup {
        bind_group: &'a wgpu::BindGroup,
    },
}

#[derive(PartialEq)]
enum BindGroup {
    Default,
    Image(carbide_core::image_map::Id),
}

pub fn create_render_pass_commands<'a>(
    def_bind_group: &'a wgpu::BindGroup,
    bind_groups: &'a mut HashMap<Id, DiffuseBindGroup>,
    image_map: &'a ImageMap<Image>,
    mesh: &'a Mesh,
    device: &'a Device,
    glyph_texture: &'a Texture,
    atlas_tex: &'a Texture,
    bind_group_layout: &'a BindGroupLayout,
) -> Vec<RenderPassCommand<'a>> {
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
            &glyph_texture,
            &atlas_tex,
            &bind_group_layout,
        );
        bind_groups.insert(*id, bind_group);
    }

    let mut commands = vec![];

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
                commands.push(cmd);
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
                        bind_group: def_bind_group,
                    };
                    commands.push(cmd);
                }
                let cmd = RenderPassCommand::Stencil {
                    vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                };
                commands.push(cmd);
            }

            mesh::Command::DeStencil(vertex_range) => {
                let cmd = RenderPassCommand::DeStencil {
                    vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                };
                commands.push(cmd);
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
                            bind_group: def_bind_group,
                        };
                        commands.push(cmd);
                    }
                    let cmd = RenderPassCommand::Draw {
                        vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                    };
                    commands.push(cmd);
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
                        commands.push(cmd);
                    }
                    let cmd = RenderPassCommand::Draw {
                        vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                    };
                    commands.push(cmd);
                }
            },
        }
    }

    commands
}
