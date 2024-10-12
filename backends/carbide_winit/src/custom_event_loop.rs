use winit::event_loop::{EventLoop as WinitEventLoop, EventLoopProxy, EventLoopWindowTarget};
use winit::window::{Window, WindowBuilder};

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

    pub fn proxy(&self) -> EventLoopProxy<T> {
        match self {
            EventLoop::Owned(e) => {
                e.create_proxy()
            }
            EventLoop::StaticBorrow(_) => panic!("Not available"),
            EventLoop::None => panic!("Not available")
        }
    }
}

impl<T: 'static> Default for EventLoop<T> {
    fn default() -> Self {
        EventLoop::None
    }
}