use crate::environment::Environment;
use crate::focus::focus::Focus;
use crate::focus::Refocus;
use crate::state::StateSync;
use crate::widget::CommonWidget;

pub trait Focusable: CommonWidget + StateSync {
    fn request_focus(&mut self, env: &mut Environment) {
        self.set_focus(Focus::FocusRequested);
        env.request_focus(Refocus::FocusRequest);
    }

    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        self.capture_state(ctx.env);

        if self.is_focusable() {
            if self.get_focus() == Focus::FocusRequested {
                *ctx.focus_count += 1;
                self.set_focus(Focus::Focused);
            } else {
                self.set_focus(Focus::Unfocused);
            }
        } else {
            self.foreach_child_direct(&mut |child| {
                child.process_focus_request(ctx);
            });
        }
    }

    fn process_focus_next(&mut self, ctx: &mut FocusContext) {
        self.capture_state(ctx.env);

        if self.is_focusable() {
            if *ctx.available {
                // If the current item is currently focused, we allow focus to
                // be available for the next item.
                *ctx.available = self.get_focus() == Focus::Focused;
                *ctx.focus_count += 1;
                self.set_focus(Focus::Focused);
            } else {
                // If the current item is currently focused, we allow focus to
                // be available for the next item.
                *ctx.available = self.get_focus() == Focus::Focused;
                self.set_focus(Focus::Unfocused);
            }
        } else {
            self.foreach_child_direct(&mut |child| {
                child.process_focus_next(ctx);
            });
        }
    }

    fn process_focus_previous(&mut self, ctx: &mut FocusContext) {
        self.capture_state(ctx.env);

        if self.is_focusable() {
            if *ctx.available {
                // If the current item is currently focused, we allow focus to
                // be available for the next item.
                *ctx.available = self.get_focus() == Focus::Focused;
                *ctx.focus_count += 1;
                self.set_focus(Focus::Focused);
            } else {
                // If the current item is currently focused, we allow focus to
                // be available for the next item.
                *ctx.available = self.get_focus() == Focus::Focused;
                self.set_focus(Focus::Unfocused);
            }
        } else {
            self.foreach_child_direct_rev(&mut |child| {
                child.process_focus_previous(ctx);
            });
        }
    }
}

pub struct FocusContext<'a> {
    pub env: &'a mut Environment,
    pub focus_count: &'a mut u32,
    pub available: &'a mut bool,
}