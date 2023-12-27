use std::cell::RefCell;
use std::future::Future;
use std::time::Duration;
pub use timer::Timer;

use crate::environment::Environment;

pub mod thread_task;
mod timer;

use futures::FutureExt;
use oneshot::TryRecvError;

use crate::event::{CustomEvent, EventSink, NoopEventSink};

thread_local! {
    static ASYNC_QUEUE: RefCell<Vec<Box<dyn Fn(&mut Environment) -> bool>>> = {
        RefCell::new(vec![])
    };

    static ASYNC_WAIT_QUEUE: RefCell<Vec<Box<dyn Fn(&mut Environment) -> bool>>> = {
        RefCell::new(vec![])
    };

    static EVENT_SINK: RefCell<Box<dyn EventSink>> = {
        RefCell::new(Box::new(NoopEventSink))
    }
}

/// Returns the old event_sink
pub fn set_event_sink(sink: impl EventSink + 'static) -> Box<dyn EventSink> {
    EVENT_SINK.with(|old_sink| {
        old_sink.replace(Box::new(sink))
    })
}

pub fn get_event_sink() -> Box<dyn EventSink> {
    EVENT_SINK.with(|e| e.borrow().clone())
}

pub fn spawn_task<T: Send + 'static>(
    task: impl Future<Output = T> + Send + 'static,
    cont: impl Fn(T, &mut Environment) + 'static,
) {
    let (sender, receiver) = oneshot::channel();

    let event_sink = EVENT_SINK.with(|e| e.borrow().clone());

    let task_with_oneshot = task.then(|message| async move {
        let _ = sender.send(message);
        event_sink.send(CustomEvent::Async);
        ()
    });

    let poll_message: Box<dyn Fn(&mut Environment) -> bool> = Box::new(move |env| -> bool {
        match receiver.try_recv() {
            Ok(message) => {
                cont(message, env);
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(e) => {
                eprintln!("carbide async task disconnected: {:?}", e);
                true
            }
        }
    });

    ASYNC_WAIT_QUEUE.with(|queue| queue.borrow_mut().push(poll_message));

    spawn(task_with_oneshot)
}

pub fn check_tasks(env: &mut Environment) {
    ASYNC_QUEUE.with(|queue| {
        ASYNC_WAIT_QUEUE.with(|wait_queue| {
            queue.borrow_mut().append(&mut wait_queue.borrow_mut());

            queue.borrow_mut().retain(|task| !task(env));
        });
    });
}

pub fn start_stream<T: Send + 'static>(
    receiver: std::sync::mpsc::Receiver<T>,
    next: impl Fn(T, &mut Environment) -> bool + 'static,
) {
    let poll_message: Box<dyn Fn(&mut Environment) -> bool> = Box::new(move |env| -> bool {
        let mut stop = false;
        loop {
            if stop {
                break;
            }
            match receiver.try_recv() {
                Ok(message) => {
                    stop = next(message, env);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    stop = true;
                }
            }
        }
        stop
    });

    ASYNC_WAIT_QUEUE.with(|queue| queue.borrow_mut().push(poll_message));
}


#[macro_export]
macro_rules! task {
    ($state:ident := $body:block) => {{
        let $state = $state.clone();
        carbide::asynchronous::spawn_task(async move { $body }, move |result, env| {
            $state.clone().set_value(result);
        });
    }};
    ($state:ident := $body:block $(, $state1:ident := $body1:block)*) => {{
        let $state = $state.clone();
        carbide::asynchronous::spawn_task(async move { $body }, move |result, env| {
            $state.clone().set_value(result);
            task!(env, $($state1 :=  $body1),*);
        });
    }};
    ($body:block, move |$result:ident, $env_param:ident: &mut Environment| $cont:block) => {{
        carbide::asynchronous::spawn_task(
            async move { $body },
            move |$result, $env_param: &mut carbide::environment::Environment| $cont,
        )
    }};
}

pub trait SpawnTask<G: Send + 'static> {
    fn spawn(self, env: &mut Environment, cont: impl Fn(G, &mut Environment) + 'static);
}

impl<G: Send + 'static, T: Future<Output = G> + Send + 'static> SpawnTask<G> for T {
    fn spawn(self, _env: &mut Environment, cont: impl Fn(G, &mut Environment) + 'static) {
        spawn_task(self, cont);
    }
}

pub trait StartStream<G: Send + 'static> {
    fn start_stream(self, env: &mut Environment, cont: impl Fn(G, &mut Environment) -> bool + 'static);
}

impl<T: Send + 'static> StartStream<T> for std::sync::mpsc::Receiver<T> {
    fn start_stream(self, _env: &mut Environment, cont: impl Fn(T, &mut Environment) -> bool + 'static) {
        start_stream(self, cont);
    }
}

#[cfg(feature = "tokio")]
use std::sync::OnceLock;
#[cfg(feature = "tokio")]
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

#[cfg(feature = "tokio")]
fn tokio_spawn<F, T>(future: F) where F: Future<Output = T> + Send + 'static, T: Send + 'static {
    use tokio::runtime::Runtime;
    let runtime = RUNTIME.get_or_init(|| {
        Runtime::new().unwrap()
    });

    runtime.spawn(future);
}

pub fn spawn<F, T>(future: F) where F: Future<Output = T> + Send + 'static, T: Send + 'static {
    #[cfg(feature = "tokio")]
    tokio_spawn(future);

    #[cfg(all(feature = "async-std", not(feature = "tokio")))]
    async_std::task::spawn(future);

    #[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
    std::thread::spawn(|| {
        use futures::executor::block_on;
        block_on(future)
    });
}

pub async fn sleep(duration: Duration) {
    #[cfg(feature = "tokio")]
    tokio::time::sleep(duration).await;

    #[cfg(all(feature = "async-std", not(feature = "tokio")))]
    async_std::task::sleep(duration).await;

    #[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
    std::thread::sleep(duration)
}
