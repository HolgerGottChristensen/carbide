use std::fmt::{Debug, Formatter};
use crate::accessibility::Accessibility;
use crate::environment::{EnvironmentStack, Key};
use crate::render::Render;
use std::time::Instant;
use carbide::animation::Animation;
use carbide::state::StateContract;

#[derive(Debug)]
pub struct AnimationManager {
    frame_count: u32,
    frame_time: Instant,
    out_of_band_animations: OutOfBandAnimation
}

impl AnimationManager {
    pub fn new() -> AnimationManager {
        AnimationManager {
            frame_count: 0,
            frame_time: Instant::now(),
            out_of_band_animations: OutOfBandAnimation(vec![]),
        }
    }

    pub fn number_of_animation_frames(&self) -> u32 {
        self.frame_count
    }

    pub fn request_animation_frame(&mut self) {
        self.frame_count = self.frame_count.max(1);
    }

    pub fn request_multiple_animation_frames(&mut self, n: u32) {
        self.frame_count = self.frame_count.max(n);
    }

    pub fn take_frame(&mut self) -> bool {
        if self.frame_count > 0 {
            self.frame_count -= 1;
            true
        } else {
            self.out_of_band_animations.0.len() > 0
        }
    }

    pub fn frame_time(&self) -> Instant {
        self.frame_time
    }

    pub fn update_frame_time(&mut self) {
        self.frame_time = Instant::now();
        self.out_of_band_animations.0.retain(|update_animation| !update_animation(&self.frame_time));
    }

    pub fn insert_animation<A: StateContract>(&mut self, animation: Animation<A>) {
        let poll = move |time: &Instant| {
            let mut animation = animation.clone();
            animation.update(time)
        };

        self.out_of_band_animations.0.push(Box::new(poll));
    }

    pub fn get(env_stack: &mut EnvironmentStack, f: impl FnOnce(&mut AnimationManager)) {
        if let Some(manager) = env_stack.get_mut::<AnimationManager>() {
            f(manager)
        }
    }
}

impl Key for AnimationManager {
    type Value = AnimationManager;
}

struct OutOfBandAnimation(Vec<Box<dyn Fn(&Instant) -> bool>>);

impl Debug for OutOfBandAnimation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutOfBandAnimation")
            .finish_non_exhaustive()
    }
}