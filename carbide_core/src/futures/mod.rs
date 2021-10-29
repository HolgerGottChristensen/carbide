pub mod executor;

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