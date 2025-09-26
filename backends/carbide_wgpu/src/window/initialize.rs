use std::sync::Arc;
use log::info;
use crate::application::{ActiveEventLoopKey, EVENT_LOOP_PROXY};
use crate::bind_groups::{gradient_dashes_bind_group, size_to_uniform_bind_group, uniforms_to_bind_group};
use crate::wgpu_gradient::WgpuGradient;
use crate::pipeline::{create_final_render_pipeline, create_pipelines};
use crate::render_context::WGPURenderContext;
use crate::textures::{create_depth_stencil_texture_view, create_msaa_texture_view};
use crate::wgpu_vertex::WgpuVertex;
use crate::window::initialized_window::InitializedWindow;
use crate::window::util::calculate_carbide_to_wgpu_matrix;
use crate::window::Window;
use crate::{WgpuRenderTarget};
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Dimension, DrawGradient};
use carbide_core::lifecycle::{InitializationContext, Initialize};
use carbide_core::state::ReadState;
use carbide_core::widget::{CommonWidget, Widget};
use carbide_winit::dpi::{LogicalSize, PhysicalPosition, Size};
use carbide_winit::update_scale_factor;
use carbide_winit::window::{Theme, WindowAttributes};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, SurfaceConfiguration, TextureFormat, TextureUsages};
use carbide_core::draw::color_space::ColorSpace;
use carbide_core::draw::gradient::{GradientRepeat, GradientType};
use crate::wgpu_render_target::RENDER_TARGET_FORMAT;
use crate::wgpu_context::WgpuContext;
use crate::wgpu_dashes::WgpuDashes;

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
                info!("Initializing window");

                let mut attributes = WindowAttributes::default()
                    .with_inner_size(Size::Logical(LogicalSize {
                        width: dimension.width,
                        height: dimension.height,
                    }))
                    .with_visible(false);

                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::JsCast;
                    use carbide_winit::platform::web::WindowAttributesExtWebSys;

                    let canvas = wgpu::web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("canvas")
                        .unwrap()
                        .dyn_into::<wgpu::web_sys::HtmlCanvasElement>()
                        .unwrap();

                    info!("Found and converted canvas");

                    attributes = attributes.with_canvas(Some(canvas));
                }

                let (window, accessibility_adapter) = if let Some(eventloop) = ctx.env.get::<ActiveEventLoopKey>() {
                    let window = Arc::new(eventloop.create_window(attributes).unwrap());
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

                let wgpu_context = ctx.env.get_mut::<WgpuContext>().unwrap();

                let surface = wgpu_context.instance
                    .create_surface(window.clone())
                    .unwrap();

                let surface_configuration = surface.get_default_config(
                    &wgpu_context.adapter,
                    window.inner_size().width,
                    window.inner_size().height
                ).unwrap();

                // Configure the surface with format, size and usage
                surface.configure(
                    &wgpu_context.device,
                    &surface_configuration,
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

                let gradient = WgpuGradient::convert(&gradient);
                let gradient_buffer = wgpu_context.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("carbide_gradient_buffer"),
                    contents: &*gradient.as_bytes(),
                    usage: BufferUsages::STORAGE,
                });

                let dashes = WgpuDashes {
                    dashes: [1.0; 32],
                    dash_count: 2,
                    start_cap: 0,
                    end_cap: 0,
                    total_dash_width: 2.0,
                    dash_offset: 0.0,
                };

                let dashes_buffer = wgpu_context.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("carbide_dashes_buffer"),
                    contents: &*dashes.as_bytes(),
                    usage: BufferUsages::STORAGE,
                });

                let gradient_dashed_bind_group = gradient_dashes_bind_group(
                    &wgpu_context.device,
                    &wgpu_context.gradient_buffer_bind_group_layout,
                    &gradient_buffer,
                    &dashes_buffer
                );


                let uniform_bind_group = uniforms_to_bind_group(
                    &wgpu_context.device,
                    &wgpu_context.uniform_bind_group_layout,
                    matrix,
                    0.0,
                    0.0,
                    0.0,
                    false
                );


                let texture_size_bind_group = size_to_uniform_bind_group(
                    &wgpu_context.device,
                    &wgpu_context.uniform_bind_group_layout2,
                    pixel_dimensions.width,
                    pixel_dimensions.height,
                    scale_factor,
                );

                info!("{:#?}", surface_configuration);

                if !wgpu_context.pipelines.contains_key(&RENDER_TARGET_FORMAT) {
                    let pipeline = create_pipelines(wgpu_context, RENDER_TARGET_FORMAT, msaa);
                    wgpu_context.pipelines.insert(RENDER_TARGET_FORMAT, pipeline);
                }

                let final_render_pipeline = create_final_render_pipeline(
                    &wgpu_context.device,
                    &wgpu_context.texture_bind_group_layout,
                    &wgpu_context.final_render_shader_srgb,
                    &wgpu_context.final_render_shader_linear,
                    surface_configuration.format,
                );

                let depth_texture_view = create_depth_stencil_texture_view(&wgpu_context.device, size.width, size.height, msaa);
                let msaa_texture_view = create_msaa_texture_view(&wgpu_context.device, size.width, size.height, msaa);

                let vertex_buffer = wgpu_context.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("carbide_primary_vertex_buffer"),
                    contents: &[],
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
                    surface_configuration,
                    surface_render_pipeline: final_render_pipeline,
                    msaa,
                    msaa_texture_view,
                    depth_texture_view,
                    texture_size_bind_group,
                    targets: vec![
                        WgpuRenderTarget::new(size.width, size.height, ctx.env)
                    ],
                    uniform_bind_group,
                    gradient_buffer,
                    dashes_buffer,
                    gradient_dashes_bind_group: gradient_dashed_bind_group,
                    carbide_to_wgpu_matrix: matrix,
                    vertex_buffer: (vertex_buffer, 0),
                    render_context: WGPURenderContext::new(),
                    inner: window,
                    accessibility_adapter,
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