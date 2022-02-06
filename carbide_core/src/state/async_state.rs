use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::rc::Rc;

use futures::{FutureExt};
use oneshot::{Receiver, TryRecvError};
use carbide_core::prelude::{NewStateSync, Listenable, Listener};

use crate::environment::Environment;
use crate::state::{ReadState, State, StateContract, TState};
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct AsyncState<T>
    where
        T: StateContract + Send,
{
    value: T,
    receiver: Option<Rc<Receiver<T>>>,
}

impl<T: StateContract + Send + 'static + Default> AsyncState<T> {
    pub fn new(a: impl Future<Output=T> + Send + 'static) -> TState<T> {
        let (sender, receiver) = oneshot::channel();

        let task = a.then(|message| async move {
            let _ = sender.send(message);
            ()
        });

        #[cfg(feature = "async-std")]
        let _ = async_std::task::spawn(task);

        Box::new(AsyncState {
            value: T::default(),
            receiver: Some(Rc::new(receiver)),
        }).into()
    }
}

impl<T: StateContract + Send + 'static> AsyncState<T> {
    pub fn new_with_default(a: impl Future<Output=T> + Send + 'static, default: T) -> TState<T> {
        let (sender, receiver) = oneshot::channel();

        let task = a.then(|message| async move {
            let _ = sender.send(message);
            ()
        });

        #[cfg(feature = "async-std")]
            let _ = async_std::task::spawn(task);

        Box::new(AsyncState {
            value: default,
            receiver: Some(Rc::new(receiver)),
        }).into()
    }
}

impl<T: StateContract + Send> NewStateSync for AsyncState<T> {
    fn sync(&mut self, env: &mut Environment) {
        if let Some(receiver) = &self.receiver {
            match receiver.try_recv() {
                Ok(message) => {
                    println!("Received asynchronous state");
                    self.value = message;
                    self.receiver = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(e) => {
                    eprintln!("{:?}", e)
                }
            }
        }
    }
}

impl<T: StateContract + Send> Listenable<T> for AsyncState<T> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>) {
        todo!()
    }
}

impl<T: StateContract + Send> ReadState<T> for AsyncState<T> {
    fn value(&self) -> ValueRef<T> {
        ValueRef::Borrow(&self.value)
    }

}

impl<T: StateContract + Send> State<T> for AsyncState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value(&mut self, value: T) {
        self.value = value
    }

    fn notify(&self) {
        todo!()
    }
}

impl<T: StateContract + Send + 'static> Debug for AsyncState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<T: StateContract + Send + 'static> Into<TState<T>> for Box<AsyncState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}
