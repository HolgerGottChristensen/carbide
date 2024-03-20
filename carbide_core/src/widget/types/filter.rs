use std::fmt::{Display, Formatter};
use std::ops::Div;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::utils::gaussian;

/// Filter struct containing a matrix of filter weights that can be applied to change the rendering
/// of a sub tree. For more information on image filters look at:
/// https://en.wikipedia.org/wiki/Kernel_(image_processing)
#[derive(Clone, Debug)]
pub struct ImageFilter {
    pub filter: Vec<ImageFilterValue>,
}

impl Display for ImageFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut min_x = None;
        let mut max_x = None;
        let mut min_y = None;
        let mut max_y = None;

        for item in &self.filter {
            if let Some(min_x) = &mut min_x {
                if item.offset_x < *min_x {
                    *min_x = item.offset_x;
                }
            } else {
                min_x = Some(item.offset_x);
            }

            if let Some(min_y) = &mut min_y {
                if item.offset_y < *min_y {
                    *min_y = item.offset_y;
                }
            } else {
                min_y = Some(item.offset_y);
            }

            if let Some(max_x) = &mut max_x {
                if item.offset_x > *max_x {
                    *max_x = item.offset_x;
                }
            } else {
                max_x = Some(item.offset_x);
            }

            if let Some(max_y) = &mut max_y {
                if item.offset_y > *max_y {
                    *max_y = item.offset_y;
                }
            } else {
                max_y = Some(item.offset_y);
            }
        }

        writeln!(f, "{:?}-{:?}:{:?}-{:?}", min_x, max_x, min_y, max_y)?;

        let mut res = String::new();

        if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (min_x, max_x, min_y, max_y) {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let mut found = false;
                    for item in &self.filter {
                        if item.offset_x == x && item.offset_y == y {
                            res.push_str(&format!("{:.4} ", item.weight));
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        res.push_str("x.xxxx");
                    }
                }

                res.push('\n');
            }

            writeln!(f, "{}", res)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct FilterId(u32);

impl FilterId {
    pub fn next() -> FilterId {
        static FILTER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
        FilterId(FILTER_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl ImageFilter {
    /// Applying this filter will sharpen the image
    pub fn sharpen() -> ImageFilter {
        ImageFilter {
            filter: vec![
                ImageFilterValue::new(-1, 0, -1.0),
                ImageFilterValue::new(1, 0, -1.0),
                ImageFilterValue::new(0, -1, -1.0),
                ImageFilterValue::new(0, 1, -1.0),
                ImageFilterValue::new(0, 0, 5.0),
            ],
        }
    }

    pub fn gaussian_blur_1d(sigma: f32) -> ImageFilter {
        assert!(sigma > 0.0);
        let mut entries = vec![];
        let radius = (3.0 * sigma).round() as i32;

        for x in -radius..=radius {
            entries.push(ImageFilterValue::new(
                x,
                0,
                gaussian(sigma as f64, x as f64) as f32,
            ))
        }

        let mut filter = ImageFilter { filter: entries };

        filter.normalize();
        filter
    }

    // http://demofox.org/gauss.html
    // https://lisyarus.github.io/blog/graphics/2023/02/24/blur-coefficients-generator.html
    // https://drafts.fxtf.org/filter-effects/#feGaussianBlurElement
    pub fn gaussian_blur(sigma: f64) -> ImageFilter {
        let radius = (3.0 * sigma).round() as i32;
        let mut res = vec![];

        for x in -radius..=radius {
            for y in -radius..=radius {
                res.push(ImageFilterValue::new(x, y, f64::exp(-f64::div(f64::powf(x as f64, 2.0) + f64::powf(y as f64, 2.0), f64::powf(sigma, 2.0))) as f32));
            }
        }

        let mut filter = ImageFilter {
            filter: res,
        };
        filter.normalize();

        //println!("{}", filter);

        filter
    }

    pub fn mean_blur(radius: i32) -> ImageFilter {
        let width = radius + 1 + radius;
        let factor = 1.0 / (width * width) as f32;
        let mut res = vec![];

        for x in -radius..=radius {
            for y in -radius..=radius {
                res.push(ImageFilterValue::new(x, y, factor))
            }
        }

        ImageFilter {
            filter: res,
        }
    }

    /// The sobel filter is used to detect edges: https://en.wikipedia.org/wiki/Sobel_operator
    /// Another edge detection filter is [Self::prewit()]
    pub fn sobel() -> ImageFilter {
        let mut entries = ImageFilter::sobel_x().filter;
        entries.extend(ImageFilter::sobel_y().filter);

        ImageFilter { filter: entries }
    }

    /// The x component of the sobel filter
    pub fn sobel_x() -> ImageFilter {
        ImageFilter {
            filter: vec![
                ImageFilterValue::new(-1, -1, 1.0),
                ImageFilterValue::new(-1, 0, 2.0),
                ImageFilterValue::new(-1, 1, 1.0),
                ImageFilterValue::new(1, -1, -1.0),
                ImageFilterValue::new(1, 0, -2.0),
                ImageFilterValue::new(1, 1, -1.0),
            ],
        }
    }

    /// The y component of the sobel filter
    pub fn sobel_y() -> ImageFilter {
        ImageFilter {
            filter: vec![
                ImageFilterValue::new(-1, -1, 1.0),
                ImageFilterValue::new(0, -1, 2.0),
                ImageFilterValue::new(1, -1, 1.0),
                ImageFilterValue::new(-1, 1, -1.0),
                ImageFilterValue::new(0, 1, -2.0),
                ImageFilterValue::new(1, 1, -1.0),
            ],
        }
    }

    /// The prewit filter can be used to detect edges in an image.
    /// For more information on the filter look at: https://en.wikipedia.org/wiki/Prewitt_operator
    /// Another edge detection filter is [Self::sobel()]
    pub fn prewit() -> ImageFilter {
        let mut entries = ImageFilter::prewit_x().filter;
        entries.extend(ImageFilter::prewit_y().filter);

        ImageFilter { filter: entries }
    }

    /// The x component of the prewit filter
    pub fn prewit_x() -> ImageFilter {
        ImageFilter {
            filter: vec![
                ImageFilterValue::new(-1, -1, 1.0),
                ImageFilterValue::new(-1, 0, 1.0),
                ImageFilterValue::new(-1, 1, 1.0),
                ImageFilterValue::new(1, -1, -1.0),
                ImageFilterValue::new(1, 0, -1.0),
                ImageFilterValue::new(1, 1, -1.0),
            ],
        }
    }

    /// The y component of the prewit filter
    pub fn prewit_y() -> ImageFilter {
        ImageFilter {
            filter: vec![
                ImageFilterValue::new(-1, -1, 1.0),
                ImageFilterValue::new(0, -1, 1.0),
                ImageFilterValue::new(1, -1, 1.0),
                ImageFilterValue::new(-1, 1, -1.0),
                ImageFilterValue::new(0, 1, -1.0),
                ImageFilterValue::new(1, 1, -1.0),
            ],
        }
    }

    /// Normalize a filter by calculating the sum and dividing all filters with that.
    /// This will make the sum of all the filter values 1.0.
    pub fn normalize(&mut self) {
        let mut acc = 0.0;
        for val in &self.filter {
            acc += val.weight;
        }

        for val in self.filter.iter_mut() {
            val.weight /= acc;
        }
    }

    /// Flip the x and y axis of the filter. This is also the same as changing a convolution to a
    /// cross-correlation or changing a cross-correlation to a convolution.
    pub fn flip(&mut self) {
        for val in self.filter.iter_mut() {
            let temp = val.offset_x;
            val.offset_x = val.offset_y;
            val.offset_y = temp;
        }
    }

    /// Convenience function to flip filter and return the resulting filter using [Self::flip()]
    pub fn flipped(mut self) -> ImageFilter {
        self.flip();
        self
    }

    pub fn offset(mut self, x: i32, y: i32) -> ImageFilter {
        for f in &mut self.filter {
            f.offset_x += x;
            f.offset_y += y;
        }

        self
    }

    /// Calculate the radius of the filter on the x axis
    pub fn radius_x(&self) -> u32 {
        let mut largest = 0;
        for val in &self.filter {
            if val.offset_x.abs() > largest {
                largest = val.offset_x.abs();
            }
        }

        largest as u32
    }

    /// Calculate the radius of the filter on the y axis
    pub fn radius_y(&self) -> u32 {
        let mut largest = 0;
        for val in &self.filter {
            if val.offset_y.abs() > largest {
                largest = val.offset_y.abs();
            }
        }

        largest as u32
    }
}

/// A single filter value containing the position in the matrix and a weight.
#[derive(Clone, Debug)]
pub struct ImageFilterValue {
    pub offset_x: i32,
    pub offset_y: i32,
    pub weight: f32,
}

impl ImageFilterValue {
    pub fn new(x: i32, y: i32, weight: f32) -> ImageFilterValue {
        ImageFilterValue {
            offset_x: x,
            offset_y: y,
            weight,
        }
    }
}
