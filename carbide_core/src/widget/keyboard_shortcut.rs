use std::any::Any;
use carbide::event::{KeyboardEvent, KeyboardEventContext, OtherEvent};
use carbide::ModifierWidgetImpl;
use carbide::widget::{CommonWidget, Identifiable, Widget, WidgetId};
use crate::event::{Key, KeyboardEventHandler, ModifierKey, OtherEventContext};
use crate::widget::{Empty, IntoWidget};

#[derive(Copy, Clone, Debug)]
pub struct KeyboardShortcutPressed;

#[derive(Copy, Clone, Debug)]
pub struct KeyboardShortcutReleased;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(KeyboardEvent)]
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
        match event {
            KeyboardEvent::Press { no_modifier_key, modifiers, .. } if &self.key == no_modifier_key && &self.modifiers == modifiers => {

                self.child.process_other_event(&OtherEvent::Key(KeyboardShortcutPressed.type_id()), &mut OtherEventContext {
                    text: ctx.text,
                    image: ctx.image,
                    env: ctx.env,
                })

            }
            KeyboardEvent::Release { no_modifier_key, modifiers, .. } if &self.key == no_modifier_key && &self.modifiers == modifiers => {

                self.child.process_other_event(&OtherEvent::Key(KeyboardShortcutReleased.type_id()), &mut OtherEventContext {
                    text: ctx.text,
                    image: ctx.image,
                    env: ctx.env,
                })

            }
            _ => {}
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