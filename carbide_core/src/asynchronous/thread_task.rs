#![allow(unsafe_code)]
// This file is originally from: https://github.com/PolyMeilex/rfd/blob/master/src/backend/win_cid/thread_future.rs

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use std::task::{Context, Poll, Waker};

struct FutureState {
    waker: Option<Waker>,
    data: Option<()>,
}

unsafe impl Send for FutureState {}

pub struct ThreadTask {
    state: Arc<Mutex<FutureState>>,
}

unsafe impl Send for ThreadTask {}

impl ThreadTask {
    pub fn new<F: FnOnce() + Send + 'static>(f: F) -> Self {
        let state = Arc::new(Mutex::new(FutureState {
            waker: None,
            data: None,
        }));

        {
            let state = state.clone();
            std::thread::spawn(move || {
                let mut state = state.lock().unwrap();

                f();

                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }
            });
        }

        Self { state }
    }
}

impl std::future::Future for ThreadTask {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();

        if state.data.is_some() {
            Poll::Ready(())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}