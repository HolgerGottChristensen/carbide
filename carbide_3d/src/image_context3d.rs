use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::draw::{ImageId, Texture};

pub trait InnerImageContext3d: Debug + DynClone + 'static {
    fn texture_exist(&self, id: &ImageId) -> bool;
    fn texture_dimensions(&self, id: &ImageId) -> Option<(u32, u32)>;
    fn update_texture(&mut self, id: ImageId, texture: Texture) -> bool;
}

clone_trait_object!(InnerImageContext3d);