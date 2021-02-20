use crate::prelude::{CommonWidget, GlobalState};
use crate::event_handler::WidgetEvent;
use crate::focus::focus::Focus;
use crate::flags::Flags;

pub trait Focusable<GS>: CommonWidget<GS> where GS: GlobalState {
    fn focus_retrieved(&mut self, event: &WidgetEvent);

    fn focus_dismissed(&mut self, event: &WidgetEvent);

    fn get_focus(&self) -> Focus;

    fn set_focus(&mut self, focus: Focus);

    fn process_focus_request(&mut self, event: &WidgetEvent) {

        if self.get_flag().contains(Flags::FOCUSABLE) {
            let focus = self.get_focus();
            if focus == Focus::FocusRequested {
                self.set_focus(Focus::Focused);
                self.focus_retrieved(event);
            } else if focus != Focus::Unfocused {
                self.set_focus(Focus::Unfocused);
                self.focus_dismissed(event);
            }
        }

        for child in self.get_proxied_children() {
            child.process_focus_request(event);
        }
    }

    fn process_focus_next(&mut self, event: &WidgetEvent, focus_up_for_grab: bool) -> bool {
        let mut focus_child =
            if self.get_flag().contains(Flags::FOCUSABLE) {
                if focus_up_for_grab {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event);
                    false
                } else if self.get_focus() == Focus::FocusReleased {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event);
                    true
                } else {
                    false
                }
            } else {
                focus_up_for_grab
            };

        for child in self.get_proxied_children() {
            focus_child = child.process_focus_next(event, focus_child);
        }

        return focus_child
    }

    fn process_focus_previous(&mut self, event: &WidgetEvent, focus_up_for_grab: bool) -> bool {
        let mut focus_child =
            if self.get_flag().contains(Flags::FOCUSABLE) {
                if focus_up_for_grab {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event);
                    false
                } else if self.get_focus() == Focus::FocusReleased {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event);
                    true
                } else {
                    false
                }
            } else {
                focus_up_for_grab
            };

        for child in self.get_proxied_children_rev() {
            focus_child = child.process_focus_previous(event, focus_child);
        }

        return focus_child
    }

}