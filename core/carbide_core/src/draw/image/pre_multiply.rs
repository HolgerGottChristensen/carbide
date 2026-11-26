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

            let new_pixel = Rgba([
                (red * alpha * 255.0) as u8,
                (green * alpha * 255.0) as u8,
                (blue * alpha * 255.0) as u8,
                (alpha * 255.0) as u8,
            ]);
            premultiplied.put_pixel(x, y, new_pixel);
        }
        premultiplied
    }
}

/// Premultiply the alpha of RGBA8 pixels in place: each colour channel times its alpha. The same
/// arithmetic as [`PreMultiply::pre_multiplied`], so the result is identical, but on the bytes
/// directly, with no copy and no per-pixel conversion.
pub fn pre_multiply_rgba8(pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = pixel[3] as f64 / 255.0;
        for channel in &mut pixel[..3] {
            *channel = (*channel as f64 / 255.0 * alpha * 255.0) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, GenericImageView, RgbaImage};
    use super::{pre_multiply_rgba8, PreMultiply};

    /// The in-place version gives exactly what the image version gives, for every colour value
    /// at every alpha.
    #[test]
    fn in_place_matches_the_image_version() {
        let image = RgbaImage::from_fn(256, 256, |x, y| image::Rgba([x as u8, (255 - x) as u8, (x / 2) as u8, y as u8]));
        let expected = DynamicImage::ImageRgba8(image.clone()).pre_multiplied();

        let mut pixels = image.into_raw();
        pre_multiply_rgba8(&mut pixels);

        let expected: Vec<u8> = expected.pixels().flat_map(|(_, _, pixel)| pixel.0).collect();
        assert!(pixels == expected);
    }
}
