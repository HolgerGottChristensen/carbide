use crate::application::Scenes;
use crate::offscreen_renderer::OffscreenRenderer;
use crate::wgpu_context::WgpuContext;
use carbide_core::application::ApplicationManager;
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::theme::Theme;
use carbide_core::draw::{Dimension, NOOPImageContext, Position, Scalar};
use carbide_core::environment::Environment;
use carbide_core::event::{WindowEvent, WindowEventContext, WindowEventHandler};
use carbide_core::lifecycle::InitializationContext;
use carbide_core::scene::{SceneId, SceneManager};
use carbide_core::state::ReadState;
use carbide_core::text::NOOPTextContext;
use carbide_core::widget::Widget;
use carbide_winit::raw_window_handle_05::HasRawWindowHandle;
use carbide_winit::WindowHandleKey;
use std::sync::Arc;
use wgpu::{RenderPipeline, Surface, SurfaceConfiguration};
use carbide_wgpu_core::WgpuContextNew;

pub(crate) struct InitializedWindow<T: ReadState<T=String>, C: Widget> {
    pub(crate) id: SceneId,
    pub(crate) title: T,
    pub(crate) position: Position,
    pub(crate) dimension: Dimension,
    pub(crate) child: C,

    pub(crate) surface: Surface<'static>,
    pub(crate) surface_configuration: SurfaceConfiguration,
    pub(crate) surface_render_pipeline: RenderPipeline,

    pub(crate) offscreen: OffscreenRenderer,

    pub(crate) inner: Arc<carbide_winit::window::Window>,
    pub(crate) accessibility_adapter: accesskit_winit::Adapter,
    pub(crate) visible: bool,
    pub(crate) theme: Theme,
    pub(crate) scenes: Scenes,
    pub(crate) mouse_cursor: MouseCursor,
}

impl<T: ReadState<T=String>, C: Widget> InitializedWindow<T, C> {
    pub fn close(&self, env: &mut Environment) {
        let mut closed = false;

        SceneManager::get(env, |manager| {
            println!("Close sub scene");
            manager.dismiss_sub_scene(self.id);
            closed = true;
        });

        if !closed {
            ApplicationManager::get(env, |manager| {
                println!("Close scene");
                manager.dismiss_scene(self.id);
                closed = true;
            });
        }

        if !closed {
            println!("tried to close window, but neither the application manager or the scene manager was available.")
        }
    }

    pub fn with_env(&mut self, env: &mut Environment, f: impl FnOnce(&mut Environment, &mut Self)) {
        let theme_for_frame = self.theme;
        let physical_dimensions = self.inner.inner_size();

        let mut scene_manager = SceneManager::new(
            self.inner.scale_factor(),
            Dimension::new(physical_dimensions.width as Scalar, physical_dimensions.height as Scalar)
        );

        let handle = self.inner.raw_window_handle();

        let mut cursor = self.mouse_cursor;

        env.with::<Theme>(&theme_for_frame, |env| {
            env.with_mut::<SceneManager>(&mut scene_manager, |env| {
                env.with::<WindowHandleKey>(&handle, |env| {
                    env.with_mut::<MouseCursor>(&mut cursor, |env| {
                        f(env, self)
                    })
                })
            })
        });

        self.mouse_cursor = cursor;

        let mut screenshot_requests = std::mem::take(scene_manager.screenshot_requests());
        if !screenshot_requests.is_empty() {
            let wgpu_context = env.get_mut::<WgpuContextNew>()
                .expect("WgpuContext is in scope at every call site that reaches InitializedWindow::with_env");
            let image = crate::screenshot::capture_texture_to_image(
                &wgpu_context.device,
                &wgpu_context.queue,
                &self.offscreen.targets[0].texture,
            );
            for sender in screenshot_requests.drain(..) {
                let _ = sender.send(image.clone());
            }
        }

        if scene_manager.dismiss_requested() {
            println!("Here");
            self.close(env);
        } else {
            for mut scene in scene_manager.scenes_to_add().drain(..) {
                scene.process_initialization(&mut InitializationContext {
                    env,
                });
                scene.process_window_event(&WindowEvent::Initialize, &mut WindowEventContext {
                    text: &mut NOOPTextContext,
                    image: &mut NOOPImageContext,
                    env,
                    is_current: &false, // todo
                    window_id: &0, // todo
                });
                self.scenes.push(scene);
            }

            for id in scene_manager.sub_scenes_to_dismiss() {
                self.scenes.retain(|scene| scene.id() != *id);
            }
        }
    }
}