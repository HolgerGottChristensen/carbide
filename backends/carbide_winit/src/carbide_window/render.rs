use crate::carbide_window::initialized_window::InitializedWindow;
use crate::carbide_window::Window;
use crate::carbide_window::WindowState;
use carbide_core::application::ApplicationManager;
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Alignment, Dimension, Position, Rect, Scalar};
use carbide_core::layout::LayoutContext;
use carbide_core::lifecycle::UpdateContext;
use carbide_core::render::{Render, RenderContext};
use carbide_windowing::{PhysicalSize, WindowRenderError};
use carbide_core::state::ReadState;
use carbide_core::widget::Widget;
use crate::convert_mouse_cursor;
use log::info;
use carbide_core::environment::Environment;

impl<T: ReadState<T=String>, C: Widget> Render for Window<T, C> {
    fn render(&mut self, ctx: &mut RenderContext) {
        match &mut self.state {
            WindowState::Initialized(initialized) => initialized.render(ctx),
            WindowState::UnInitialized { .. } => {}
            WindowState::Failed => {}
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> InitializedWindow<T, C> {
    fn render(&mut self, ctx: &mut RenderContext) {
        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();
        let logical_dimensions = physical_dimensions.to_logical(scale_factor);
        let dimensions = Dimension::new(logical_dimensions.width, logical_dimensions.height);

        info!("Render window, {}, {}", scale_factor, dimensions);

        let mut result = Ok(());

        self.with_env(ctx.env, |env, initialized| {
            for scene in &mut initialized.scenes {
                scene.render(&mut RenderContext {
                    render: ctx.render,
                    image: ctx.image,
                    env,
                });
            }

            // Update children
            initialized.child.process_update(&mut UpdateContext {
                image: ctx.image,
                env,
            });

            // Calculate size
            initialized.child.calculate_size(dimensions, &mut LayoutContext {
                image: ctx.image,
                env,
            });

            // Position children
            let alignment = Alignment::Center;
            initialized.child.set_position(alignment.position(Position::new(0.0, 0.0), dimensions, initialized.child.dimension()));

            // When we have a window, everything outside that window can be clipped when rendered.
            let bounding_box = Rect::new(
                Position::new(0.0, 0.0),
                dimensions
            );

            initialized.child.position_children(
                bounding_box,
                &mut LayoutContext {
                    image: ctx.image,
                    env,
                }
            );

            if initialized.visible {
                {
                    initialized.title.sync(env);

                    let current = &*initialized.title.value();
                    if &initialized.inner.title() != current {
                        initialized.inner.set_title(current);
                    }
                }

                initialized.inner.pre_present_notify();

                result = initialized.renderer.render(&mut initialized.child, dimensions, scale_factor, ctx.image, env);
            }
        });

        if self.visible {
            let cursor = convert_mouse_cursor(self.mouse_cursor);
            self.inner.set_cursor(cursor);
            self.mouse_cursor = MouseCursor::Default;
        }

        match result {
            Ok(_) => {}
            // Recreate the swap_chain if lost
            Err(WindowRenderError::Lost) => {
                println!("Swap chain lost");
                self.resize(self.inner.inner_size(), ctx.env);
                self.inner.request_redraw();
            }
            // The system is out of memory, we should probably quit
            Err(WindowRenderError::OutOfMemory) => {
                println!("Swap chain out of memory");
                ApplicationManager::get(ctx.env, |manager| {
                    manager.close();
                });
            }
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(WindowRenderError::Retry) => {
                // We request a redraw the next frame
                self.inner.request_redraw();
            }
        }
    }

    pub fn resize(&mut self, new_size: crate::dpi::PhysicalSize<u32>, env: &mut Environment) {
        let scale_factor: Scalar = self.inner.scale_factor();

        self.renderer.resize(PhysicalSize::new(new_size.width, new_size.height), scale_factor, env);
    }
}
