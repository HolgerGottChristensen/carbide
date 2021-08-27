#[derive(Clone, Debug)]
pub struct ImageFilter {
    pub filter: Vec<ImageFilterValue>,
}

impl ImageFilter {
    pub fn normalize(&mut self) {
        let mut acc = 0.0;
        for val in &self.filter {
            acc += val.weight;
        }

        for val in self.filter.iter_mut() {
            val.weight /= acc;
        }
    }

    pub fn flip(&mut self) {
        for val in self.filter.iter_mut() {
            let temp = val.offset_x;
            val.offset_x = val.offset_y;
            val.offset_y = temp;
        }
    }

    pub fn flipped(mut self) -> ImageFilter {
        self.flip();
        self
    }

    pub fn radius_x(&self) -> u32 {
        let mut largest = 0;
        for val in &self.filter {
            if val.offset_x.abs() > largest {
                largest = val.offset_x.abs();
            }
        }

        largest as u32
    }

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

