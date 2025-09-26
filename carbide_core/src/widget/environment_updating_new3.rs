use crate::accessibility::Accessibility;
use crate::accessibility::AccessibilityContext;
use crate::draw::Dimension;
use crate::environment::EnvironmentKeyable;
use crate::event::{AccessibilityEvent, AccessibilityEventContext, OtherEvent, KeyboardEvent, KeyboardEventContext, MouseEvent, MouseEventContext, OtherEventContext, WindowEvent, WindowEventContext};
use crate::event::{AccessibilityEventHandler, KeyboardEventHandler, MouseEventHandler, OtherEventHandler, WindowEventHandler};
use crate::focus::FocusContext;
use crate::focus::Focusable;
use crate::layout::Layout;
use crate::layout::LayoutContext;
use crate::lifecycle::{InitializationContext, UpdateContext};
use crate::lifecycle::{Initialize, Update};
use crate::render::Render;
use crate::render::RenderContext;
use crate::widget::{CommonWidget, Widget, WidgetId};
use crate::ModifierWidgetImpl;
use std::fmt::Debug;
use crate::identifiable::Identifiable;
use crate::state::ReadState;

#[derive(Debug, Clone, Widget)]
#[carbide_derive(StateSync)]
pub struct EnvUpdatingNew3<C, K, V> where C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>, K::Output: Clone {
    child: C,
    key: K,
    value: V,
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    pub fn new(key: K, value: V, child: C) -> EnvUpdatingNew3<C, K, V> {
        EnvUpdatingNew3 {
            child,
            key,
            value
        }
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> Layout for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.value.sync(ctx.env);

        let mut response = requested_size;

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            response = self.child.calculate_size(requested_size, &mut LayoutContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
            });
        });

        response
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        self.value.sync(ctx.env);

        let alignment = self.alignment();
        let position = self.position();
        let dimension = self.dimension();

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
            self.child.position_children(&mut LayoutContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> Update for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_update(&mut UpdateContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> Initialize for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_initialization(&mut InitializationContext {
                env: inner,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> OtherEventHandler for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn process_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_other_event(event, &mut OtherEventContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
                is_current: ctx.is_current,
                is_consumed: ctx.is_consumed,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> WindowEventHandler for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_window_event(event, &mut WindowEventContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> AccessibilityEventHandler for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_accessibility_event(event, &mut AccessibilityEventContext {
                env: inner,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> KeyboardEventHandler for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_keyboard_event(event, &mut KeyboardEventContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
                prevent_default: ctx.prevent_default,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> MouseEventHandler for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_mouse_event(event, &mut MouseEventContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
                consumed: ctx.consumed,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> Focusable for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn process_focus_next(&mut self, ctx: &mut FocusContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_focus_next(&mut FocusContext {
                env: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }

    fn process_focus_previous(&mut self, ctx: &mut FocusContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_focus_previous(&mut FocusContext {
                env: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }

    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_focus_request(&mut FocusContext {
                env: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> Accessibility for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.process_accessibility(&mut AccessibilityContext {
                env: inner,
                nodes: ctx.nodes,
                parent_id: ctx.parent_id,
                children: ctx.children,
                hidden: ctx.hidden,
                inherited_label: ctx.inherited_label,
                inherited_hint: ctx.inherited_hint,
                inherited_value: ctx.inherited_value,
                inherited_enabled: ctx.inherited_enabled,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> Render for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn render(&mut self, ctx: &mut RenderContext) {
        self.value.sync(ctx.env);

        self.key.with(&*self.value.value(), ctx.env, |inner| {
            self.child.render(&mut RenderContext {
                render: ctx.render,
                text: ctx.text,
                image: ctx.image,
                env: inner,
            })
        })
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> Identifiable<WidgetId> for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    fn id(&self) -> WidgetId {
        self.child.id()
    }
}

impl<C: Widget, K: EnvironmentKeyable + Clone, V: ReadState<T=K::Output>> CommonWidget for EnvUpdatingNew3<C, K, V> where K::Output: Clone {
    ModifierWidgetImpl!(self, child: self.child);
}