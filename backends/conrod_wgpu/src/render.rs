use crate::render_pass_command::RenderPassCommand;

/// A render produced by the `Renderer::render` method.
pub struct Render<'a> {
    pub vertex_buffer: wgpu::Buffer,
    pub commands: Vec<RenderPassCommand<'a>>,
}