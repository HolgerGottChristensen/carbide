use cgmath::Matrix4;
use wgpu::{BindGroup, Buffer, Surface, TextureFormat, TextureView};
use carbide_core::draw::{Dimension, Position};
use carbide_core::state::ReadState;
use carbide_core::widget::{Widget, WidgetId};
use crate::render_context::WGPURenderContext;
use crate::RenderTarget;

pub(crate) struct InitializedWindow<T: ReadState<T=String>, C: Widget> {
    pub(crate) id: WidgetId,
    pub(crate) title: T,
    pub(crate) position: Position,
    pub(crate) dimension: Dimension,
    pub(crate) child: C,
    pub(crate) surface: Surface,
    pub(crate) texture_format: TextureFormat,
    pub(crate) depth_texture_view: TextureView,
    pub(crate) texture_size_bind_group: BindGroup,
    pub(crate) targets: Vec<RenderTarget>,
    pub(crate) uniform_bind_group: BindGroup,
    pub(crate) gradient_buffer: Buffer,
    pub(crate) dashes_buffer: Buffer,
    pub(crate) gradient_dashes_bind_group: BindGroup,
    pub(crate) carbide_to_wgpu_matrix: Matrix4<f32>,
    pub(crate) vertex_buffer: (Buffer, usize),
    pub(crate) second_vertex_buffer: Buffer,
    pub(crate) render_context: WGPURenderContext,
    pub(crate) inner: carbide_winit::window::Window,
    pub(crate) accessibility_adapter: accesskit_winit::Adapter,
    pub(crate) visible: bool,
    pub(crate) close_application_on_window_close: bool,
}