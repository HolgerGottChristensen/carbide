use carbide_3d::{InnerRenderContext3d, Mesh, Object};
use carbide_core::render::{InnerLayer, Layer};
use std::any::Any;
use encase::{ArrayLength, ShaderSize, ShaderType};
use wgpu::{AddressMode, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, CompareFunction, DepthBiasState, DepthStencilState, Device, FilterMode, FragmentState, FrontFace, IndexFormat, Operations, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassDepthStencilAttachment, SamplerBindingType, SamplerDescriptor, ShaderStages, StencilFaceState, StencilOperation, StencilState, Texture, TextureFormat, TextureView, VertexState};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use carbide_3d::camera::Camera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::Material;
use carbide_core::color::{Color, ColorExt};
use carbide_core::render::matrix::{Matrix4, SquareMatrix, Transform, Vector2, Vector3, Vector4, Zero};
use carbide_wgpu::{DEVICE, QUEUE, RenderTarget};
use crate::camera::WgpuCamera;
use crate::directional_light::{WgpuDirectionalLight, WgpuDirectionalLightBuffer};
use crate::material::WgpuMaterial;
use crate::object::WgpuObject;
use crate::pbr_material::WgpuPbrMaterial;
use crate::point_light::WgpuPointLightBuffer;
use crate::render_pass_command::RenderPassCommand;
use crate::SHADER;
use crate::storage_buffer::StorageBuffer;
use crate::uniforms::WgpuUniforms;
use crate::vertex::WgpuVertex;

pub(crate) fn render_context_3d_initializer() -> Box<dyn InnerRenderContext3d> {
    Box::new(WGPURenderContext3d::new())
}

#[derive(Debug)]
pub struct WGPURenderContext3d {
    material_stack: Vec<WgpuMaterial>,
    materials: Vec<WgpuMaterial>,
    current_material: Option<(WgpuMaterial, usize)>,

    transform_stack: Vec<(Matrix4<f32>, usize)>,
    objects: Vec<WgpuObject>,

    directional_lights: Vec<WgpuDirectionalLight>,

    depth_stencil_texture: Texture,
    depth_stencil_view: TextureView,
    vertices: Vec<WgpuVertex>,
    indices: Vec<u32>,

    commands: Vec<RenderPassCommand>,
}



impl WGPURenderContext3d {
    pub fn new() -> Self {
        let depth_texture = DEVICE.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture descriptor"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = depth_texture.create_view(&Default::default());

        WGPURenderContext3d {
            material_stack: vec![],
            materials: vec![],
            current_material: None,
            transform_stack: vec![
                (Matrix4::identity(), 0)
            ],
            objects: vec![],
            directional_lights: vec![],
            depth_stencil_texture: depth_texture,
            depth_stencil_view: view,
            vertices: vec![],
            indices: vec![],
            commands: vec![],
        }
    }

    fn clear(&mut self) {
        self.material_stack.clear();
        self.current_material = None;
        self.materials.clear();

        self.transform_stack.clear();
        self.transform_stack.push((Matrix4::identity(), 0));
        self.objects.clear();
        self.directional_lights.clear();

        self.vertices.clear();
        self.indices.clear();
        self.commands.clear();
    }

    fn ensure_current_object(&mut self) {
        if let Some(object) = self.objects.last() {
            if object.material_index == self.current_material.as_ref().unwrap().1 as u32 && self.transform_stack.last().unwrap().0 == object.transform {
                return;
            }
        }

        self.objects.push(WgpuObject {
            transform: self.transform_stack.last().unwrap().0,
            material_index: self.current_material.as_ref().unwrap().1 as u32,
        })
    }

    fn ensure_current_material(&mut self, material: &WgpuMaterial) {
        let needs_update = if let Some((current, _)) = &self.current_material {
            current != material
        } else {
            true
        };

        if needs_update {
            self.materials.push((*material).clone());
            self.current_material = Some(((*material).clone(), self.materials.len() - 1));
        }
    }

    fn push_command(&mut self, command: RenderPassCommand) {
        self.commands.push(command);
    }
}

impl InnerRenderContext3d for WGPURenderContext3d {
    fn start(&mut self) {
        self.clear();
    }

    fn render(&mut self, layer: Layer, camera: &dyn Camera) {
        let render_target = layer.inner.downcast_ref::<RenderTarget>().expect("The layer is not compatible with carbide wgpu 3d");

        let dimension = self.depth_stencil_texture.size();

        if render_target.dimensions().0 != dimension.width || render_target.dimensions().1 != dimension.height {
            let depth_texture = DEVICE.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth texture descriptor"),
                size: wgpu::Extent3d {
                    width: render_target.dimensions().0,
                    height: render_target.dimensions().1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            self.depth_stencil_view = depth_texture.create_view(&Default::default());
            self.depth_stencil_texture = depth_texture;
        }

        let mut encoder = DEVICE.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let bind_group_layout = DEVICE.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group_layout2 = DEVICE.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = DEVICE.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout,
                &bind_group_layout2,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = DEVICE.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &SHADER,
                entry_point: "main_vs",
                buffers: &[WgpuVertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &SHADER,
                entry_point: "main_fs",
                targets: &[Some(ColorTargetState {
                    format: render_target.texture_format(),
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let sampler = DEVICE.create_sampler(&SamplerDescriptor {
            label: Some("linear"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let pbr_materials = self.materials.iter().cloned().map(|a| match a { WgpuMaterial::PBR(m) => m }).collect::<Vec<_>>();

        //println!("{:#?}", pbr_materials);
        //println!("{:#?}", self.objects);
        //println!("{:#?}", self.vertices);
        //println!("{:#?}", self.indices);
        let mut materials_buffer = StorageBuffer::from(pbr_materials);
        let mut objects_buffer = StorageBuffer::from(self.objects.clone());
        let mut camera_buffer = StorageBuffer::from(WgpuCamera {
            view: camera.view(),
            view_proj: camera.view_projection(1.0),
            orig_view: Matrix4::from_cols(
                camera.view().x,
                camera.view().y,
                camera.view().z,
                Vector4::unit_w(),
            ),
            inv_view: camera.view().invert().unwrap(),
            aspect_ratio: 1.0,
        });
        let mut uniforms = StorageBuffer::from(WgpuUniforms {
            ambient: Vector4::new(0.1, 0.1, 0.1, 1.0)
        });

        let mut directional_lights = StorageBuffer::from(WgpuDirectionalLightBuffer {
            count: ArrayLength,
            array: self.directional_lights.clone()
        });
        let mut point_lights = StorageBuffer::from(WgpuPointLightBuffer {
            count: ArrayLength,
            array: vec![],
        });


        materials_buffer.write_buffer(&DEVICE, &QUEUE);
        objects_buffer.write_buffer(&DEVICE, &QUEUE);
        camera_buffer.write_buffer(&DEVICE, &QUEUE);
        directional_lights.write_buffer(&DEVICE, &QUEUE);
        point_lights.write_buffer(&DEVICE, &QUEUE);
        uniforms.write_buffer(&DEVICE, &QUEUE);

        let material_bind_group = DEVICE.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: materials_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: objects_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: camera_buffer.binding().unwrap(),
                }
            ],
        });

        let bind_group2 = DEVICE.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout2,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: directional_lights.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: point_lights.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: uniforms.binding().unwrap(),
                },
            ],
        });

        let new_vertex_buffer = DEVICE.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        let new_index_buffer = DEVICE.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: render_target.view(),
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_stencil_view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: true,
                }),
                stencil_ops: Some(Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_vertex_buffer(0, new_vertex_buffer.slice(..));
        render_pass.set_index_buffer(new_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.set_bind_group(0, &material_bind_group, &[]);
        render_pass.set_bind_group(1, &bind_group2, &[]);

        //println!("{:#?}", self.commands);

        for command in &self.commands {
            match command {
                RenderPassCommand::DrawIndexed(range) => {
                    render_pass.draw_indexed(range.clone(), 0, 0..1);
                }
            }
        }

        drop(render_pass);
        QUEUE.submit(std::iter::once(encoder.finish()));
    }

    fn mesh(&mut self, mesh: &Mesh) {
        let material = self.material_stack.last().unwrap().clone();

        self.ensure_current_material(&material);
        self.ensure_current_object();

        let index = self.indices.len();

        let object_index = self.objects.len() as u32 - 1;

        let indices_offset = self.vertices.len() as u32;
        self.vertices.extend(mesh.vertices.iter().map(|v| WgpuVertex::from_vertex(v, object_index)));
        self.indices.extend(mesh.indices.iter().map(|i| i + indices_offset));

        self.push_command(RenderPassCommand::DrawIndexed(index as u32..self.indices.len() as u32));
    }

    fn material(&mut self, material: &Material) {
        self.material_stack.push(
        match material {
                Material::PBR(material) => WgpuMaterial::PBR(WgpuPbrMaterial::from_material(material))
            }
        )
    }

    fn pop_material(&mut self) {
        self.material_stack.pop();
    }

    fn transform(&mut self, transform: &Matrix4<f32>) {
        let (latest_transform, _) = &self.transform_stack[self.transform_stack.len() - 1];

        let new_transform = latest_transform * transform;
        let index = self.objects.len();

        self.transform_stack.push((new_transform, index));
    }

    fn pop_transform(&mut self) {
        self.transform_stack.pop();
    }

    fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>) {
        self.directional_lights.push(WgpuDirectionalLight {
            view_proj: Matrix4::identity(),
            color: Vector3::new(color.red(), color.green(), color.blue()) * intensity,
            direction,
            inv_resolution: Vector2::zero(),
            atlas_offset: Vector2::zero(),
            atlas_size: Vector2::zero(),
        })
    }
}

impl Clone for WGPURenderContext3d {
    fn clone(&self) -> Self {
        todo!()
    }
}