use crate::draw::image::ImageId;
use crate::draw::Texture;

pub trait ImageContext {
    fn texture_exist(&self, id: &ImageId) -> bool;
    fn texture_dimensions(&self, id: &ImageId) -> Option<(u32, u32)>;
    fn update_texture(&mut self, id: ImageId, texture: Texture) -> bool;
}

pub struct NOOPImageContext;

impl ImageContext for NOOPImageContext {
    fn texture_exist(&self, _id: &ImageId) -> bool {
        unimplemented!()
    }

    fn texture_dimensions(&self, _id: &ImageId) -> Option<(u32, u32)> {
        unimplemented!()
    }

    fn update_texture(&mut self, _id: ImageId, _texture: Texture) -> bool {
        unimplemented!()
    }
}