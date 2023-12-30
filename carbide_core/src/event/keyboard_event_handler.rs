use bitflags::Flags;
use crate::environment::Environment;
use crate::event::{Key, KeyboardEvent, ModifierKey};
use crate::flags::WidgetFlag as CarbideFlags;
use crate::focus::{Focus, Focusable, Refocus};
use crate::state::StateSync;
use crate::widget::CommonWidget;

pub trait KeyboardEventHandler: CommonWidget + StateSync + Focusable {
    /// A function that will get called when a keyboard event occurs.
    /// This event will be given to all widgets, no matter if they are in focus or not.
    /// This is because the focus will be decided by the widgets themselves.
    #[allow(unused_variables)]
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {}

    /// This function is used to delegate the keyboard events, first to its own handle event
    /// [KeyboardEventHandler::handle_keyboard_event()] and then to the direct children.
    /// Notice this means that proxy widgets will receive the events and should make sure to
    /// delegate events to their children themselves. This is opposed to layout where the
    /// proxy widgets will be skipped in the tree. If you override this, you will need to
    /// manage the events yourself. Overriding this you are thereby able to restrict events to
    /// a widgets children.
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        if self.flag().contains(CarbideFlags::FOCUSABLE) && self.get_focus() == Focus::Focused && env.is_event_current() {
            match event {
                KeyboardEvent::Press(key, modifier) => {
                    if key == &Key::Tab {
                        if modifier.shift_key() {
                            self.set_focus(Focus::FocusReleased);
                            env.request_focus(Refocus::FocusPrevious);
                        } else if modifier.is_empty() {
                            self.set_focus(Focus::FocusReleased);
                            env.request_focus(Refocus::FocusNext);
                        }
                    }
                }
                _ => (),
            }
        }

        if env.is_event_current() {
            self.capture_state(env);
            self.handle_keyboard_event(event, env);
            self.release_state(env);
        }

        self.foreach_child_direct(&mut |child| {
            child.process_keyboard_event(event, env);
        });
    }
}
