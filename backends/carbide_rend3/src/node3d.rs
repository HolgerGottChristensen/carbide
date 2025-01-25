use std::fmt::Debug;
use std::sync::Arc;
use dyn_clone::DynClone;
use rend3::Renderer;
use carbide::environment::EnvironmentStack;
use carbide_core::state::StateSync;

pub trait AnyNode3D: DynClone + 'static {
    fn update(&mut self, renderer: &Arc<Renderer>, env: &mut EnvironmentStack);
}

dyn_clone::clone_trait_object!(AnyNode3D);

pub trait Node3D: AnyNode3D + Clone + private::Sealed {}

mod private {
    use super::AnyNode3D;

    pub trait Sealed {}

    impl<T> Sealed for T where T: AnyNode3D {}
}

