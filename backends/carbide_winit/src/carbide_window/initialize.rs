use std::sync::Arc;
use log::info;
use crate::app::{ActiveEventLoopKey, EVENT_LOOP_PROXY};
use crate::carbide_window::initialized_window::InitializedWindow;
use crate::carbide_window::Window;
use crate::carbide_window::WindowState;
use carbide_core::cursor::MouseCursor;
use carbide_core::lifecycle::{InitializationContext, Initialize};
use carbide_windowing::{PhysicalSize, WindowRendererFactory, WindowRenderer};
use carbide_core::state::ReadState;
use carbide_core::widget::Widget;
use crate::dpi::{LogicalSize, PhysicalPosition, Size};
use crate::update_scale_factor;
use crate::window::{Theme, WindowAttributes};

pub const ZOOM: f32 = 1.0;

impl<T: ReadState<T=String>, C: Widget> Initialize for Window<T, C> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        self.state = match std::mem::replace(&mut self.state, WindowState::Failed) {
            WindowState::UnInitialized {
                id,
                title,
                position,
                dimension,
                mut child,
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
                    use crate::platform::web::WindowAttributesExtWebSys;

                    let canvas = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("canvas")
                        .unwrap()
                        .dyn_into::<web_sys::HtmlCanvasElement>()
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

                update_scale_factor(window.id(), window.scale_factor());

                child.initialize(ctx);

                let factory = *ctx.env.get::<WindowRendererFactory>()
                    .expect("No renderer is registered. Enable carbide_winit's `wgpu` feature, or link another renderer crate");
                let renderer = WindowRenderer::new(factory, window.clone(), PhysicalSize::new(size.width, size.height), scale_factor, ctx.env);

                let theme = match window.theme().unwrap_or(Theme::Dark) {
                    Theme::Light => carbide_core::draw::theme::Theme::Light,
                    Theme::Dark => carbide_core::draw::theme::Theme::Dark,
                };

                WindowState::Initialized(InitializedWindow {
                    id,
                    renderer,
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