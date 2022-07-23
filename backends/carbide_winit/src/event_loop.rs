use std::ops::Deref;
use winit::event_loop::{EventLoop as WinitEventLoop, EventLoopWindowTarget};
use winit::window::{Window, WindowBuilder};
use crate::WinitWindow;

pub enum EventLoop<T: 'static> {
    Owned(WinitEventLoop<T>),
    StaticBorrow(&'static EventLoopWindowTarget<T>),
    None
}

impl<T: 'static> EventLoop<T> {
    pub fn create_inner_window(&self, builder: WindowBuilder) -> Window {
        match self {
            EventLoop::Owned(e) => {
                builder.build(e).unwrap()
            },
            EventLoop::StaticBorrow(e) => {
                builder.build(*e).unwrap()
            },
            EventLoop::None => panic!("Not available")
        }
    }
}

impl<T: 'static> Default for EventLoop<T> {
    fn default() -> Self {
        EventLoop::None
    }
}