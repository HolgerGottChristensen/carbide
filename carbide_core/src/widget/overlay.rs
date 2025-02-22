use std::marker::PhantomData;
use carbide::environment::Environment;
use carbide::event::{AccessibilityEvent, AccessibilityEventContext, WindowEvent, WindowEventContext};
use carbide::lifecycle::InitializationContext;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::EnvironmentKey;
use crate::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, OtherEvent, WindowEventHandler, AccessibilityEventHandler};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::lifecycle::{Initialize, Update, UpdateContext};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetId};

#[derive(Debug)]
pub enum OverlayAction {
    Clear,
    Insert(Box<dyn AnyWidget>)
}

#[derive(Debug)]
pub struct OverlayManager {
    overlay: Option<OverlayAction>
}

impl OverlayManager {
    pub fn clear(&mut self) {
        self.overlay = Some(OverlayAction::Clear);
    }

    pub fn insert(&mut self, overlay: impl Widget) {
        self.overlay = Some(OverlayAction::Insert(overlay.boxed()));
    }

    pub fn get<K: EnvironmentKey<Value=OverlayManager>>(env: &mut Environment, f: impl FnOnce(&mut OverlayManager)) {
        if let Some(manager) = env.get_mut::<K>() {
            f(manager)
        }
    }
}

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, MouseEvent, KeyboardEvent, OtherEvent, Initialize, Update, WindowEvent, AccessibilityEvent)]
pub struct Overlay<K, C> where C: Widget, K: EnvironmentKey<Value=OverlayManager> + Clone {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: C,

    key: PhantomData<K>,

    overlay: Option<Box<dyn AnyWidget>>,
    steal_events_when_some: bool,
}

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> Overlay<K, C> {
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

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> Overlay<K, C> {
    pub fn steal_events(mut self) -> Overlay<K, C> {
        self.steal_events_when_some = true;
        self
    }

    fn with(&mut self, env: &mut Environment, f: impl FnOnce(&mut Environment, &mut Self)) {
        let mut manager = OverlayManager {
            overlay: None,
        };

        env.with_mut::<K>(&mut manager, |env| {
            f(env, self)
        });

        if let Some(overlay) = manager.overlay {
            match overlay {
                OverlayAction::Clear => {
                    self.overlay = None;
                }
                OverlayAction::Insert(new) => {
                    self.overlay = Some(new);
                }
            }
        }
    }
}

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> Initialize for Overlay<K, C> {
    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        self.with(ctx.env, |env, inner| {
            let inner_ctx = &mut InitializationContext {
                env,
            };

            if let Some(overlay) = &mut inner.overlay {
                overlay.process_initialization(inner_ctx);
            }

            inner.child.process_initialization(inner_ctx);
        })
    }
}

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> Update for Overlay<K, C> {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.with(ctx.env, |env, inner| {
            let inner_ctx = &mut UpdateContext {
                text: ctx.text,
                image: ctx.image,
                env,
            };

            if let Some(overlay) = &mut inner.overlay {
                overlay.process_update(inner_ctx);
            }

            inner.child.process_update(inner_ctx);
        })
    }
}

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> MouseEventHandler for Overlay<K, C> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.with(ctx.env, |env, inner| {
            let inner_ctx = &mut MouseEventContext {
                text: ctx.text,
                image: ctx.image,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
                consumed: ctx.consumed,
                env,
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

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> KeyboardEventHandler for Overlay<K, C> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.with(ctx.env, |env, inner| {
            let inner_ctx = &mut KeyboardEventContext {
                text: ctx.text,
                image: ctx.image,
                env,
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

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> OtherEventHandler for Overlay<K, C> {
    fn process_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {
        self.with(ctx.env, |env, inner| {
            let inner_ctx = &mut OtherEventContext {
                text: ctx.text,
                image: ctx.image,
                env,
                is_current: ctx.is_current,
                is_consumed: ctx.is_consumed,
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

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> WindowEventHandler for Overlay<K, C> {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.with(ctx.env, |env, inner| {
            let inner_ctx = &mut WindowEventContext {
                text: ctx.text,
                image: ctx.image,
                env,
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

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> AccessibilityEventHandler for Overlay<K, C> {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        self.with(ctx.env, |env, inner| {
            let inner_ctx = &mut AccessibilityEventContext {
                env,
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

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> Layout for Overlay<K, C> {
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

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget> Render for Overlay<K, C> {
    fn render(&mut self, context: &mut RenderContext) {
        self.child.render(context);

        if let Some(overlay) = &mut self.overlay {
            overlay.render(context)
        }
    }
}

impl<K: EnvironmentKey<Value=OverlayManager> + Clone, C: Widget>CommonWidget for Overlay<K, C> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}