use cgmath::Matrix4;
use wgpu::{BindGroup, Buffer, Surface, TextureFormat, TextureView};
use carbide_core::application::ApplicationManager;
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Dimension, Position, Scalar};
use carbide_core::draw::theme::Theme;
use carbide_core::environment::Environment;
use carbide_core::lifecycle::InitializationContext;
use carbide_core::scene::{SceneId, SceneManager};
use carbide_core::state::ReadState;
use carbide_core::widget::{CommonWidget, Widget};
use carbide_winit::raw_window_handle_05::HasRawWindowHandle;
use carbide_winit::WindowHandleKey;
use crate::application::Scenes;
use crate::msaa::Msaa;
use crate::render_context::WGPURenderContext;
use crate::RenderTarget;

pub(crate) struct InitializedWindow<T: ReadState<T=String>, C: Widget> {
    pub(crate) id: SceneId,
    pub(crate) title: T,
    pub(crate) position: Position,
    pub(crate) dimension: Dimension,
    pub(crate) child: C,
    pub(crate) surface: Surface,
    pub(crate) texture_format: TextureFormat,
    pub(crate) msaa: Msaa,
    pub(crate) msaa_texture_view: Option<TextureView>,
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

        if scene_manager.dismiss_requested() {
            println!("Here");
            self.close(env);
        } else {
            let mut ctx = InitializationContext {
                env,
            };

            for mut scene in scene_manager.scenes_to_add().drain(..) {
                scene.process_initialization(&mut ctx);
                self.scenes.push(scene);
            }

            for id in scene_manager.sub_scenes_to_dismiss() {
                self.scenes.retain(|scene| scene.id() != *id);
            }
        }
    }
}