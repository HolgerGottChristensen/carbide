/// Data that must be unique per `wgpu::TextureComponentType`, i.e. bind group layout and render
/// pipeline.
pub struct Pipeline {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}