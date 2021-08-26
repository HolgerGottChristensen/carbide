use wgpu::{BindGroupLayout, Device, PipelineLayout};

pub(crate) fn main_pipeline_layout(device: &Device, main_bind_group_layout: &BindGroupLayout, uniform_bind_group_layout: &BindGroupLayout) -> PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            main_bind_group_layout,
            uniform_bind_group_layout,
        ],
        push_constant_ranges: &[],
    })
}

pub(crate) fn filter_pipeline_layout(device: &Device, filter_bind_group_layout: &BindGroupLayout, uniform_bind_group_layout: &BindGroupLayout) -> PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &filter_bind_group_layout,
            &uniform_bind_group_layout,
        ],
        push_constant_ranges: &[],
    })
}