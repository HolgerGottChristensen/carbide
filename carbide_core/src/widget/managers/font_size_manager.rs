use crate::accessibility::{Accessibility, AccessibilityContext};
use crate::color::rgba_bytes;
use crate::draw::theme::Theme;
use crate::draw::Color;
use crate::draw::Dimension;
use crate::environment::{EnvironmentColor, EnvironmentFontSize, Keyable};
use crate::event::{AccessibilityEvent, AccessibilityEventContext, AccessibilityEventHandler, Event, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::focus::{FocusContext, Focusable};
use crate::layout::{Layout, LayoutContext};
use crate::lifecycle::{InitializationContext, Initialize, Update, UpdateContext};
use crate::render::Render;
use crate::render::RenderContext;
use crate::widget::{CommonWidget, Widget};
use carbide::ModifierWidgetImpl;

#[derive(Debug, Clone, Widget)]
#[carbide_derive(StateSync)]
pub struct FontSizeManager<C> where C: Widget {
    child: C,
    sizes: Vec<(EnvironmentFontSize, u32)>,
}

impl<C: Widget> FontSizeManager<C> {
    pub fn new(child: C) -> FontSizeManager<C> {
        FontSizeManager {
            child,
            sizes: vec![
                (EnvironmentFontSize::LargeTitle, 30),
                (EnvironmentFontSize::Title, 24),
                (EnvironmentFontSize::Title2, 20),
                (EnvironmentFontSize::Title3, 18),
                (EnvironmentFontSize::Headline, 16),
                (EnvironmentFontSize::Body, 13),
                (EnvironmentFontSize::Callout, 12),
                (EnvironmentFontSize::Subhead, 11),
                (EnvironmentFontSize::Footnote, 9),
                (EnvironmentFontSize::Caption, 8),
                (EnvironmentFontSize::Caption2, 7),
            ],
        }
    }
}

impl<C: Widget> Layout for FontSizeManager<C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let mut response = requested_size;

        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
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

        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
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

impl<C: Widget> Update for FontSizeManager<C> {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
            self.child.process_update(&mut UpdateContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget> Initialize for FontSizeManager<C> {
    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
            self.child.process_initialization(&mut InitializationContext {
                lifecycle_manager: ctx.lifecycle_manager,
                env: ctx.env,
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget> OtherEventHandler for FontSizeManager<C> {
    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
            self.child.process_other_event(event, &mut OtherEventContext {
                text: ctx.text,
                image: ctx.image,
                env: ctx.env,
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget> WindowEventHandler for FontSizeManager<C> {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
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

impl<C: Widget> AccessibilityEventHandler for FontSizeManager<C> {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
            self.child.process_accessibility_event(event, &mut AccessibilityEventContext {
                env: ctx.env,
                env_stack: inner,
            })
        })
    }
}

impl<C: Widget> KeyboardEventHandler for FontSizeManager<C> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
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

impl<C: Widget> MouseEventHandler for FontSizeManager<C> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
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

impl<C: Widget> Focusable for FontSizeManager<C> {
    fn process_focus_next(&mut self, ctx: &mut FocusContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
            self.child.process_focus_next(&mut FocusContext {
                env: ctx.env,
                env_stack: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }

    fn process_focus_previous(&mut self, ctx: &mut FocusContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
            self.child.process_focus_previous(&mut FocusContext {
                env: ctx.env,
                env_stack: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }

    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
            self.child.process_focus_request(&mut FocusContext {
                env: ctx.env,
                env_stack: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }
}

impl<C: Widget> Accessibility for FontSizeManager<C> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
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

impl<C: Widget> Render for FontSizeManager<C> {
    fn render(&mut self, ctx: &mut RenderContext) {
        EnvironmentFontSize::with_all(&self.sizes, ctx.env_stack, |inner| {
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

impl<C: Widget> CommonWidget for FontSizeManager<C> {
    ModifierWidgetImpl!(self, child: self.child);
}