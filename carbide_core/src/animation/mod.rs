pub use animatable::Animatable;
pub use animation::Animation;

pub use crate::animation::animation_curve::*;
pub use animation_manager::AnimationManager;

mod animatable;
mod animation;
pub mod animation_curve;
mod animation_manager;

#[macro_export]
macro_rules! animate {
    // The case where we dont give a custom interpolation function we rely on the value implementing animate
    ($env:expr, $state:ident $(:= $from:expr)? => $to:expr $(, curve: $curve:expr)? $(, duration: $duration:expr)?) => {
        {
            if let Some(manager) = $env.get_mut::<$crate::animation::AnimationManager>() {
                use $crate::state::State;
                use $crate::state::ReadState;

                let start = $state.value().clone();
                let animation = carbide::animation::Animation::new(
                    $state.clone(),
                    start,
                    $to,
                )$(
                    .from($from)
                )?
                $(
                    .curve($curve)
                )?
                $(
                    .duration($duration)
                )?;
                manager.insert_animation(animation);
            }
        }
    };
    // If we have the interpolation, we dont require the value to be animate, but instead use the provided function.
    ($env:expr, $state:ident $(:= $from:expr)? => $to:expr, interpolation: $interpolation:expr $(, curve: $curve:expr)? $(, duration: $duration:expr)?) => {
        {
            if let Some(manager) = $env.get_mut::<$crate::animation::AnimationManager>() {
                use $crate::state::State;
                use $crate::state::ReadState;


                let start = $state.value().clone();
                let animation = carbide::animation::Animation::new_custom(
                    $state.clone(),
                    start,
                    $to,
                    $interpolation,
                )$(
                    .from($from)
                )?
                $(
                    .curve($curve)
                )?
                $(
                    .duration($duration)
                )?;

                manager.insert_animation(animation);
            }
        }
    };
}

/*fn hejsa() {
    let mut env = Environment::new(vec![], Dimension::new(100.0, 100.0), 1.0);
    let mut state = LocalState::new(0.0);
    animate!(env, state => 3.0);
    animate!(env, state := 1.0 => 3.0);
    animate!(env, state := 1.0 => 3.0, curve: elastic_in_out);
    animate!(env, state := 1.0 => 3.0, curve: elastic_in_out, duration: Duration::new(3, 0));
    animate!(env, state := 1.0 => 3.0, interpolation: f32::interpolate, curve: elastic_in_out, duration: Duration::new(3, 0));
    animate!(env, state := 1.0 => 3.0, interpolation: f32::interpolate, curve: elastic_in_out);
    animate!(env, state := 1.0 => 3.0, duration: Duration::new(3, 0));
    //animate!(env, state := 1 -> 3);
    //animate!(env, state := <-> 3);
    //animate!(env, state := -3 <-> 3);
    //animate!(env, state := -3 >-> 3);
}*/
