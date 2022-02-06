pub use animatable::Animatable;
pub use animation::Animation;

mod animation;
mod animatable;
pub mod animation_curve;

#[macro_export]
macro_rules! animate {
    // The case where we dont give a custom interpolation function we rely on the value implementing animate
    ($env:ident, $state:ident $(:= $from:expr)? => $to:expr $(, curve: $curve:expr)? $(, duration: $duration:expr)?) => {
        {
            use carbide_core::state::State;
            use carbide_core::state::ReadState;

            let start = $state.value().clone();
            let animation = carbide_core::animation::Animation::new(
                $state.clone().to_boxed_state(),
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
            $env.insert_animation(animation);
        }
    };
    // If we have the interpolation, we dont require the value to be animate, but instead use the provided function.
    ($env:ident, $state:ident $(:= $from:expr)? => $to:expr, interpolation: $interpolation:expr $(, curve: $curve:expr)? $(, duration: $duration:expr)?) => {
        {
            use carbide_core::state::State;
            use carbide_core::state::ReadState;


            let start = $state.value().clone();
            let animation = carbide_core::animation::Animation::new_custom(
                $state.clone().to_boxed_state(),
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
            $env.insert_animation(animation);
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



