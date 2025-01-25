use carbide::environment::Environment;
use crate::focus::focus::Focus;
use crate::focus::{FocusManager, Refocus};
use crate::widget::{CommonWidget, WidgetSync};

pub trait Focusable: CommonWidget + WidgetSync {
    fn request_focus(&mut self, env: &mut Environment) {
        self.set_focus(Focus::FocusRequested);
        FocusManager::get(env, |manager| {
            manager.request_focus(Refocus::FocusRequest)
        });
    }

    fn request_blur(&mut self, env: &mut Environment) {
        self.set_focus(Focus::FocusReleased);
        FocusManager::get(env, |manager| {
            manager.request_focus(Refocus::FocusRequest)
        });
    }

    fn has_focus(&self) -> bool {
        self.get_focus() == Focus::Focused
    }

    fn process_focus_request(&mut self, ctx: &mut FocusContext) {
        self.sync(ctx.env);

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
        self.sync(ctx.env);

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
        self.sync(ctx.env);

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

pub struct FocusContext<'a, 'b: 'a> {
    pub env: &'a mut Environment<'b>,
    pub focus_count: &'a mut u32,
    pub available: &'a mut bool,
}