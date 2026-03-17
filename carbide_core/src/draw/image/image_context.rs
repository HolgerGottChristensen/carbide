use carbide::draw::image::image_metrics::ImageMetrics;
use crate::environment::Environment;
use crate::draw::image::ImageId;
use crate::draw::{Dimension, Texture};
use crate::render::RenderInstruction;

pub trait ImageContext {
    /// Check if an image already exist in the context. This is used to determine whether to
    /// load an image.
    fn exist(&self, id: &ImageId, env: &mut Environment) -> bool;

    /// 
    fn metrics(&self, id: &ImageId, env: &mut Environment) -> ImageMetrics;
    fn update_texture(&mut self, id: &ImageId, texture: Texture, env: &mut Environment) -> bool;
    fn update_vector(&mut self, id: &ImageId, description: Vec<RenderInstruction>, size: Dimension, env: &mut Environment) -> bool;
}

pub struct NOOPImageContext;

impl ImageContext for NOOPImageContext {
    fn exist(&self, _id: &ImageId, _env: &mut Environment) -> bool {
        unimplemented!()
    }

    fn metrics(&self, id: &ImageId, env: &mut Environment) -> ImageMetrics {
        unimplemented!()
    }

    fn update_texture(&mut self, id: &ImageId, texture: Texture, env: &mut Environment) -> bool {
        unimplemented!()
    }

    fn update_vector(&mut self, id: &ImageId, description: Vec<RenderInstruction>, size: Dimension, env: &mut Environment) -> bool {
        unimplemented!()
    }
}