use std::path::Path;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};

use wgpu::{Device, Queue};

use carbide_core::mesh;
use carbide_core::widget::ImageInformation;

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

impl mesh::mesh::ImageDimensions for Image {
    fn dimensions(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}

impl Image {
    pub fn image_information(&self) -> ImageInformation {
        ImageInformation {
            width: self.width,
            height: self.height,
        }
    }

    pub fn new<P>(path: P, device: &Device, queue: &Queue) -> Self
        where
            P: AsRef<Path>,
    {
        let rgba_logo_image = image::open(path).expect("Couldn't load logo").pre_multiplied().to_rgba8();

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

    pub fn new_from_dynamic(image: image::DynamicImage, device: &Device, queue: &Queue) -> Self
    {

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

trait PreMultiply {
    fn pre_multiplied(&self) -> DynamicImage;
}

impl PreMultiply for DynamicImage {
    fn pre_multiplied(&self) -> DynamicImage {
        let mut premultiplied = self.clone();
        for (x, y, rgba) in self.pixels() {
            let red = rgba.0[0] as f64 / 255.0;
            let green = rgba.0[1] as f64 / 255.0;
            let blue = rgba.0[2] as f64 / 255.0;
            let alpha = rgba.0[3] as f64 / 255.0;

            let new_pixel = Rgba([(red * alpha * 255.0) as u8, (green * alpha * 255.0) as u8, (blue * alpha * 255.0) as u8, (alpha * 255.0) as u8]);
            premultiplied.put_pixel(x, y, new_pixel);
        }
        premultiplied
    }
}