use crate::accessibility::{Accessibility, AccessibilityContext};
use crate::color::rgba_bytes;
use crate::draw::theme::Theme;
use crate::draw::Color;
use crate::draw::Dimension;
use crate::environment::{EnvironmentColor, EnvironmentKeyable};
use crate::event::{AccessibilityEvent, AccessibilityEventContext, AccessibilityEventHandler, OtherEvent, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use crate::focus::{FocusContext, Focusable};
use crate::identifiable::Identifiable;
use crate::layout::{Layout, LayoutContext};
use crate::lifecycle::{InitializationContext, Initialize, Update, UpdateContext};
use crate::render::Render;
use crate::render::RenderContext;
use crate::widget::{CommonWidget, Widget};
use crate::ModifierWidgetImpl;
use crate::widget::{WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_derive(StateSync)]
pub struct ThemeManager<C> where C: Widget {
    child: C,
    light: Vec<(EnvironmentColor, Color)>,
    dark: Vec<(EnvironmentColor, Color)>
}

impl<C: Widget> ThemeManager<C> {
    pub fn new(child: C) -> ThemeManager<C> {
        ThemeManager {
            child,
            light: vec![
                (EnvironmentColor::Blue, rgba_bytes(0, 122, 255, 1.0)),
                (EnvironmentColor::Green, rgba_bytes(52, 199, 89, 1.0)),
                (EnvironmentColor::Indigo, rgba_bytes(88, 86, 214, 1.0)),
                (EnvironmentColor::Orange, rgba_bytes(255, 149, 0, 1.0)),
                (EnvironmentColor::Pink, rgba_bytes(255, 45, 85, 1.0)),
                (EnvironmentColor::Purple, rgba_bytes(175, 82, 222, 1.0)),
                (EnvironmentColor::Red, rgba_bytes(255, 59, 48, 1.0)),
                (EnvironmentColor::Teal, rgba_bytes(90, 200, 250, 1.0)),
                (EnvironmentColor::Yellow, rgba_bytes(255, 204, 0, 1.0)),
                (EnvironmentColor::Gray, rgba_bytes(142, 142, 147, 1.0)),
                (EnvironmentColor::Gray2, rgba_bytes(174, 174, 178, 1.0)),
                (EnvironmentColor::Gray3, rgba_bytes(199, 199, 204, 1.0)),
                (EnvironmentColor::Gray4, rgba_bytes(209, 209, 214, 1.0)),
                (EnvironmentColor::Gray5, rgba_bytes(229, 229, 234, 1.0)),
                (EnvironmentColor::Gray6, rgba_bytes(242, 242, 247, 1.0)),
                (EnvironmentColor::SystemBackground, rgba_bytes(255, 255, 255, 1.0)),
                (EnvironmentColor::SecondarySystemBackground, rgba_bytes(242, 242, 247, 1.0)),
                (EnvironmentColor::TertiarySystemBackground, rgba_bytes(255, 255, 255, 1.0)),
                (EnvironmentColor::Label, rgba_bytes(10, 10, 10, 1.0)),
                (EnvironmentColor::SecondaryLabel, rgba_bytes(138, 138, 142, 1.0)),
                (EnvironmentColor::TertiaryLabel, rgba_bytes(196, 196, 198, 1.0)),
                (EnvironmentColor::QuaternaryLabel, rgba_bytes(220, 220, 221, 1.0)),
                (EnvironmentColor::PlaceholderText, rgba_bytes(196, 196, 198, 1.0)),
                (EnvironmentColor::Link, rgba_bytes(0, 122, 255, 1.0)),
                (EnvironmentColor::SystemFill, rgba_bytes(228, 228, 230, 1.0)),
                (EnvironmentColor::SecondarySystemFill, rgba_bytes(233, 233, 235, 1.0)),
                (EnvironmentColor::TertiarySystemFill, rgba_bytes(239, 239, 240, 1.0)),
                (EnvironmentColor::QuaternarySystemFill, rgba_bytes(244, 244, 245, 1.0)),
                (EnvironmentColor::OpaqueSeparator, rgba_bytes(220, 220, 222, 1.0)),
                (EnvironmentColor::Separator, rgba_bytes(0, 0, 0, 0.137)),
                (EnvironmentColor::Accent, rgba_bytes(0, 122, 255, 1.0)),
                (EnvironmentColor::LightText, rgba_bytes(0, 0, 0, 1.0)),
                (EnvironmentColor::DarkText, rgba_bytes(255, 255, 255, 1.0)),
                // Material colors
                (EnvironmentColor::UltraThick, rgba_bytes(255, 255, 255, 0.8)),
                (EnvironmentColor::Thick, rgba_bytes(255, 255, 255, 0.6)),
                (EnvironmentColor::Regular, rgba_bytes(255, 255, 255, 0.4)),
                (EnvironmentColor::Thin, rgba_bytes(255, 255, 255, 0.25)),
                (EnvironmentColor::UltraThin, rgba_bytes(255, 255, 255, 0.15)),
                // Material colors light
                (EnvironmentColor::UltraThickLight, rgba_bytes(255, 255, 255, 0.8)),
                (EnvironmentColor::ThickLight, rgba_bytes(255, 255, 255, 0.6)),
                (EnvironmentColor::RegularLight, rgba_bytes(255, 255, 255, 0.4)),
                (EnvironmentColor::ThinLight, rgba_bytes(255, 255, 255, 0.25)),
                (EnvironmentColor::UltraThinLight, rgba_bytes(255, 255, 255, 0.15)),
                // Material colors dark
                (EnvironmentColor::UltraThickDark, rgba_bytes(0, 0, 0, 0.8)),
                (EnvironmentColor::ThickDark, rgba_bytes(0, 0, 0, 0.6)),
                (EnvironmentColor::RegularDark, rgba_bytes(0, 0, 0, 0.4)),
                (EnvironmentColor::ThinDark, rgba_bytes(0, 0, 0, 0.25)),
                (EnvironmentColor::UltraThinDark, rgba_bytes(0, 0, 0, 0.15)),
            ],
            dark: vec![
                (EnvironmentColor::Blue, rgba_bytes(10, 132, 255, 1.0)),
                (EnvironmentColor::Green, rgba_bytes(48, 209, 88, 1.0)),
                (EnvironmentColor::Indigo, rgba_bytes(94, 92, 230, 1.0)),
                (EnvironmentColor::Orange, rgba_bytes(255, 149, 10, 1.0)),
                (EnvironmentColor::Pink, rgba_bytes(255, 55, 95, 1.0)),
                (EnvironmentColor::Purple, rgba_bytes(191, 90, 242, 1.0)),
                (EnvironmentColor::Red, rgba_bytes(255, 69, 58, 1.0)),
                (EnvironmentColor::Teal, rgba_bytes(100, 210, 255, 1.0)),
                (EnvironmentColor::Yellow, rgba_bytes(255, 214, 10, 1.0)),
                (EnvironmentColor::Gray, rgba_bytes(142, 142, 147, 1.0)),
                (EnvironmentColor::Gray2, rgba_bytes(99, 99, 102, 1.0)),
                (EnvironmentColor::Gray3, rgba_bytes(72, 72, 74, 1.0)),
                (EnvironmentColor::Gray4, rgba_bytes(58, 58, 60, 1.0)),
                (EnvironmentColor::Gray5, rgba_bytes(44, 44, 46, 1.0)),
                (EnvironmentColor::Gray6, rgba_bytes(28, 28, 30, 1.0)),
                (EnvironmentColor::SystemBackground, rgba_bytes(28, 28, 30, 1.0)),
                (EnvironmentColor::SecondarySystemBackground, rgba_bytes(44, 44, 46, 1.0)),
                (EnvironmentColor::TertiarySystemBackground, rgba_bytes(58, 58, 60, 1.0)),
                (EnvironmentColor::Label, rgba_bytes(245, 245, 245, 1.0)),
                (EnvironmentColor::SecondaryLabel, rgba_bytes(152, 152, 159, 1.0)),
                (EnvironmentColor::TertiaryLabel, rgba_bytes(90, 90, 95, 1.0)),
                (EnvironmentColor::QuaternaryLabel, rgba_bytes(65, 65, 69, 1.0)),
                (EnvironmentColor::PlaceholderText, rgba_bytes(71, 71, 74, 1.0)),
                (EnvironmentColor::Link, rgba_bytes(9, 132, 255, 1.0)),
                (EnvironmentColor::SystemFill, rgba_bytes(61, 61, 65, 1.0)),
                (EnvironmentColor::SecondarySystemFill, rgba_bytes(57, 57, 61, 1.0)),
                (EnvironmentColor::TertiarySystemFill, rgba_bytes(50, 50, 54, 1.0)),
                (EnvironmentColor::QuaternarySystemFill, rgba_bytes(44, 44, 48, 1.0)),
                (EnvironmentColor::OpaqueSeparator, rgba_bytes(61, 61, 65, 1.0)),
                (EnvironmentColor::Separator, rgba_bytes(255, 255, 255, 0.15)),
                (EnvironmentColor::Accent, rgba_bytes(10, 132, 255, 1.0)),
                (EnvironmentColor::LightText, rgba_bytes(0, 0, 0, 1.0)),
                (EnvironmentColor::DarkText, rgba_bytes(255, 255, 255, 1.0)),
                // Material colors
                (EnvironmentColor::UltraThick, rgba_bytes(0, 0, 0, 0.8)),
                (EnvironmentColor::Thick, rgba_bytes(0, 0, 0, 0.6)),
                (EnvironmentColor::Regular, rgba_bytes(0, 0, 0, 0.4)),
                (EnvironmentColor::Thin, rgba_bytes(0, 0, 0, 0.25)),
                (EnvironmentColor::UltraThin, rgba_bytes(0, 0, 0, 0.15)),
                // Material colors light
                (EnvironmentColor::UltraThickLight, rgba_bytes(255, 255, 255, 0.8)),
                (EnvironmentColor::ThickLight, rgba_bytes(255, 255, 255, 0.6)),
                (EnvironmentColor::RegularLight, rgba_bytes(255, 255, 255, 0.4)),
                (EnvironmentColor::ThinLight, rgba_bytes(255, 255, 255, 0.25)),
                (EnvironmentColor::UltraThinLight, rgba_bytes(255, 255, 255, 0.15)),
                // Material colors dark
                (EnvironmentColor::UltraThickDark, rgba_bytes(0, 0, 0, 0.8)),
                (EnvironmentColor::ThickDark, rgba_bytes(0, 0, 0, 0.6)),
                (EnvironmentColor::RegularDark, rgba_bytes(0, 0, 0, 0.4)),
                (EnvironmentColor::ThinDark, rgba_bytes(0, 0, 0, 0.25)),
                (EnvironmentColor::UltraThinDark, rgba_bytes(0, 0, 0, 0.15)),
            ],
        }
    }
}

impl<C: Widget> Layout for ThemeManager<C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let mut response = requested_size;

        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
            response = self.child.calculate_size(requested_size, &mut LayoutContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
            });
        });

        response
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let alignment = self.alignment();
        let position = self.position();
        let dimension = self.dimension();

        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
            self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
            self.child.position_children(&mut LayoutContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
            })
        })
    }
}

impl<C: Widget> Update for ThemeManager<C> {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
            self.child.process_update(&mut UpdateContext {
                text: ctx.text,
                image: ctx.image,
                env: inner,
            })
        })
    }
}

impl<C: Widget> Initialize for ThemeManager<C> {
    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
            self.child.process_initialization(&mut InitializationContext {
                env: inner,
            })
        })
    }
}

impl<C: Widget> OtherEventHandler for ThemeManager<C> {
    fn process_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
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

impl<C: Widget> WindowEventHandler for ThemeManager<C> {
    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
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

impl<C: Widget> AccessibilityEventHandler for ThemeManager<C> {
    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
            self.child.process_accessibility_event(event, &mut AccessibilityEventContext {
                env: inner,
            })
        })
    }
}

impl<C: Widget> KeyboardEventHandler for ThemeManager<C> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
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

impl<C: Widget> MouseEventHandler for ThemeManager<C> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
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

impl<C: Widget> Focusable for ThemeManager<C> {
    fn process_focus_next(&mut self, ctx: &mut FocusContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
            self.child.process_focus_next(&mut FocusContext {
                env: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }

    fn process_focus_previous(&mut self, ctx: &mut FocusContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
            self.child.process_focus_previous(&mut FocusContext {
                env: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }

    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
            self.child.process_focus_request(&mut FocusContext {
                env: inner,
                focus_count: ctx.focus_count,
                available: ctx.available,
            })
        })
    }
}

impl<C: Widget> Accessibility for ThemeManager<C> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
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

impl<C: Widget> Render for ThemeManager<C> {
    fn render(&mut self, ctx: &mut RenderContext) {
        let theme = ctx.env.get::<Theme>().cloned().unwrap_or_default();

        let values = match theme {
            Theme::Light => &self.light,
            Theme::Dark => &self.dark
        };

        EnvironmentColor::with_all(values, ctx.env, |inner| {
            self.child.render(&mut RenderContext {
                render: ctx.render,
                text: ctx.text,
                image: ctx.image,
                env: inner,
            })
        })
    }
}

impl<C: Widget> Identifiable<WidgetId> for ThemeManager<C> {
    fn id(&self) -> WidgetId {
        self.child.id()
    }
}

impl<C: Widget> CommonWidget for ThemeManager<C> {
    ModifierWidgetImpl!(self, child: self.child);
}