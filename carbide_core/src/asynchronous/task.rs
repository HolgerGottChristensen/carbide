use std::future::Future;
use crate::asynchronous::{AsyncContext, spawn_task};

pub struct Task<T, Task, Cont> where
    T: Send + 'static,
    Task: Future<Output = T> + Send + 'static,
    Cont: Fn(T, &mut AsyncContext) + 'static
{
    task: fn()->Task,
    cont: Cont,
}

impl<T: Send + 'static, Ta: Future<Output = T> + Send + 'static, Cont: Fn(T, &mut AsyncContext) + 'static> Task<T, Ta, Cont> {
    pub fn new(task: fn()->Ta, continuation: Cont) -> Task<T, Ta, Cont> {
        Task {
            task,
            cont: continuation
        }
    }

    pub fn start(self) {
        spawn_task((self.task)(), self.cont)
    }
}

impl<T: Send + 'static, Ta: Future<Output = T> + Send + 'static, Cont: Fn(T, &mut AsyncContext) + 'static + Clone> Clone for Task<T, Ta, Cont> {
    fn clone(&self) -> Self {
        Task::new(self.task.clone(), self.cont.clone())
    }
}