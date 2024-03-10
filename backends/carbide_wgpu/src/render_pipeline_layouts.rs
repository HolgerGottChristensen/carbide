use wgpu::{BindGroupLayout, Device, PipelineLayout};

pub struct RenderPipelines {
    pub(crate) render_pipeline_no_mask: wgpu::RenderPipeline,
    pub(crate) render_pipeline_add_mask: wgpu::RenderPipeline,
    pub(crate) render_pipeline_in_mask: wgpu::RenderPipeline,
    pub(crate) render_pipeline_remove_mask: wgpu::RenderPipeline,

    /// This is used when applying normal filter, or in the second pass of the of the two pass filter
    pub(crate) render_pipeline_in_mask_filter: wgpu::RenderPipeline,
    pub(crate) render_pipeline_no_mask_filter: wgpu::RenderPipeline,
}


pub(crate) fn main_pipeline_layout(
    device: &Device,
    main_bind_group_layout: &BindGroupLayout,
    uniform_bind_group_layout: &BindGroupLayout,
    gradient_bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[main_bind_group_layout, uniform_bind_group_layout, gradient_bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub(crate) fn gradient_pipeline_layout(
    device: &Device,
    gradient_bind_group_layout: &BindGroupLayout,
    uniform_bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Gradient render Pipeline Layout"),
        bind_group_layouts: &[gradient_bind_group_layout, uniform_bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub(crate) fn filter_pipeline_layout(
    device: &Device,
    filter_texture_bind_group_layout: &BindGroupLayout,
    filter_buffer_bind_group_layout: &BindGroupLayout,
    uniform_bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &filter_texture_bind_group_layout,
            &filter_buffer_bind_group_layout,
            &uniform_bind_group_layout,
            &uniform_bind_group_layout,
        ],
        push_constant_ranges: &[],
    })
}
