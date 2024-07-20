use carbide::state::StateSync;
use crate::draw::InnerImageContext;
use crate::environment::Environment;
use crate::event::{Key, ModifierKey};
use crate::focus::Focusable;
use crate::text::InnerTextContext;
use crate::widget::{CommonWidget, WidgetSync};

pub trait KeyboardEventHandler: CommonWidget + WidgetSync + Focusable {
    /// A function that will get called when a keyboard event occurs.
    /// This event will be given to all widgets, no matter if they are in focus or not.
    /// This is because the focus will be decided by the widgets themselves.
    #[allow(unused_variables)]
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {}

    /// This function is used to delegate the keyboard events, first to its own handle event
    /// [KeyboardEventHandler::handle_keyboard_event()] and then to the direct children.
    /// Notice this means that proxy widgets will receive the events and should make sure to
    /// delegate events to their children themselves. This is opposed to layout where the
    /// proxy widgets will be skipped in the tree. If you override this, you will need to
    /// manage the events yourself. Overriding this you are thereby able to restrict events to
    /// a widgets children.
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        /*if bitflags::Flags::contains(&self.flag(), CarbideFlags::FOCUSABLE) && self.get_focus() == Focus::Focused && *ctx.is_current {
            match event {
                KeyboardEvent::Press(key, modifier) => {
                    if key == &Key::Tab {
                        if modifier.shift_key() {
                            self.set_focus(Focus::FocusReleased);
                            ctx.env.request_focus(Refocus::FocusPrevious);
                        } else if bitflags::Flags::is_empty(modifier) {
                            self.set_focus(Focus::FocusReleased);
                            ctx.env.request_focus(Refocus::FocusNext);
                        }
                    }
                }
                _ => (),
            }
        }*/

        if *ctx.is_current {
            self.sync(ctx.env);
            self.handle_keyboard_event(event, ctx);
        }

        self.foreach_child_direct(&mut |child| {
            child.process_keyboard_event(event, ctx);
        });
    }
}


pub struct KeyboardEventContext<'a> {
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
    pub env: &'a mut Environment,
    pub is_current: &'a bool,
    pub window_id: &'a u64,
    pub prevent_default: &'a mut bool,
}

impl KeyboardEventContext<'_> {
    pub fn prevent_default(&mut self) {
        *self.prevent_default = true;
    }
}


#[derive(Clone, Debug)]
pub enum KeyboardEvent {
    Press(Key, ModifierKey),
    Release(Key, ModifierKey),
    Ime(Ime),
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub enum Ime {
    PreEdit(String, Option<(usize, usize)>),
    Commit(String),
}
