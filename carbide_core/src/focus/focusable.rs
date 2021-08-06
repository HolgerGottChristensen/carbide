use crate::event_handler::WidgetEvent;
use crate::flags::Flags;
use crate::focus::focus::Focus;
use crate::focus::Refocus;
use crate::prelude::{CommonWidget, Environment, GlobalStateContract};
use crate::state::global_state::GlobalStateContainer;
use crate::state::state_sync::StateSync;

pub trait Focusable<GS>: CommonWidget<GS> + StateSync<GS> where GS: GlobalStateContract {
    fn focus_retrieved(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>);

    fn focus_dismissed(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>);

    fn get_focus(&self) -> Focus;

    /// For normal setting of focus use set_focus_and_request
    /// which makes sure the request is processed.
    fn set_focus(&mut self, focus: Focus);

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment<GS>);

    fn process_focus_request(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) -> bool;

    fn request_focus(&mut self, env: &mut Environment<GS>) {
        if self.get_focus() == Focus::Unfocused {
            self.set_focus_and_request(Focus::FocusRequested, env);
        }
    }

    fn release_focus(&mut self, env: &mut Environment<GS>) {
        if self.get_focus() == Focus::Focused {
            self.set_focus_and_request(Focus::FocusReleased, env);
        }
    }

    /// Returns a boolean whether any widget in the tree now contains focus.
    /// This is useful for checking whether the next tab should focus the first element
    fn process_focus_request_default(&mut self, event: &WidgetEvent, focus_request: &Refocus, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) -> bool {
        self.capture_state(env, global_state);

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

        self.release_state(env);

        for child in self.get_proxied_children() {
            if child.process_focus_request(event, focus_request, env, global_state) {
                any_focus = true;
            }
        }

        any_focus
    }

    fn process_focus_next(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) -> bool;

    fn process_focus_next_default(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) -> bool {
        self.capture_state(env, global_state);

        let mut focus_child =
            if self.get_flag().contains(Flags::FOCUSABLE) {
                //println!("{}, {:?}", focus_up_for_grab, self.get_focus());
                if focus_up_for_grab {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event, focus_request, env, global_state);
                    //println!("{}, {:?}", focus_up_for_grab, self.get_focus());
                    false
                } else if self.get_focus() == Focus::FocusReleased {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event, focus_request, env, global_state);
                    //println!("{}, {:?}", focus_up_for_grab, self.get_focus());
                    true
                } else {
                    false
                }
            } else {
                focus_up_for_grab
            };

        self.release_state(env);

        for child in self.get_proxied_children() {
            focus_child = child.process_focus_next(event, focus_request, focus_child, env, global_state);
        }

        focus_child
    }

    fn process_focus_previous(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) -> bool;

    fn process_focus_previous_default(&mut self, event: &WidgetEvent, focus_request: &Refocus, focus_up_for_grab: bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) -> bool {
        self.capture_state(env, global_state);

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

        self.release_state(env);

        for child in self.get_proxied_children_rev() {
            focus_child = child.process_focus_previous(event, focus_request, focus_child, env, global_state);
        }

        focus_child
    }
}