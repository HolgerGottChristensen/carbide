

/// Filter struct containing a matrix of filter weights that can be applied to change the rendering
/// of a sub tree. For more information on image filters look at:
/// https://en.wikipedia.org/wiki/Kernel_(image_processing)
#[derive(Clone, Debug)]
pub struct ImageFilter {
    pub filter: Vec<ImageFilterValue>,
}

#[derive(Clone, Debug, Copy)]
pub struct FilterId(u32);

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
            ]
        }
    }

    /// The sobel filter is used to detect edges: https://en.wikipedia.org/wiki/Sobel_operator
    /// Another edge detection filter is [Self::prewit()]
    pub fn sobel() -> ImageFilter {
        let mut entries = ImageFilter::sobel_x().filter;
        entries.extend(ImageFilter::sobel_y().filter);

        ImageFilter {
            filter: entries
        }
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
            ]
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
            ]
        }
    }

    /// The prewit filter can be used to detect edges in an image.
    /// For more information on the filter look at: https://en.wikipedia.org/wiki/Prewitt_operator
    /// Another edge detection filter is [Self::sobel()]
    pub fn prewit() -> ImageFilter {
        let mut entries = ImageFilter::prewit_x().filter;
        entries.extend(ImageFilter::prewit_y().filter);

        ImageFilter {
            filter: entries
        }
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
            ]
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
            ]
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

