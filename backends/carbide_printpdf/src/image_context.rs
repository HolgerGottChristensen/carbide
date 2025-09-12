use std::cell::RefCell;
use std::collections::HashMap;
use carbide_core::draw::ImageId;
use carbide_core::draw::{ImageContext, Texture, TextureFormat};
use printpdf::Image as PdfImage;
use carbide_core::environment::Environment;
use carbide_core::image::{DynamicImage, RgbaImage};

thread_local! {
    pub(crate) static IMAGES: RefCell<HashMap<ImageId, (PdfImage, u32, u32)>> = RefCell::new(HashMap::new());
}

pub struct PDFImageContext;

impl ImageContext for PDFImageContext {
    fn texture_exist(&self, id: &ImageId, env: &mut Environment) -> bool {
        IMAGES.with(|images| {
            images.borrow().contains_key(id)
        })
    }

    fn texture_dimensions(&self, id: &ImageId, env: &mut Environment) -> Option<(u32, u32)> {
        IMAGES.with(|images| {
            let borrow = images.borrow();
            let (_, width, height) = borrow.get(id)?;

            Some((*width, *height))
        })
    }

    fn update_texture(&mut self, id: ImageId, texture: Texture, env: &mut Environment) -> bool {
        IMAGES.with(|images| {
            let width = texture.width;
            let height = texture.height;

            let dynamic_image = match texture.format {
                TextureFormat::RGBA8 => RgbaImage::from_raw(texture.width, texture.height, texture.data.to_vec()).unwrap(),
                TextureFormat::BGRA8 => todo!(),
            };

            // Image with alpha support waiting for merge and release of: https://github.com/fschutt/printpdf/pull/158
            //let pdf_image = PdfImage::from_dynamic_image(&DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(dynamic_image).to_rgb8()));

            //images.borrow_mut().insert(id, (pdf_image, width, height));

            todo!();
            true
        })

    }
}