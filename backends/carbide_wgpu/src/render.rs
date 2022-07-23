use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsages, Extent3d, ImageCopyTexture, LoadOp, Operations, RenderPassDepthStencilAttachment,
};

use carbide_core::draw::{Dimension, Position, Rect};

use crate::bind_groups::filter_buffer_bind_group;
use crate::filter::Filter;
use crate::render_pass_command::{create_render_pass_commands, RenderPass, RenderPassCommand};
use crate::texture_atlas_command::TextureAtlasCommand;
use crate::vertex::Vertex;
use crate::window::Window;

impl Window {
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                    width,
                    height,
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

        let keys = self
            .ui
            .environment
            .filters()
            .keys()
            .cloned()
            .collect::<Vec<_>>();

        self.filter_buffer_bind_groups
            .retain(|id, _| keys.contains(id));

        for (filter_id, filter) in self.ui.environment.filters() {
            if !self.filter_buffer_bind_groups.contains_key(filter_id) {
                let filter: Filter = filter.clone().into();
                let filter_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Filter Buffer"),
                            contents: &*filter.as_bytes(),
                            usage: wgpu::BufferUsages::STORAGE,
                        });
                let filter_buffer_bind_group = filter_buffer_bind_group(
                    &self.device,
                    &self.filter_buffer_bind_group_layout,
                    &filter_buffer,
                );
                self.filter_buffer_bind_groups
                    .insert(*filter_id, filter_buffer_bind_group);
            }
        }

        let commands = create_render_pass_commands(
            &self.diffuse_bind_group,
            &mut self.bind_groups,
            &mut uniform_bind_groups,
            &self.image_map,
            &self.mesh,
            &self.device,
            &self.atlas_cache_tex,
            &self.texture_bind_group_layout,
            &self.uniform_bind_group_layout,
            &self.gradient_bind_group_layout,
            self.carbide_to_wgpu_matrix,
        );

        let vertices: Vec<Vertex> = self
            .mesh
            .vertices()
            .iter()
            .map(|v| Vertex::from(*v))
            .collect::<Vec<_>>();

        if vertices.len() <= self.vertex_buffer.1 {
            // There is space in the current vertex buffer
            self.queue
                .write_buffer(&self.vertex_buffer.0, 0, bytemuck::cast_slice(&vertices));
        } else {
            // We need to create a new and larger vertex buffer
            let new_vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            self.vertex_buffer = (new_vertex_buffer, vertices.len());
        }

        let instance_range = 0..1;
        let mut stencil_level = 0;
        let mut first_pass = true;

        let mut current_main_render_pipeline = &self.render_pipelines.render_pipeline_no_mask;
        let current_vertex_buffer_slice = self.vertex_buffer.0.slice(..);
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
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });

                    render_pass.set_stencil_reference(stencil_level);
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
                                render_pass.set_pipeline(&self.render_pipelines.render_pipeline_add_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
                                current_main_render_pipeline = &self.render_pipelines.render_pipeline_in_mask;
                                render_pass.set_pipeline(current_main_render_pipeline);
                                render_pass.set_stencil_reference(stencil_level);
                            }
                            RenderPassCommand::DeStencil { vertex_range } => {
                                stencil_level -= 1;
                                render_pass.set_pipeline(&self.render_pipelines.render_pipeline_remove_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
                                render_pass.set_stencil_reference(stencil_level);
                                if stencil_level == 0 {
                                    current_main_render_pipeline = &self.render_pipelines.render_pipeline_no_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                } else {
                                    current_main_render_pipeline = &self.render_pipelines.render_pipeline_in_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                }
                            }
                            RenderPassCommand::Transform {
                                uniform_bind_group_index,
                            } => {
                                current_uniform_bind_group =
                                    &uniform_bind_groups[uniform_bind_group_index];
                                render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                            }
                        }
                    }
                }
                RenderPass::Gradient(vertex_range, bind_group_index) => {
                    let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });

                    render_pass.set_pipeline(&self.render_pipelines.render_pipeline_in_mask_gradient);
                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &uniform_bind_groups[bind_group_index], &[]);
                    render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                    render_pass.draw(vertex_range, instance_range.clone());
                }
                RenderPass::Filter(vertex_range, bind_group_index) => {
                    encoder.copy_texture_to_texture(
                        ImageCopyTexture {
                            texture: &self.main_tex,
                            mip_level: 0,
                            origin: Default::default(),
                            aspect: Default::default(),
                        },
                        ImageCopyTexture {
                            texture: &self.secondary_tex,
                            mip_level: 0,
                            origin: Default::default(),
                            aspect: Default::default(),
                        },
                        Extent3d {
                            width: self.size.width,
                            height: self.size.height,
                            depth_or_array_layers: 1,
                        },
                    );

                    let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });
                    render_pass.set_pipeline(&self.render_pipelines.render_pipeline_in_mask_filter);
                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &self.filter_secondary_texture_bind_group, &[]);
                    render_pass.set_bind_group(
                        1,
                        &self
                            .filter_buffer_bind_groups
                            .get(&bind_group_index)
                            .unwrap(),
                        &[],
                    );
                    render_pass.set_bind_group(2, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(3, &self.texture_size_bind_group, &[]);
                    render_pass.draw(vertex_range, instance_range.clone());
                }
                RenderPass::FilterSplitPt1(vertex_range, filter_id) => {
                    let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.secondary_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });
                    render_pass.set_pipeline(&self.render_pipelines.render_pipeline_no_mask_filter);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &self.filter_main_texture_bind_group, &[]);
                    render_pass.set_bind_group(
                        1,
                        &self.filter_buffer_bind_groups.get(&filter_id).unwrap(),
                        &[],
                    );
                    render_pass.set_bind_group(2, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(3, &self.texture_size_bind_group, &[]);
                    render_pass.draw(vertex_range, instance_range.clone());
                }
                RenderPass::FilterSplitPt2(vertex_range, filter_id) => {
                    let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });
                    render_pass.set_pipeline(&self.render_pipelines.render_pipeline_in_mask_filter);
                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &self.filter_secondary_texture_bind_group, &[]);
                    render_pass.set_bind_group(
                        1,
                        &self.filter_buffer_bind_groups.get(&filter_id).unwrap(),
                        &[],
                    );
                    render_pass.set_bind_group(2, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(3, &self.texture_size_bind_group, &[]);
                    render_pass.draw(vertex_range, instance_range.clone());
                }
            };
        }

        // Render from the texture to the swap chain
        let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);

        // This blocks until a new frame is available.
        let output = self.surface.get_current_texture()?;
        let frame_view = output.texture.create_view(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame_view, // Here is the render target
                resolve_target: None,
                ops: color_op,
            }],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: None,
                stencil_ops: Some(stencil_op),
            }),
        });

        render_pass.set_pipeline(&self.render_pipelines.render_pipeline_no_mask);
        render_pass.set_vertex_buffer(0, self.second_vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.main_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.draw(0..6, instance_range);

        drop(render_pass);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}