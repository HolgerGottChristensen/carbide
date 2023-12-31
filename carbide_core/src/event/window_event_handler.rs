use crate::draw::{Dimension, InnerImageContext, Scalar};
use crate::environment::Environment;
use crate::state::StateSync;
use crate::text::InnerTextContext;
use crate::widget::CommonWidget;

pub trait WindowEventHandler: CommonWidget + StateSync {
    #[allow(unused_variables)]
    fn handle_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {}

    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        if *ctx.is_current {
            self.capture_state(ctx.env);
            self.handle_window_event(event, ctx);
            self.release_state(ctx.env);
        }

        self.foreach_child_direct(&mut |child| {
            child.process_window_event(event, ctx);
        });
    }
}


pub struct WindowEventContext<'a> {
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
    pub env: &'a mut Environment,
    pub is_current: &'a bool,
    pub window_id: &'a u64,
}


#[derive(Clone, Debug)]
pub enum WindowEvent {
    Resize(Dimension),
    Focus,
    UnFocus,
    Redraw,
    CloseRequested,
    ScaleFactorChanged(Scalar),
    ThemeChanged(Theme)
}

#[derive(Clone, Debug)]
pub enum Theme {
    Light,
    Dark
}