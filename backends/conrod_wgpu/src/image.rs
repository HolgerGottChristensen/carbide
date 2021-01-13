use crate::texture::Texture;
use conrod_core::mesh;
use wgpu::{Device, Queue};
use crate::{texture, DEFAULT_IMAGE_TEX_FORMAT};
use std::path::Path;
use conrod_core::widget::types::image_information::ImageInformation;

/// A loaded wgpu texture and it's width/height
pub struct Image {
    /// The immutable image type, represents the data loaded onto the GPU.
    ///
    /// Uses a dynamic format for flexibility on the kinds of images that might be loaded.
    pub texture: Texture,
    /// The format of the `texture`.
    pub texture_format: wgpu::TextureFormat,
    /// The width of the image.
    pub width: u32,
    /// The height of the image.
    pub height: u32,
}

impl mesh::mesh::ImageDimensions for Image {
    fn dimensions(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}


impl Image {
    pub fn texture_component_type(&self) -> wgpu::TextureComponentType {
        self.texture_format.into()
    }

    pub fn image_information(&self) -> ImageInformation {
        ImageInformation {
            width: self.width,
            height: self.height
        }
    }

    pub fn new<P>(logo_path: P,  device: &Device, queue: &Queue) -> Self
    where P: AsRef<Path> {

        let rgba_logo_image = image::open(logo_path)
            .expect("Couldn't load logo")
            .to_rgba();

        // Create the GPU texture and upload the image data.
        let (width, height) = rgba_logo_image.dimensions();

        let texture = texture::Texture::from_image(device, queue, &rgba_logo_image, Option::from("conrod_loaded_image")).unwrap();

        Image {
            texture,
            texture_format: DEFAULT_IMAGE_TEX_FORMAT,
            width,
            height
        }
    }
}