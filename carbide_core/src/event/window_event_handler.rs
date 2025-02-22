use crate::draw::{Dimension, ImageContext, Position, Scalar};
use crate::environment::{Environment};
use crate::text::TextContext;
use crate::widget::{CommonWidget, WidgetSync};

pub trait WindowEventHandler: CommonWidget + WidgetSync {
    #[allow(unused_variables)]
    fn handle_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {}

    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        if *ctx.is_current {
            self.sync(ctx.env);
            self.handle_window_event(event, ctx);
        }

        self.foreach_child_direct(&mut |child| {
            child.process_window_event(event, ctx);
        });
    }
}


pub struct WindowEventContext<'a, 'b: 'a> {
    pub text: &'a mut dyn TextContext,
    pub image: &'a mut dyn ImageContext,
    pub env: &'a mut Environment<'b>,
    pub is_current: &'a bool,
    pub window_id: &'a u64,
}


#[derive(Clone, Debug)]
pub enum WindowEvent {
    Resize(Dimension),
    Moved(Position),
    Focus,
    UnFocus,
    Redraw,
    CloseRequested,
    ScaleFactorChanged(Scalar),
    ThemeChanged
}

#[derive(Clone, Debug)]
pub enum Theme {
    Light,
    Dark
}