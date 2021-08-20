use std::time::Instant;

use wgpu::{Extent3d, LoadOp, Operations, Origin3d, RenderPassDepthStencilAttachmentDescriptor, TextureCopyView};
use wgpu::util::DeviceExt;

use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::mesh::MODE_IMAGE;

use crate::glyph_cache_command::GlyphCacheCommand;
use crate::render_pass_command::{create_render_pass_commands, RenderPass, RenderPassCommand};
use crate::texture_atlas_command::TextureAtlasCommand;
use crate::vertex::Vertex;
use crate::window::Window;

impl Window {
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        // This blocks until a new frame is available.
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let primitives = self.ui.draw();
        let fill = self
            .mesh
            .fill(
                Rect::new(
                    Position::new(0.0, 0.0),
                    Dimension::new(self.size.width as f64, self.size.height as f64),
                ),
                &mut self.ui.environment,
                &self.image_map,
                primitives,
            )
            .unwrap();

        // Check if an upload to texture atlas is needed.
        let texture_atlas_cmd = match fill.atlas_requires_upload {
            true => {
                let width = self.mesh.texture_atlas().width();
                let height = self.mesh.texture_atlas().height();
                Some(TextureAtlasCommand {
                    texture_atlas_buffer: self.mesh.texture_atlas_image_as_bytes(),
                    texture_atlas_texture: &self.atlas_cache_tex,
                    width: 512,
                    height: 512,
                })
            }
            false => None,
        };

        match texture_atlas_cmd {
            None => (),
            Some(cmd) => {
                cmd.load_buffer_and_encode(&self.device, &mut encoder);
            }
        }

        let mut uniform_bind_groups = vec![];

        let commands = create_render_pass_commands(
            &self.diffuse_bind_group,
            &mut self.bind_groups,
            &mut uniform_bind_groups,
            &self.image_map,
            &self.mesh,
            &self.device,
            &self.glyph_cache_tex,
            &self.atlas_cache_tex,
            &self.texture_bind_group_layout,
            &self.uniform_bind_group_layout,
            self.carbide_to_wgpu_matrix,
        );

        let vertices: Vec<Vertex> = self
            .mesh
            .vertices()
            .iter()
            .map(|v| Vertex::from(*v))
            .collect::<Vec<_>>();

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let instance_range = 0..1;
        let mut stencil_level = 0;
        let mut first_pass = true;

        let mut current_main_render_pipeline = &self.render_pipeline_no_mask;
        let mut current_vertex_buffer_slice = vertex_buffer.slice(..);
        let mut current_uniform_bind_group = &self.uniform_bind_group;

        for command in commands {
            match command {
                RenderPass::Normal(inner) => {
                    if inner.len() == 0 {
                        continue;
                    }
                    let (color_op, stencil_op) = if first_pass {
                        first_pass = false;
                        render_pass_ops(RenderPassOps::Start)
                    } else {
                        render_pass_ops(RenderPassOps::Middle)
                    };
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });

                    render_pass.set_pipeline(current_main_render_pipeline);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(1, current_uniform_bind_group, &[]);

                    for inner_command in inner {
                        match inner_command {
                            RenderPassCommand::SetBindGroup { bind_group } => {
                                render_pass.set_bind_group(0, bind_group, &[]);
                            }
                            RenderPassCommand::SetScissor {
                                top_left,
                                dimensions,
                            } => {
                                let [x, y] = top_left;
                                let [w, h] = dimensions;
                                render_pass.set_scissor_rect(x, y, w, h);
                            }
                            RenderPassCommand::Draw { vertex_range } => {
                                render_pass.draw(vertex_range, instance_range.clone());
                            }
                            RenderPassCommand::Stencil { vertex_range } => {
                                stencil_level += 1;
                                render_pass.set_pipeline(&self.render_pipeline_add_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
                                current_main_render_pipeline = &self.render_pipeline_in_mask;
                                render_pass.set_pipeline(current_main_render_pipeline);
                                render_pass.set_stencil_reference(stencil_level);
                            }
                            RenderPassCommand::DeStencil { vertex_range } => {
                                stencil_level -= 1;
                                render_pass.set_pipeline(&self.render_pipeline_remove_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
                                render_pass.set_stencil_reference(stencil_level);
                                if stencil_level == 0 {
                                    current_main_render_pipeline = &self.render_pipeline_no_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                } else {
                                    current_main_render_pipeline = &self.render_pipeline_in_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                }
                            }
                            RenderPassCommand::Transform { uniform_bind_group_index } => {
                                current_uniform_bind_group = &uniform_bind_groups[uniform_bind_group_index];
                                render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                            }
                        }
                    }
                }
                RenderPass::Filter(vertex_range) => {
                    encoder.copy_texture_to_texture(TextureCopyView {
                        texture: &self.main_tex,
                        mip_level: 0,
                        origin: Default::default(),
                    }, TextureCopyView {
                        texture: &self.secondary_tex,
                        mip_level: 0,
                        origin: Default::default(),
                    }, Extent3d {
                        width: self.size.width,
                        height: self.size.height,
                        depth: 1,
                    });

                    let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });
                    render_pass.set_pipeline(&self.render_pipeline_in_mask_filter);
                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(0, &self.secondary_bind_group, &[]);
                    render_pass.draw(vertex_range, instance_range.clone());
                }
            };
        }

        // Render from the texture to the swap chain

        let last_verts: Vec<Vertex> = vec![
            Vertex::new_from_2d(0.0, 0.0, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(self.size.width as f32 / self.ui.environment.get_scale_factor() as f32, 0.0, [0.0, 0.0, 0.0, 0.0], [1.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(0.0, self.size.height as f32 / self.ui.environment.get_scale_factor() as f32, [0.0, 0.0, 0.0, 0.0], [0.0, 1.0], MODE_IMAGE),
            Vertex::new_from_2d(self.size.width as f32 / self.ui.environment.get_scale_factor() as f32, 0.0, [0.0, 0.0, 0.0, 0.0], [1.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(self.size.width as f32 / self.ui.environment.get_scale_factor() as f32, self.size.height as f32 / self.ui.environment.get_scale_factor() as f32, [0.0, 0.0, 0.0, 0.0], [1.0, 1.0], MODE_IMAGE),
            Vertex::new_from_2d(0.0, self.size.height as f32 / self.ui.environment.get_scale_factor() as f32, [0.0, 0.0, 0.0, 0.0], [0.0, 1.0], MODE_IMAGE),
        ];
        let last_verts_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&last_verts),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view, // Here is the render target
                resolve_target: None,
                ops: color_op,
            }],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture_view,
                depth_ops: None,
                stencil_ops: Some(stencil_op),
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline_no_mask);
        render_pass.set_vertex_buffer(0, last_verts_buffer.slice(..));
        render_pass.set_bind_group(0, &self.main_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.draw(0..6, instance_range);

        drop(render_pass);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}

enum RenderPassOps {
    Start,
    Middle,
}

fn render_pass_ops(ops_type: RenderPassOps) -> (Operations<wgpu::Color>, Operations<u32>) {
    let color_op = match ops_type {
        RenderPassOps::Start => {
            wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                store: true,
            }
        }
        RenderPassOps::Middle => {
            wgpu::Operations {
                load: LoadOp::Load,
                store: true,
            }
        }
    };

    let stencil_op = match ops_type {
        RenderPassOps::Start => {
            wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: true,
            }
        }
        RenderPassOps::Middle => {
            wgpu::Operations {
                load: LoadOp::Load,
                store: true,
            }
        }
    };

    (color_op, stencil_op)
}