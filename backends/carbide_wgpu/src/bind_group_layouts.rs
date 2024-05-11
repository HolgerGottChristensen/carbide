use once_cell::sync::Lazy;
use wgpu::{
    BindGroupLayout, BufferBindingType, Device, SamplerBindingType, TextureSampleType,
    TextureViewDimension,
};
use crate::application::DEVICE;

pub(crate) static UNIFORM_BIND_GROUP_LAYOUT: Lazy<BindGroupLayout> = Lazy::new(|| {
    uniform_bind_group_layout(&DEVICE)
});

pub(crate) static UNIFORM_BIND_GROUP_LAYOUT2: Lazy<BindGroupLayout> = Lazy::new(|| {
    uniform_bind_group_layout2(&DEVICE)
});

pub(crate) static FILTER_TEXTURE_BIND_GROUP_LAYOUT: Lazy<BindGroupLayout> = Lazy::new(|| {
    filter_texture_bind_group_layout(&DEVICE)
});

pub(crate) static FILTER_BUFFER_BIND_GROUP_LAYOUT: Lazy<BindGroupLayout> = Lazy::new(|| {
    filter_buffer_bind_group_layout(&DEVICE)
});

pub(crate) static MAIN_TEXTURE_BIND_GROUP_LAYOUT: Lazy<BindGroupLayout> = Lazy::new(|| {
    main_bind_group_layout(&DEVICE)
});

pub(crate) static GRADIENT_DASHES_BIND_GROUP_LAYOUT: Lazy<BindGroupLayout> = Lazy::new(|| {
    gradient_buffer_bind_group_layout(&DEVICE)
});

pub(crate) static ATLAS_BIND_GROUP_LAYOUT: Lazy<BindGroupLayout> = Lazy::new(|| {
    atlas_bind_group_layout(&DEVICE)
});

pub(crate) fn uniform_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    min_binding_size: None,
                    has_dynamic_offset: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    min_binding_size: None,
                    has_dynamic_offset: false,
                },
                count: None,
            },
        ],
        label: Some("uniform_bind_group_layout"),
    })
}

pub(crate) fn uniform_bind_group_layout2(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    min_binding_size: None,
                    has_dynamic_offset: false,
                },
                count: None,
            },
        ],
        label: Some("uniform_bind_group_layout2"),
    })
}

pub(crate) fn filter_texture_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("uniform_bind_group_layout"),
    })
}

pub(crate) fn filter_buffer_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                min_binding_size: None,
                has_dynamic_offset: false,
            },
            count: None,
        }],
        label: Some("uniform_bind_group_layout"),
    })
}

pub(crate) fn gradient_buffer_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    min_binding_size: None,
                    has_dynamic_offset: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    min_binding_size: None,
                    has_dynamic_offset: false,
                },
                count: None,
            },
        ],
        label: Some("gradient_buffer_bind_group_layout"),
    })
}

pub(crate) fn atlas_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
        label: Some("atlas_bind_group_layout"),
    })
}


pub(crate) fn main_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            }
        ],
        label: Some("texture_bind_group_layout"),
    })
}
