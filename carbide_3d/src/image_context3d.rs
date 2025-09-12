use carbide::draw::{ImageId, Texture};
use dyn_clone::{clone_trait_object, DynClone};
use std::fmt::Debug;
use carbide::any_debug::AnyDebug;
use carbide::environment::{Environment, EnvironmentKey};

pub trait InnerImageContext3d: AnyDebug + DynClone + 'static {
    fn texture_exist(&self, id: &ImageId, env: &mut Environment) -> bool;
    fn texture_dimensions(&self, id: &ImageId, env: &mut Environment) -> Option<(u32, u32)>;
    fn update_texture(&mut self, id: ImageId, texture: Texture, env: &mut Environment) -> bool;
}

clone_trait_object!(InnerImageContext3d);

#[derive(Debug, Clone)]
pub struct NoopImageContext3d;

impl InnerImageContext3d for NoopImageContext3d {
    fn texture_exist(&self, id: &ImageId, env: &mut Environment) -> bool {
        todo!()
    }

    fn texture_dimensions(&self, id: &ImageId, env: &mut Environment) -> Option<(u32, u32)> {
        todo!()
    }

    fn update_texture(&mut self, id: ImageId, texture: Texture, env: &mut Environment) -> bool {
        todo!()
    }
}