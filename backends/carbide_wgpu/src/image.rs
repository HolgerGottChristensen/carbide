use wgpu::BindGroup;

pub struct BindGroupExtended {
    pub bind_group: BindGroup,
    pub width: u32,
    pub height: u32,
}


use carbide_core::image::DynamicImage;
use wgpu::{Device, Queue};

use carbide_core::mesh::pre_multiply::PreMultiply;

use crate::texture;
use crate::texture::Texture;

/// A loaded wgpu texture and it's width/height
pub struct Image {
    /// The immutable image type, represents the data loaded onto the GPU.
    ///
    /// Uses a dynamic format for flexibility on the kinds of images that might be loaded.
    pub texture: Texture,
    /// The width of the image.
    pub width: u32,
    /// The height of the image.
    pub height: u32,
}

impl Image {

    pub fn new_from_dynamic(image: DynamicImage, device: &Device, queue: &Queue) -> Self {
        let rgba_logo_image = image.pre_multiplied().to_rgba8();

        // Create the GPU texture and upload the image data.
        let (width, height) = rgba_logo_image.dimensions();

        let texture = texture::Texture::from_image(
            device,
            queue,
            &rgba_logo_image,
            Option::from("carbide_loaded_image"),
        )
        .unwrap();

        Image {
            texture,
            width,
            height,
        }
    }
}
