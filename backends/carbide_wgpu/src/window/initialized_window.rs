use cgmath::Matrix4;
use wgpu::{BindGroup, Buffer, Surface, TextureFormat, TextureView};
use carbide_core::application::ApplicationManager;
use carbide_core::draw::{Dimension, Position, Scalar};
use carbide_core::draw::theme::Theme;
use carbide_core::environment::EnvironmentStack;
use carbide_core::lifecycle::InitializationContext;
use carbide_core::scene::{SceneId, SceneManager};
use carbide_core::state::ReadState;
use carbide_core::widget::{CommonWidget, Widget, WidgetId};
use carbide_winit::raw_window_handle::HasWindowHandle;
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
}

impl<T: ReadState<T=String>, C: Widget> InitializedWindow<T, C> {
    pub fn close(&self, env_stack: &mut EnvironmentStack) {
        let mut closed = false;

        SceneManager::get(env_stack, |manager| {
            println!("Close sub scene");
            manager.dismiss_sub_scene(self.id);
            closed = true;
        });

        if !closed {
            ApplicationManager::get(env_stack, |manager| {
                println!("Close scene");
                manager.dismiss_scene(self.id);
                closed = true;
            });
        }

        if !closed {
            println!("tried to close window, but neither the application manager or the scene manager was available.")
        }
    }

    pub fn with_env_stack(&mut self, env_stack: &mut EnvironmentStack, f: impl FnOnce(&mut EnvironmentStack, &mut Self)) {
        let theme_for_frame = self.theme;
        let physical_dimensions = self.inner.inner_size();

        let mut scene_manager = SceneManager::new(
            self.inner.scale_factor(),
            Dimension::new(physical_dimensions.width as Scalar, physical_dimensions.height as Scalar)
        );

        let handle = self.inner.raw_window_handle();

        env_stack.with::<Theme>(&theme_for_frame, |env_stack| {
            env_stack.with_mut::<SceneManager>(&mut scene_manager, |env_stack| {
                env_stack.with::<WindowHandleKey>(&handle, |env_stack| {
                    f(env_stack, self)
                })
            })
        });

        if scene_manager.dismiss_requested() {
            println!("Here");
            self.close(env_stack);
        } else {
            let mut ctx = InitializationContext {
                env_stack,
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