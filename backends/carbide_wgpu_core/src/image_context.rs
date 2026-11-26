use std::rc::Rc;
use carbide_core::draw::{Dimension, ImageContext, ImageFormat, ImageId, ImageMetrics, Texture};
use carbide_core::environment::Environment;
use carbide_core::render::{RenderInstruction, RenderInstructionCache};
use crate::{WgpuContextNew, WgpuTextureStore};

/// The image context of the wgpu renderers. Raster images go into the shared `WgpuTextureStore`
/// store, which the 2D and 3D renderers both draw from. Vector images go into core's
/// `RenderInstructionCache`.
#[derive(Debug, Clone, Copy, Default)]
pub struct WgpuImageContext;

impl ImageContext for WgpuImageContext {
    fn exist(&self, id: &ImageId, env: &mut Environment) -> bool {
        match id.format() {
            ImageFormat::Unknown => false,
            ImageFormat::Svg => {
                let cache = env.get::<RenderInstructionCache>().unwrap();
                cache.contains_key(id)
            }
            _ => {
                let textures = env.get::<WgpuTextureStore>().unwrap();
                textures.contains(id)
            }
        }
    }

    fn metrics(&self, id: &ImageId, env: &mut Environment) -> ImageMetrics {
        match id.format() {
            ImageFormat::Unknown => ImageMetrics::Unknown,
            ImageFormat::Svg => {
                let cache = env.get::<RenderInstructionCache>().unwrap();
                cache
                    .get(id)
                    .map(|vector| ImageMetrics::Vector { dimension: vector.0 })
                    .unwrap_or(ImageMetrics::Unknown)
            }
            _ => {
                let textures = env.get::<WgpuTextureStore>().unwrap();
                textures.get(id)
                    .map(|texture| ImageMetrics::Raster { width: texture.width, height: texture.height })
                    .unwrap_or(ImageMetrics::Unknown)
            }
        }
    }

    fn update_texture(&mut self, id: &ImageId, texture: Texture, env: &mut Environment) -> bool {
        let wgpu_context = env.get::<WgpuContextNew>().unwrap();
        let device = wgpu_context.device.clone();
        let queue = wgpu_context.queue.clone();

        let textures = env.get_mut::<WgpuTextureStore>().unwrap();
        textures.insert(id.clone(), texture, &device, &queue);

        true
    }

    fn update_vector(&mut self, id: &ImageId, description: Vec<RenderInstruction>, size: Dimension, env: &mut Environment) -> bool {
        let cache = env.get_mut::<RenderInstructionCache>().unwrap();

        cache.insert(id.clone(), Rc::new((size, description)));

        true
    }
}
