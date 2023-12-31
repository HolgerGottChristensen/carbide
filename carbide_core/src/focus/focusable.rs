use bitflags::Flags;
use crate::environment::Environment;
use crate::flags::WidgetFlag;
use crate::focus::focus::Focus;
use crate::focus::Refocus;
use crate::state::StateSync;
use crate::widget::CommonWidget;

pub trait Focusable: CommonWidget + StateSync {

    fn focus_children(&self) -> bool {
        true
    }

    #[allow(unused_variables)]
    fn focus_retrieved(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
    }

    #[allow(unused_variables)]
    fn focus_dismissed(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) {
    }

    fn set_focus_and_request(&mut self, focus: Focus, env: &mut Environment) {
        if focus == Focus::FocusReleased {
            env.request_focus(Refocus::FocusRequest)
        } else if focus == Focus::FocusRequested {
            env.request_focus(Refocus::FocusRequest)
        }
        self.set_focus(focus)
    }

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

    fn process_focus_request(
        &mut self,
        focus_request: &Refocus,
        env: &mut Environment,
    ) -> bool {
        self.capture_state(env);

        let mut any_focus = false;

        if self.flag().contains(WidgetFlag::FOCUSABLE) {
            let focus = self.get_focus();
            if focus == Focus::FocusRequested {
                self.set_focus(Focus::Focused);
                self.focus_retrieved(focus_request, env);
                any_focus = true;
            } else if focus != Focus::Unfocused {
                self.set_focus(Focus::Unfocused);
                self.focus_dismissed(focus_request, env);
            }
        }

        self.release_state(env);

        if self.focus_children() {
            self.foreach_child_direct(&mut |child| {
                if child.process_focus_request(focus_request, env) {
                    any_focus = true;
                }
            });
        }

        any_focus
    }

    fn process_focus_next(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.capture_state(env);

        let mut focus_child = if self.flag().contains(WidgetFlag::FOCUSABLE) {
            //println!("{}, {:?}", focus_up_for_grab, self.get_focus());
            if focus_up_for_grab {
                self.set_focus(Focus::Focused);
                self.focus_retrieved(focus_request, env);
                //println!("{}, {:?}", focus_up_for_grab, self.get_focus());
                false
            } else if self.get_focus() == Focus::FocusReleased {
                self.set_focus(Focus::Unfocused);
                self.focus_dismissed(focus_request, env);
                //println!("{}, {:?}", focus_up_for_grab, self.get_focus());
                true
            } else {
                false
            }
        } else {
            focus_up_for_grab
        };

        self.release_state(env);

        if self.focus_children() {
            self.foreach_child_direct(&mut |child| {
                focus_child = child.process_focus_next(focus_request, focus_child, env);
            });
        }

        focus_child
    }

    fn process_focus_previous(
        &mut self,
        focus_request: &Refocus,
        focus_up_for_grab: bool,
        env: &mut Environment,
    ) -> bool {
        self.capture_state(env);

        let mut focus_child = if self.flag().contains(WidgetFlag::FOCUSABLE) {
            if focus_up_for_grab {
                self.set_focus(Focus::Focused);
                self.focus_retrieved(focus_request, env);
                false
            } else if self.get_focus() == Focus::FocusReleased {
                self.set_focus(Focus::Unfocused);
                self.focus_dismissed(focus_request, env);
                true
            } else {
                false
            }
        } else {
            focus_up_for_grab
        };

        self.release_state(env);

        if self.focus_children() {
            self.foreach_child_direct_rev(&mut |child| {
                focus_child = child.process_focus_previous(focus_request, focus_child, env);
            });
        }

        focus_child
    }
}
