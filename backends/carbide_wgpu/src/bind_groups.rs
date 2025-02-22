use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device};

use carbide_core::draw::Scalar;
use carbide_core::math::Matrix4;

pub(crate) fn create_uniform_bind_group(
    device: &Device,
    uniform_bind_group_layout: &BindGroupLayout,
    uniform_buffer: Buffer,
) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding()),
        }],
        label: Some("carbide_uniform_bind_group"),
    })
}

pub(crate) fn uniforms_to_bind_group(
    device: &Device,
    uniform_bind_group_layout: &BindGroupLayout,
    matrix: Matrix4<f32>,
    hue_rotation: f32,
    saturation_shift: f32,
    luminance_shift: f32,
    color_invert: bool,
) -> BindGroup {
    let uniforms: [[f32; 4]; 4] = matrix.into();

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("carbide_uniform_buffer"),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let color_filter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("carbide_color_filter_buffer"),
        contents: bytemuck::cast_slice(&[
            bytemuck::cast::<f32, [u8; 4]>(hue_rotation),
            bytemuck::cast::<f32, [u8; 4]>(saturation_shift),
            bytemuck::cast::<f32, [u8; 4]>(luminance_shift),
            bytemuck::cast::<u32, [u8; 4]>(if color_invert { 1u32 } else { 0u32 })
        ]),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(color_filter_buffer.as_entire_buffer_binding()),
            },
        ],
        label: Some("carbide_uniform_bind_group"),
    })
}


pub(crate) fn size_to_uniform_bind_group(
    device: &Device,
    uniform_bind_group_layout: &BindGroupLayout,
    width: Scalar,
    height: Scalar,
    scale_factor: Scalar,
) -> BindGroup {
    let uniforms: [f32; 2] = [
        (width / scale_factor) as f32,
        (height / scale_factor) as f32,
    ];

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("carbide_uniform_buffer"),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let uniform_bind_group = create_uniform_bind_group(device, uniform_bind_group_layout, uniform_buffer);

    uniform_bind_group
}

pub(crate) fn filter_buffer_bind_group(
    device: &Device,
    filter_bind_group_layout: &BindGroupLayout,
    buffer: &Buffer,
) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &filter_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
        }],
        label: Some("carbide_filter_bind_group"),
    })
}

pub(crate) fn gradient_dashes_bind_group(
    device: &Device,
    gradient_bind_group_layout: &BindGroupLayout,
    gradient_buffer: &Buffer,
    dashes_buffer: &Buffer,
) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &gradient_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(gradient_buffer.as_entire_buffer_binding()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(dashes_buffer.as_entire_buffer_binding()),
            },
        ],
        label: Some("carbide_gradient_dashes_bind_group"),
    })
}
