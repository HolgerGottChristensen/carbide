use crate::prelude::{CommonWidget, GlobalState, Environment};
use crate::event_handler::WidgetEvent;
use crate::focus::focus::Focus;
use crate::flags::Flags;
use crate::focus::Refocus;

pub trait Focusable<GS>: CommonWidget<GS> where GS: GlobalState {
    fn focus_retrieved(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &mut GS);

    fn focus_dismissed(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &mut GS);

    fn get_focus(&self) -> Focus;

    /// For normal setting of focus use set_focus_and_request
    /// which makes sure the request is processed.
    fn set_focus(&mut self, focus: Focus);

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment<GS>);

    /// Returns a boolean whether any widget in the tree now contains focus.
    /// This is useful for checking whether the next tab should focus the first element
    fn process_focus_request(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &mut GS) -> bool {

        let mut any_focus = false;

        if self.get_flag().contains(Flags::FOCUSABLE) {
            let focus = self.get_focus();
            if focus == Focus::FocusRequested {
                self.set_focus(Focus::Focused);
                self.focus_retrieved(event, focus_request, env, global_state);
                any_focus = true;
            } else if focus != Focus::Unfocused {
                self.set_focus(Focus::Unfocused);
                self.focus_dismissed(event, focus_request, env, global_state);
            }
        }

        for child in self.get_proxied_children() {
            if child.process_focus_request(event, focus_request, env, global_state) {
                any_focus = true;
            }
        }

        any_focus
    }

    fn process_focus_next(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &mut GS) -> bool {
        let mut focus_child =
            if self.get_flag().contains(Flags::FOCUSABLE) {
                if focus_up_for_grab {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event, focus_request, env, global_state);
                    false
                } else if self.get_focus() == Focus::FocusReleased {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event, focus_request, env, global_state);
                    true
                } else {
                    false
                }
            } else {
                focus_up_for_grab
            };

        for child in self.get_proxied_children() {
            focus_child = child.process_focus_next(event, focus_request, focus_child, env, global_state);
        }

        return focus_child
    }

    fn process_focus_previous(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &mut GS) -> bool {
        let mut focus_child =
            if self.get_flag().contains(Flags::FOCUSABLE) {
                if focus_up_for_grab {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event, focus_request, env, global_state);
                    false
                } else if self.get_focus() == Focus::FocusReleased {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event, focus_request, env, global_state);
                    true
                } else {
                    false
                }
            } else {
                focus_up_for_grab
            };

        for child in self.get_proxied_children_rev() {
            focus_child = child.process_focus_previous(event, focus_request, focus_child, env, global_state);
        }

        return focus_child
    }

}