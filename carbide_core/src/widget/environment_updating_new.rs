use crate::accessibility::Accessibility;
use crate::accessibility::AccessibilityContext;
use crate::draw::Dimension;
use crate::environment::Key;
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
use crate::widget::{CommonWidget, Widget};
use crate::ModifierWidgetImpl;
use std::fmt::Debug;
use std::marker::PhantomData;
use crate::misc::flags::WidgetFlag;
use carbide::state::ReadState;
use carbide::widget::{AnyWidget, Flagged, Identifiable, WidgetId};

#[derive(Debug, Widget)]
#[carbide_derive(StateSync)]
pub struct EnvUpdatingNew<C, K> where C: Widget, K: Key, K::Value: Clone {
    child: C,
    key: PhantomData<K>,
    value: K::Value,
}

impl<C: Widget, K: Key> Clone for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn clone(&self) -> Self {
        EnvUpdatingNew {
            child: self.child.clone(),
            key: Default::default(),
            value: self.value.clone(),
        }
    }
}

impl<C: Widget, K: Key> EnvUpdatingNew<C, K> where K::Value: Clone {
    pub fn new(value: K::Value, child: C) -> EnvUpdatingNew<C, K> {
        EnvUpdatingNew {
            child,
            key: PhantomData::default(),
            value
        }
    }
}

impl<C: Widget, K: Key> Layout for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let mut response = requested_size;

        ctx.env_stack.with::<K>(&self.value, |inner| {
            response = self.child.calculate_size(requested_size, &mut LayoutContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
            });
        });

        response
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let alignment = self.alignment();
        let position = self.position();
        let dimension = self.dimension();

        ctx.env_stack.with::<K>(&self.value,|inner| {
            self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
            self.child.position_children(&mut LayoutContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget, K: Key> Update for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_update(&mut UpdateContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget, K: Key> Initialize for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_initialization(&mut InitializationContext {
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget, K: Key> OtherEventHandler for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn process_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_other_event(event, &mut OtherEventContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget, K: Key> WindowEventHandler for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_window_event(event, &mut WindowEventContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
            })
        })
    }
}

impl<C: Widget, K: Key> AccessibilityEventHandler for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_accessibility_event(event, &mut AccessibilityEventContext {
                env: ctx.env,
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget, K: Key> KeyboardEventHandler for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_keyboard_event(event, &mut KeyboardEventContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
                prevent_default: ctx.prevent_default,
            })
        })
    }
}

impl<C: Widget, K: Key> MouseEventHandler for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_mouse_event(event, &mut MouseEventContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
                is_current: ctx.is_current,
                window_id: ctx.window_id,
                consumed: ctx.consumed,
            })
        })
    }
}

impl<C: Widget, K: Key> Focusable for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn process_focus_next(&mut self, ctx: &mut FocusContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_focus_next(&mut FocusContext {
                env: ctx.env,
                env_stack: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }

    fn process_focus_previous(&mut self, ctx: &mut FocusContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_focus_previous(&mut FocusContext {
                env: ctx.env,
                env_stack: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }

    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_focus_request(&mut FocusContext {
                env: ctx.env,
                env_stack: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }
}

impl<C: Widget, K: Key> Accessibility for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.process_accessibility(&mut AccessibilityContext {
                env: ctx.env,
                env_stack: inner,
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

impl<C: Widget, K: Key> Render for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.env_stack.with::<K>(&self.value, |inner| {
            self.child.render(&mut RenderContext {
                render: ctx.render,
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget, K: Key> Identifiable for EnvUpdatingNew<C, K> where K::Value: Clone {
    fn id(&self) -> WidgetId {
        self.child.id()
    }
}

impl<C: Widget, K: Key> CommonWidget for EnvUpdatingNew<C, K> where K::Value: Clone {
    ModifierWidgetImpl!(self, child: self.child);
}