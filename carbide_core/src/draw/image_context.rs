use crate::draw::image::ImageId;
use crate::draw::Texture;

pub struct ImageContext(Box<dyn InnerImageContext>);

impl ImageContext {
    pub fn new<C: InnerImageContext + 'static>(context: C) -> Self {
        ImageContext(Box::new(context))
    }

    pub fn texture_exist(&self, id: &ImageId) -> bool {
        self.0.texture_exist(id)
    }

    pub fn texture_dimensions(&self, id: &ImageId) -> Option<(u32, u32)> {
        self.0.texture_dimensions(id)
    }

    pub fn update_texture(&mut self, id: ImageId, texture: Texture) {
        self.0.update_texture(id, texture);
    }
}

pub trait InnerImageContext {
    fn texture_exist(&self, id: &ImageId) -> bool;
    fn texture_dimensions(&self, id: &ImageId) -> Option<(u32, u32)>;
    fn update_texture(&mut self, id: ImageId, texture: Texture) -> bool;
}

pub struct NOOPImageContext;

impl InnerImageContext for NOOPImageContext {
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