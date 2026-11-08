use std::sync::Arc;
use log::info;
use crate::application::{ActiveEventLoopKey, EVENT_LOOP_PROXY};
use crate::offscreen_renderer::OffscreenRenderer;
use crate::pipeline::create_final_render_pipeline;
use crate::window::initialized_window::InitializedWindow;
use crate::window::Window;
use carbide_core::cursor::MouseCursor;
use carbide_core::lifecycle::{InitializationContext, Initialize};
use carbide_core::state::ReadState;
use carbide_core::widget::Widget;
use carbide_wgpu_core::WgpuContextNew;
use carbide_winit::dpi::{LogicalSize, PhysicalPosition, Size};
use carbide_winit::update_scale_factor;
use carbide_winit::window::{Theme, WindowAttributes};
use crate::wgpu_context::WgpuContext;

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

                let attributes = WindowAttributes::default()
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
                    let adapter = accesskit_winit::Adapter::with_event_loop_proxy(eventloop, &window, EVENT_LOOP_PROXY.get().unwrap().clone());

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
                let scale_factor = window.scale_factor();

                let (wgpu_context, wgpu_context_new) = ctx.env.get_many::<(WgpuContext, WgpuContextNew)>().unwrap();

                let surface = wgpu_context_new.instance
                    .create_surface(window.clone())
                    .unwrap();

                let surface_configuration = surface.get_default_config(
                    &wgpu_context_new.adapter,
                    window.inner_size().width,
                    window.inner_size().height
                ).unwrap();

                // Configure the surface with format, size and usage
                surface.configure(
                    &wgpu_context_new.device,
                    &surface_configuration,
                );

                info!("{:#?}", surface_configuration);

                let final_render_pipeline = create_final_render_pipeline(
                    &wgpu_context_new.device,
                    &wgpu_context.texture_bind_group_layout,
                    &wgpu_context.final_render_shader_srgb,
                    &wgpu_context.final_render_shader_linear,
                    surface_configuration.format,
                );

                update_scale_factor(window.id(), window.scale_factor());

                child.initialize(ctx);

                let offscreen = OffscreenRenderer::new(size, scale_factor, msaa, ctx.env);

                let theme = match window.theme().unwrap_or(Theme::Dark) {
                    Theme::Light => carbide_core::draw::theme::Theme::Light,
                    Theme::Dark => carbide_core::draw::theme::Theme::Dark,
                };

                Window::Initialized(InitializedWindow {
                    id,
                    surface,
                    surface_configuration,
                    surface_render_pipeline: final_render_pipeline,
                    offscreen,
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