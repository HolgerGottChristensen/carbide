use std::marker::PhantomData;
use carbide::environment::EnvironmentStack;
use carbide::event::{AccessibilityEvent, AccessibilityEventContext, WindowEvent, WindowEventContext};
use carbide::lifecycle::InitializationContext;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Key;
use crate::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, Event, WindowEventHandler, AccessibilityEventHandler};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::lifecycle::{Initialize, Update, UpdateContext};
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug)]
pub enum OverlayAction {
    None,
    Clear,
    Insert(Box<dyn AnyWidget>)
}

#[derive(Debug)]
pub struct OverlayManager {
    overlay: OverlayAction
}

impl OverlayManager {
    pub fn clear(&mut self) {
        self.overlay = OverlayAction::Clear;
    }

    pub fn insert(&mut self, overlay: impl Widget) {
        self.overlay = OverlayAction::Insert(overlay.boxed());
    }

    pub fn get<K: Key<Value=OverlayManager>>(env_stack: &mut EnvironmentStack, f: impl FnOnce(&mut OverlayManager)) {
        if let Some(manager) = env_stack.get_mut::<K>() {
            f(manager)
        }
    }
}

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, MouseEvent, KeyboardEvent, OtherEvent, Initialize, Update, WindowEvent, AccessibilityEvent)]
pub struct Overlay<K, C> where C: Widget, K: Key<Value=OverlayManager> + Clone {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: C,

    key: PhantomData<K>,

    overlay: Option<Box<dyn AnyWidget>>,
    steal_events_when_some: bool,
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> Overlay<K, C> {
    pub fn new(child: C) -> Overlay<K, C> {
        Overlay {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child,
            key: Default::default(),
            overlay: None,
            steal_events_when_some: false,
        }
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> Overlay<K, C> {
    pub fn steal_events(mut self) -> Overlay<K, C> {
        self.steal_events_when_some = true;
        self
    }

    fn with(&mut self, env_stack: &mut EnvironmentStack, f: impl FnOnce(&mut EnvironmentStack, &mut Self)) {
        let mut manager = OverlayManager {
            overlay: OverlayAction::None,
        };

        env_stack.with_mut::<K>(&mut manager, |env_stack| {
            f(env_stack, self)
        });

        match manager.overlay {
            OverlayAction::None => {}
            OverlayAction::Clear => {
                self.overlay = None;
            }
            OverlayAction::Insert(new) => {
                self.overlay = Some(new);
            }
        }
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> Initialize for Overlay<K, C> {
    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        self.with(ctx.env_stack, |env_stack, inner| {
            let inner_ctx = &mut InitializationContext {
                env: ctx.env,
                env_stack,
                lifecycle_manager: ctx.lifecycle_manager,
            };

            if let Some(overlay) = &mut inner.overlay {
                overlay.process_initialization(inner_ctx);
            }

            inner.child.process_initialization(inner_ctx);
        })
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> Update for Overlay<K, C> {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.with(ctx.env_stack, |env_stack, inner| {
            let inner_ctx = &mut UpdateContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack,
            };

            if let Some(overlay) = &mut inner.overlay {
                overlay.process_update(inner_ctx);
            }

            inner.child.process_update(inner_ctx);
        })
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> MouseEventHandler for Overlay<K, C> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.with(ctx.env_stack, |env_stack, inner| {
            let inner_ctx = &mut MouseEventContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
                consumed: ctx.consumed,
                env_stack,
            };

            if let Some(overlay) = &mut inner.overlay {
                overlay.process_mouse_event(event, inner_ctx);
                if inner.steal_events_when_some {
                    return;
                }
            }

            inner.child.process_mouse_event(event, inner_ctx);
        })

    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> KeyboardEventHandler for Overlay<K, C> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.with(ctx.env_stack, |env_stack, inner| {
            let inner_ctx = &mut KeyboardEventContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
                prevent_default: ctx.prevent_default,
            };

            if let Some(overlay) = &mut inner.overlay {
                overlay.process_keyboard_event(event, inner_ctx);
                if inner.steal_events_when_some {
                    return;
                }
            }

            inner.child.process_keyboard_event(event, inner_ctx);
        })
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> OtherEventHandler for Overlay<K, C> {
    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        self.with(ctx.env_stack, |env_stack, inner| {
            let inner_ctx = &mut OtherEventContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack,
            };

            if let Some(overlay) = &mut inner.overlay {
                overlay.process_other_event(event, inner_ctx);
                if inner.steal_events_when_some {
                    return;
                }
            }

            inner.child.process_other_event(event, inner_ctx);
        })
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> WindowEventHandler for Overlay<K, C> {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.with(ctx.env_stack, |env_stack, inner| {
            let inner_ctx = &mut WindowEventContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
            };

            if let Some(overlay) = &mut inner.overlay {
                overlay.process_window_event(event, inner_ctx);
                if inner.steal_events_when_some {
                    return;
                }
            }

            inner.child.process_window_event(event, inner_ctx);
        })
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> AccessibilityEventHandler for Overlay<K, C> {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        self.with(ctx.env_stack, |env_stack, inner| {
            let inner_ctx = &mut AccessibilityEventContext {
                env: ctx.env,
                env_stack,
            };

            if let Some(overlay) = &mut inner.overlay {
                overlay.process_accessibility_event(event, inner_ctx);
                if inner.steal_events_when_some {
                    return;
                }
            }

            inner.child.process_accessibility_event(event, inner_ctx);
        })
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> Layout for Overlay<K, C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        if let Some(overlay) = &mut self.overlay {
            overlay.calculate_size(requested_size, ctx);
        }

        self.dimension = self.child.calculate_size(requested_size, ctx);
        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let alignment = self.alignment();
        let position = self.position();
        let dimension = self.dimension();

        self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
        self.child.position_children(ctx);

        if let Some(overlay) = &mut self.overlay {
            overlay.set_position(alignment.position(position, dimension, overlay.dimension()));
            overlay.position_children(ctx);
        }
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget> Render for Overlay<K, C> {
    fn render(&mut self, context: &mut RenderContext) {
        self.child.render(context);

        if let Some(overlay) = &mut self.overlay {
            overlay.render(context)
        }
    }
}

impl<K: Key<Value=OverlayManager> + Clone, C: Widget>CommonWidget for Overlay<K, C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}