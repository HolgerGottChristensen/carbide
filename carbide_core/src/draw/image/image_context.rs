use crate::environment::Environment;
use crate::draw::image::ImageId;
use crate::draw::Texture;

pub trait ImageContext {
    fn texture_exist(&self, id: &ImageId, env: &mut Environment) -> bool;
    fn texture_dimensions(&self, id: &ImageId, env: &mut Environment) -> Option<(u32, u32)>;
    fn update_texture(&mut self, id: ImageId, texture: Texture, env: &mut Environment) -> bool;
}

pub struct NOOPImageContext;

impl ImageContext for NOOPImageContext {
    fn texture_exist(&self, _id: &ImageId, _env: &mut Environment) -> bool {
        unimplemented!()
    }

    fn texture_dimensions(&self, _id: &ImageId, _env: &mut Environment) -> Option<(u32, u32)> {
        unimplemented!()
    }

    fn update_texture(&mut self, _id: ImageId, _texture: Texture, _env: &mut Environment) -> bool {
        unimplemented!()
    }
}