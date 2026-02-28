use carbide::draw::Scalar;

pub trait SizeCollection {
    fn iter_range(&self, from: Scalar, to: Scalar) -> impl Iterator<Item=(Scalar, Scalar, u32)>;
}

impl SizeCollection for Vec<f64> {
    fn iter_range(&self, from: Scalar, to: Scalar) -> impl Iterator<Item=(Scalar, Scalar, u32)> {
        VecRangeIter::new(self, from, to)
    }
}

struct VecRangeIter<'a> {
    min: f64,
    max: f64,
    current: f64,
    current_index: usize,
    collection: &'a Vec<f64>
}

impl VecRangeIter<'_> {
    fn new(vec: &Vec<f64>, from: f64, to: f64) -> VecRangeIter {
        let mut current = 0.0;
        let mut current_index = 0;

        if from >= 0.0 {
            for (index, item) in vec.iter().enumerate() {
                if current + item >= from {
                    break;
                }

                current += item;
                current_index = index + 1;
            }
        }

        VecRangeIter {
            min: from,
            max: to,
            current,
            current_index,
            collection: vec,
        }
    }
}

impl Iterator for VecRangeIter<'_> {
    type Item = (Scalar, Scalar, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.max {
            None
        } else if self.current_index >= self.collection.len() {
            None
        } else {
            let item = self.collection[self.current_index];
            let res = Some((self.current, item, self.current_index as u32));;

            self.current_index += 1;
            self.current += item;
            res
        }
    }
}