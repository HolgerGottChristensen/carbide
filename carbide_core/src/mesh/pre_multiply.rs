use image::{DynamicImage, GenericImage, GenericImageView, Rgba};

pub trait PreMultiply {
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