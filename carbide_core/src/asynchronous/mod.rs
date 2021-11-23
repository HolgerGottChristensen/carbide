use std::future::Future;

use crate::prelude::Environment;

pub mod executor;
pub mod thread_task;

#[macro_export]
macro_rules! task {
    ($env:ident, $state:ident := $body:block) => {
        {
            let $state = $state.clone();
            $env.spawn_task(
                async move {$body},
                move |result, env| {
                    $state.clone().set_value(result)
                }
            )
        }
    };
    ($env:ident, $body:block, move |$result:ident, $env_param:ident: &mut Environment| $cont:block) => {
        {
            $env.spawn_task(
                async move {$body},
                move |$result, $env_param: &mut carbide_core::environment::Environment| $cont
            )
        }
    };
}

pub trait SpawnTask<G: Send + 'static> {
    fn spawn(self, env: &mut Environment, cont: impl Fn(G, &mut Environment) + 'static);
}

impl<G: Send + 'static, T: Future<Output=G> + Send + 'static> SpawnTask<G> for T {
    fn spawn(self, env: &mut Environment, cont: impl Fn(G, &mut Environment) + 'static) {
        env.spawn_task(self, cont);
    }
}