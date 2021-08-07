use crate::event::WidgetEvent;
use crate::flags::Flags;
use crate::focus::focus::Focus;
use crate::focus::Refocus;
use crate::prelude::{CommonWidget, Environment};
use crate::state::StateSync;

pub trait Focusable: CommonWidget + StateSync {
    fn focus_retrieved(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment);

    fn focus_dismissed(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment);

    fn get_focus(&self) -> Focus;

    /// For normal setting of focus use set_focus_and_request
    /// which makes sure the request is processed.
    fn set_focus(&mut self, focus: Focus);

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment);

    fn process_focus_request(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment) -> bool;

    fn request_focus(&mut self, env: &mut Environment) {
        if self.get_focus() == Focus::Unfocused {
            self.set_focus_and_request(Focus::FocusRequested, env);
        }
    }

    fn release_focus(&mut self, env: &mut Environment) {
        if self.get_focus() == Focus::Focused {
            self.set_focus_and_request(Focus::FocusReleased, env);
        }
    }

    /// Returns a boolean whether any widget in the tree now contains focus.
    /// This is useful for checking whether the next tab should focus the first element
    fn process_focus_request_default(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment) -> bool {
        self.capture_state(env);

        let mut any_focus = false;

        if self.flag().contains(Flags::FOCUSABLE) {
            let focus = self.get_focus();
            if focus == Focus::FocusRequested {
                self.set_focus(Focus::Focused);
                self.focus_retrieved(event, focus_request, env);
                any_focus = true;
            } else if focus != Focus::Unfocused {
                self.set_focus(Focus::Unfocused);
                self.focus_dismissed(event, focus_request, env);
            }
        }

        self.release_state(env);

        for child in self.proxied_children() {
            if child.process_focus_request(event, focus_request, env) {
                any_focus = true;
            }
        }

        any_focus
    }

    fn process_focus_next(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool;

    fn process_focus_next_default(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool {
        self.capture_state(env);

        let mut focus_child =
            if self.flag().contains(Flags::FOCUSABLE) {
                //println!("{}, {:?}", focus_up_for_grab, self.get_focus());
                if focus_up_for_grab {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event, focus_request, env);
                    //println!("{}, {:?}", focus_up_for_grab, self.get_focus());
                    false
                } else if self.get_focus() == Focus::FocusReleased {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event, focus_request, env);
                    //println!("{}, {:?}", focus_up_for_grab, self.get_focus());
                    true
                } else {
                    false
                }
            } else {
                focus_up_for_grab
            };

        self.release_state(env);

        for child in self.proxied_children() {
            focus_child = child.process_focus_next(event, focus_request, focus_child, env);
        }

        focus_child
    }

    fn process_focus_previous(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool;

    fn process_focus_previous_default(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment) -> bool {
        self.capture_state(env);

        let mut focus_child =
            if self.flag().contains(Flags::FOCUSABLE) {
                if focus_up_for_grab {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event, focus_request, env);
                    false
                } else if self.get_focus() == Focus::FocusReleased {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event, focus_request, env);
                    true
                } else {
                    false
                }
            } else {
                focus_up_for_grab
            };

        self.release_state(env);

        for child in self.proxied_children_rev() {
            focus_child = child.process_focus_previous(event, focus_request, focus_child, env);
        }

        focus_child
    }
}