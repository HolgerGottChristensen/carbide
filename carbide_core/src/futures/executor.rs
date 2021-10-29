// use std::future::Future;
// use oneshot::Receiver;
// use futures::FutureExt;
//
// // This is currently derived from the iced futures crate.
// pub trait Executor<T> {
//     fn new() -> Result<Box<dyn Executor<T>>, futures::io::Error>;
//
//     /*fn spawn(&self, task: impl Future<Output = T> + Send + 'static) -> Receiver<T> {
//         let (sender, receiver) = oneshot::channel();
//
//         let task_with_oneshot = task.then(|message| async move {
//             let _ = sender.send(message);
//             ()
//         });
//
//         self.spawn_internal(task_with_oneshot);
//         receiver
//     }*/
//
//     fn spawn(&self, future: impl Future<Output = ()> + Send + 'static);
//
//
//     /// Runs the given closure inside the [`Executor`].
//     ///
//     /// Some executors, like `tokio`, require some global state to be in place
//     /// before creating futures. This method can be leveraged to set up this
//     /// global state, call a function, restore the state, and obtain the result
//     /// of the call.
//     fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
//         f()
//     }
// }