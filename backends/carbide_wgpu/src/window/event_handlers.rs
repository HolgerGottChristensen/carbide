use carbide_core::draw::Dimension;
use carbide_core::event::{AccessibilityEvent, AccessibilityEventContext, AccessibilityEventHandler, Event, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use carbide_core::state::ReadState;
use carbide_core::widget::{CommonWidget, Widget};
use carbide_winit::convert_mouse_cursor;
use carbide_winit::dpi::LogicalSize;
use carbide_winit::window::Theme;
use crate::window::Window;

impl<T: ReadState<T=String>, C: Widget> MouseEventHandler for Window<T, C> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        match self {
            Window::Initialized(initialized) => {
                let id: u64 = initialized.inner.id().into();

                initialized.with_env_stack(ctx.env_stack, |env_stack, initialized| {
                    let new_ctx = &mut MouseEventContext {
                        text: ctx.text,
                        image: ctx.image,
                        env: ctx.env,
                        is_current: &(*ctx.window_id == id),
                        window_id: ctx.window_id,
                        consumed: ctx.consumed,
                        env_stack,
                    };

                    initialized.child.process_mouse_event(event, new_ctx);
                });
            }
            Window::UnInitialized { .. } => {}
            Window::Failed => {}
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> AccessibilityEventHandler for Window<T, C> {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        match self {
            Window::Initialized(initialized) => {
                initialized.with_env_stack(ctx.env_stack, |env_stack, initialized| {
                    let new_ctx = &mut AccessibilityEventContext {
                        env: ctx.env,
                        env_stack,
                    };

                    initialized.child.process_accessibility_event(event, new_ctx);
                })
            }
            Window::UnInitialized { .. } => {}
            Window::Failed => {}
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> KeyboardEventHandler for Window<T, C> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        match self {
            Window::Initialized(initialized) => {
                let id: u64 = initialized.inner.id().into();

                initialized.with_env_stack(ctx.env_stack, |env_stack, initialized| {
                    let new_ctx = &mut KeyboardEventContext {
                        text: ctx.text,
                        image: ctx.image,
                        env: ctx.env,
                        env_stack,
                        is_current: &(*ctx.window_id == id),
                        window_id: ctx.window_id,
                        prevent_default: ctx.prevent_default,
                    };

                    initialized.child.process_keyboard_event(event, new_ctx);
                })
            }
            Window::UnInitialized { .. } => {}
            Window::Failed => {}
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> OtherEventHandler for Window<T, C> {
    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        match self {
            Window::Initialized(initialized) => {
                initialized.with_env_stack(ctx.env_stack, |env_stack, initialized| {
                    initialized.child.process_other_event(event, &mut OtherEventContext {
                        text: ctx.text,
                        image: ctx.image,
                        env: ctx.env,
                        env_stack,
                    });
                })
            }
            Window::UnInitialized { .. } => {}
            Window::Failed => {}
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> WindowEventHandler for Window<T, C> {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        match self {
            Window::Initialized(initialized) => {
                let window_id = ctx.window_id;

                let id: u64 = initialized.inner.id().into();

                let is_current = *window_id == id;

                if is_current {
                    match event {
                        WindowEvent::Resize(size) => {
                            initialized.resize(LogicalSize::new(size.width, size.height).to_physical(initialized.inner.scale_factor()));
                            initialized.inner.request_redraw();
                        }
                        WindowEvent::Focus => {
                            /*#[cfg(target_os = "macos")]
                            {
                                use carbide_macos::NSMenu;
                                use carbide_macos::NSMenuItem;

                                // The outer menu is not visible, but only a container for
                                // other menus.

                                if let Some(menu) = &self.window_menu {
                                    let mut outer_menu = NSMenu::new("");

                                    for m in menu {
                                        let item = NSMenuItem::new(m.name(), None)
                                            .set_submenu(NSMenu::from(m, ctx.env));

                                        outer_menu = outer_menu.add_item(item);
                                    }

                                    outer_menu.set_as_main_menu();
                                } else {
                                    // Todo: Set default application menu
                                }
                            }*/
                        }
                        WindowEvent::CloseRequested => {
                            initialized.visible = false;
                            if initialized.close_application_on_window_close {
                                ctx.env.close_application();
                            } else {
                                initialized.inner.set_visible(false);
                            }
                        }
                        WindowEvent::ThemeChanged => {
                            let theme = match initialized.inner.theme().unwrap_or(Theme::Dark) {
                                Theme::Light => carbide_core::draw::theme::Theme::Light,
                                Theme::Dark => carbide_core::draw::theme::Theme::Dark,
                            };
                            println!("Theme: {:?}", theme);

                            initialized.theme = theme;
                        }
                        _ => ()
                    }
                }

                initialized.with_env_stack(ctx.env_stack, |env_stack, initialized| {
                    initialized.child.process_window_event(event, &mut WindowEventContext {
                        text: ctx.text,
                        image: ctx.image,
                        env: ctx.env,
                        env_stack,
                        is_current: &is_current,
                        window_id: ctx.window_id,
                    });
                })
            }
            Window::UnInitialized { .. } => {}
            Window::Failed => {}
        }
    }
}