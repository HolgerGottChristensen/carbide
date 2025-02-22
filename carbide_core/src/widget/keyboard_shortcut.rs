use crate::event::{Key, KeyboardEventHandler, ModifierKey, OtherEventContext, OtherEventHandler};
use crate::event::{KeyboardEvent, KeyboardEventContext, OtherEvent};
use crate::widget::managers::{ShortcutManager, ShortcutPressed, ShortcutReleased};
use crate::widget::{CommonWidget, Identifiable, Widget, WidgetId};
use crate::widget::{Empty, IntoWidget};
use crate::ModifierWidgetImpl;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(KeyboardEvent, OtherEvent)]
pub struct KeyboardShortcut<C> where C: Widget {
    child: C,
    key: Key,
    modifiers: ModifierKey
}

impl KeyboardShortcut<Empty> {
    pub fn new<C: IntoWidget>(child: C, key: impl Into<Key>, modifiers: ModifierKey) -> KeyboardShortcut<C::Output> {
        KeyboardShortcut {
            child: child.into_widget(),
            key: key.into(),
            modifiers,
        }
    }
}

impl<C: Widget> KeyboardEventHandler for KeyboardShortcut<C> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        let handle_event = match event {
            KeyboardEvent::Press { no_modifier_key, modifiers, .. } => {
                &self.key == no_modifier_key && &self.modifiers == modifiers
            }
            KeyboardEvent::Release { no_modifier_key, modifiers, .. } => {
                &self.key == no_modifier_key && &self.modifiers == modifiers
            }
            _ => false
        };

        if handle_event {
            if let Some(manager) = ctx.env.get_mut::<ShortcutManager>() {
                manager.shortcut(self.id());
            }
        }
    }
}

impl<C: Widget> OtherEventHandler for KeyboardShortcut<C> {
    fn process_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {
        if let Some(pressed) = event.value::<ShortcutPressed>() {
            if pressed.0 == self.id() {
                self.foreach_child_direct(&mut |child| {
                    child.process_other_event(event, &mut OtherEventContext {
                        text: ctx.text,
                        image: ctx.image,
                        env: ctx.env,
                        is_current: &true,
                        is_consumed: ctx.is_consumed,
                    });
                });
            }
        }

        if let Some(released) = event.value::<ShortcutReleased>() {
            if released.0 == self.id() {
                self.foreach_child_direct(&mut |child| {
                    child.process_other_event(event, &mut OtherEventContext {
                        text: ctx.text,
                        image: ctx.image,
                        env: ctx.env,
                        is_current: &true,
                        is_consumed: ctx.is_consumed,
                    });
                });
            }
        }
    }
}

impl<C: Widget> Identifiable for KeyboardShortcut<C> {
    fn id(&self) -> WidgetId {
        self.child.id()
    }
}

impl<C: Widget> CommonWidget for KeyboardShortcut<C> {
    ModifierWidgetImpl!(self, child: self.child);
}