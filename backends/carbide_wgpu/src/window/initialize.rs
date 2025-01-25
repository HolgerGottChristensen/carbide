use wgpu::{BufferUsages, SurfaceConfiguration, TextureFormat, TextureUsages};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{ColorSpace, Dimension, DrawGradient};
use carbide_core::lifecycle::{InitializationContext, Initialize};
use carbide_core::state::ReadState;
use carbide_core::widget::{CommonWidget, GradientRepeat, GradientType, Widget};
use carbide_winit::dpi::{LogicalSize, PhysicalPosition, Size};
use carbide_winit::update_scale_factor;
use carbide_winit::window::{Theme, WindowAttributes};
use crate::application::{ActiveEventLoopKey, ADAPTER, EVENT_LOOP_PROXY, INSTANCE};
use crate::bind_group_layouts::{GRADIENT_DASHES_BIND_GROUP_LAYOUT, UNIFORM_BIND_GROUP_LAYOUT, UNIFORM_BIND_GROUP_LAYOUT2};
use crate::bind_groups::{gradient_dashes_bind_group, size_to_uniform_bind_group, uniforms_to_bind_group};
use crate::{RenderTarget, DEVICE};
use crate::gradient::{Dashes, Gradient};
use crate::pipeline::create_pipelines;
use crate::render_context::WGPURenderContext;
use crate::textures::{create_depth_stencil_texture_view, create_msaa_texture_view};
use crate::vertex::Vertex;
use crate::globals::PIPELINES;
use crate::window::initialized_window::InitializedWindow;
use crate::window::util::calculate_carbide_to_wgpu_matrix;
use crate::window::Window;

pub const ZOOM: f32 = 1.0;

impl<T: ReadState<T=String>, C: Widget> Initialize for Window<T, C> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        *self = match std::mem::replace(self, Window::Failed) {
            Window::UnInitialized {
                id,
                title,
                position,
                dimension,
                mut child,
                msaa,
            } => {
                let attributes = WindowAttributes::default()
                    .with_inner_size(Size::Logical(LogicalSize {
                        width: dimension.width,
                        height: dimension.height,
                    }))
                    .with_visible(false);



                let (window, adapter) = if let Some(eventloop) = ctx.env_stack.get::<ActiveEventLoopKey>() {
                    let window = eventloop.create_window(attributes).unwrap();
                    let adapter = accesskit_winit::Adapter::with_event_loop_proxy(&window, EVENT_LOOP_PROXY.get().unwrap().clone());

                    (window, adapter)
                } else {
                    panic!("Could not downcast the lifecycle manager to `ActiveEventLoop`");
                };

                window.set_ime_allowed(true);

                // Position the window in the middle of the screen.
                if let Some(monitor) = window.current_monitor() {
                    let size = monitor.size();

                    let outer_window_size = window.outer_size();

                    let position = PhysicalPosition::new(
                        size.width / 2 - outer_window_size.width / 2,
                        size.height / 2 - outer_window_size.height / 2,
                    );

                    window.set_outer_position(position);
                }

                println!("DPI: {}", window.scale_factor());

                window.set_visible(true);

                let size = window.inner_size();

                let pixel_dimensions = Dimension::new(
                    window.inner_size().width as f64,
                    window.inner_size().height as f64,
                );
                let scale_factor = window.scale_factor();


                let surface = unsafe { INSTANCE.create_surface(&window) }.unwrap();

                // Configure the surface with format, size and usage
                let surface_caps = surface.get_capabilities(&*ADAPTER);

                surface.configure(
                    &DEVICE,
                    &SurfaceConfiguration {
                        usage: TextureUsages::RENDER_ATTACHMENT,
                        format: TextureFormat::Bgra8UnormSrgb,
                        width: window.inner_size().width,
                        height: window.inner_size().height,
                        present_mode: surface_caps.present_modes[0],
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        view_formats: vec![],
                    },
                );

                let matrix = calculate_carbide_to_wgpu_matrix(pixel_dimensions, scale_factor);

                let gradient = DrawGradient {
                    colors: vec![],
                    ratios: vec![],
                    gradient_type: GradientType::Linear,
                    gradient_repeat: GradientRepeat::Clamp,
                    start: Default::default(),
                    end: Default::default(),
                    color_space: ColorSpace::Linear,
                };

                let gradient = Gradient::convert(&gradient);
                let gradient_buffer = DEVICE.create_buffer_init(&BufferInitDescriptor {
                    label: Some("carbide_gradient_buffer"),
                    contents: &*gradient.as_bytes(),
                    usage: BufferUsages::STORAGE,
                });

                let dashes = Dashes {
                    dashes: [1.0; 32],
                    dash_count: 2,
                    start_cap: 0,
                    end_cap: 0,
                    total_dash_width: 2.0,
                    dash_offset: 0.0,
                };

                let dashes_buffer = DEVICE.create_buffer_init(&BufferInitDescriptor {
                    label: Some("carbide_dashes_buffer"),
                    contents: &*dashes.as_bytes(),
                    usage: BufferUsages::STORAGE,
                });

                let gradient_dashed_bind_group = gradient_dashes_bind_group(
                    &DEVICE,
                    &GRADIENT_DASHES_BIND_GROUP_LAYOUT,
                    &gradient_buffer,
                    &dashes_buffer
                );


                let uniform_bind_group = uniforms_to_bind_group(
                    &DEVICE,
                    &UNIFORM_BIND_GROUP_LAYOUT,
                    matrix,
                    0.0,
                    0.0,
                    0.0,
                    false
                );


                let texture_size_bind_group = size_to_uniform_bind_group(
                    &DEVICE,
                    &UNIFORM_BIND_GROUP_LAYOUT2,
                    pixel_dimensions.width,
                    pixel_dimensions.height,
                    scale_factor,
                );

                let preferred_format = surface_caps.formats.iter()
                    .copied()
                    .filter(|f| f.is_srgb())
                    .next()
                    .unwrap_or(surface_caps.formats[0]);

                if !PIPELINES.contains_key(&preferred_format) {
                    PIPELINES.insert(preferred_format, create_pipelines(&DEVICE, preferred_format, msaa));
                }

                let depth_texture_view = create_depth_stencil_texture_view(&DEVICE, size.width, size.height, msaa);
                let msaa_texture_view = create_msaa_texture_view(&DEVICE, size.width, size.height, msaa);

                let vertex_buffer = DEVICE.create_buffer_init(&BufferInitDescriptor {
                    label: Some("carbide_primary_vertex_buffer"),
                    contents: &[],
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                });

                let secondary_vertex_buffer = DEVICE.create_buffer_init(&BufferInitDescriptor {
                    label: Some("carbide_secondary_vertex_buffer"),
                    contents: bytemuck::cast_slice(&Vertex::rect(size, scale_factor, ZOOM)),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                });

                update_scale_factor(window.id(), window.scale_factor());

                child.initialize(ctx);

                let theme = match window.theme().unwrap_or(Theme::Dark) {
                    Theme::Light => carbide_core::draw::theme::Theme::Light,
                    Theme::Dark => carbide_core::draw::theme::Theme::Dark,
                };


                Window::Initialized(InitializedWindow {
                    id,
                    surface,
                    texture_format: preferred_format,
                    msaa,
                    msaa_texture_view,
                    depth_texture_view,
                    texture_size_bind_group,
                    targets: vec![
                        RenderTarget::new(size.width, size.height)
                    ],
                    uniform_bind_group,
                    gradient_buffer,
                    dashes_buffer,
                    gradient_dashes_bind_group: gradient_dashed_bind_group,
                    carbide_to_wgpu_matrix: matrix,
                    vertex_buffer: (vertex_buffer, 0),
                    second_vertex_buffer: secondary_vertex_buffer,
                    render_context: WGPURenderContext::new(),
                    inner: window,
                    accessibility_adapter: adapter,
                    visible: true,
                    title,
                    position,
                    dimension,
                    child,
                    theme,
                    scenes: Default::default(),
                    mouse_cursor: MouseCursor::Default
                })
            }
            x => x,
        };
    }
}