#[derive(Clone, Debug)]
pub struct ImageFilter {
    pub filter: Vec<ImageFilterValue>,
}

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

