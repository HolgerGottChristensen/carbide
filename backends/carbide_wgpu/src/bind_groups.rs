use cgmath::Matrix4;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Sampler, Texture, TextureView};
use wgpu::util::DeviceExt;

use carbide_core::Scalar;

pub(crate) fn uniform_bind_group(device: &Device, uniform_bind_group_layout: &BindGroupLayout, uniform_buffer: Buffer) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding()),
            }
        ],
        label: Some("uniform_bind_group"),
    })
}

pub(crate) fn matrix_to_uniform_bind_group(device: &Device, uniform_bind_group_layout: &BindGroupLayout, matrix: Matrix4<f32>) -> BindGroup {
    let uniforms: [[f32; 4]; 4] = matrix.into();

    let uniform_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM,
        }
    );

    let uniform_bind_group = uniform_bind_group(device, uniform_bind_group_layout, uniform_buffer);

    uniform_bind_group
}

pub(crate) fn size_to_uniform_bind_group(device: &Device, uniform_bind_group_layout: &BindGroupLayout, width: Scalar, height: Scalar, scale_factor: Scalar) -> BindGroup {
    let uniforms: [f32; 2] = [(width / scale_factor) as f32, (height / scale_factor) as f32];

    let uniform_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM,
        }
    );

    let uniform_bind_group = uniform_bind_group(device, uniform_bind_group_layout, uniform_buffer);

    uniform_bind_group
}

pub(crate) fn filter_texture_bind_group(device: &Device, filter_bind_group_layout: &BindGroupLayout, texture: &TextureView, sampler: &Sampler) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &filter_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            }
        ],
        label: Some("filter_texture_bind_group"),
    })
}

pub(crate) fn filter_buffer_bind_group(device: &Device, filter_bind_group_layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &filter_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }
        ],
        label: Some("filter_bind_group"),
    })
}

pub(crate) fn main_bind_group(device: &Device, main_bind_group_layout: &BindGroupLayout, texture: &TextureView, sampler: &Sampler, atlas_texture: &Texture) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &main_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(
                    &atlas_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
        ],
        label: Some("diffuse_bind_group"),
    })
}