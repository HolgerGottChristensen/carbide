use std::fmt::Debug;
use std::iter::{Copied, Enumerate, Map, once, Once};
use std::slice::Iter;
use carbide::draw::Scalar;
use carbide::render::matrix::{One, Zero};
use crate::dataset::datapoint::DataPoint;
use crate::dataset::datavalue::DataValue;

pub trait DataSet: Debug + Clone + 'static {
    type Item<'a>: DataPoint;
    type Iter<'a>: Iterator<Item=Self::Item<'a>> where Self: 'a;

    fn points<'a>(&'a self) -> Self::Iter<'a>;

    fn min(&self) -> (<Self::Item<'_> as DataPoint>::X, <Self::Item<'_> as DataPoint>::Y, <Self::Item<'_> as DataPoint>::Z);
    fn max(&self) -> (<Self::Item<'_> as DataPoint>::X, <Self::Item<'_> as DataPoint>::Y, <Self::Item<'_> as DataPoint>::Z);
}

impl<X: DataValue + PartialOrd, Y: DataValue + PartialOrd, Z: DataValue + PartialOrd, T: DataPoint<X=X, Y=Y, Z=Z> + PartialOrd + Clone + Debug + 'static> DataSet for Vec<T> {
    type Item<'a> = &'a T;
    type Iter<'a> = Iter<'a, T> where T: 'a;

    fn points<'a>(&'a self) -> Self::Iter<'a> {
        self.iter()
    }

    fn min(&self) -> (X, Y, Z) {
        let mut iter = self.iter();
        if let Some(item) = iter.next() {
            let mut min_x = item.x();
            let mut min_y = item.y();
            let mut min_z = item.z();

            for item in iter {
                min_x = if PartialOrd::lt(&min_x, &item.x()) { min_x } else { item.x() };
                min_y = if PartialOrd::lt(&min_y, &item.y()) { min_y } else { item.y() };
                min_z = if PartialOrd::lt(&min_z, &item.z()) { min_z } else { item.z() };
            }

            (min_x, min_y, min_z)
        } else {
            (X::zero(), Y::zero(), Z::zero())
        }
    }

    fn max(&self) -> (X, Y, Z) {
        let mut iter = self.iter();
        if let Some(item) = iter.next() {
            let mut max_x = item.x();
            let mut max_y = item.y();
            let mut max_z = item.z();

            for item in iter {
                max_x = if PartialOrd::gt(&max_x, &item.x()) { max_x } else { item.x() };
                max_y = if PartialOrd::gt(&max_y, &item.y()) { max_y } else { item.y() };
                max_z = if PartialOrd::gt(&max_z, &item.z()) { max_z } else { item.z() };
            }

            (max_x, max_y, max_z)
        } else {
            (X::one(), Y::one(), Z::one())
        }
    }
}

impl DataSet for Vec<Scalar> {
    type Item<'a> = (Scalar, Scalar);
    type Iter<'a> = Map<Enumerate<Copied<Iter<'a, Scalar>>>, fn((usize, Scalar))->(Scalar, Scalar)> where Scalar: 'a;

    fn points<'a>(&'a self) -> Self::Iter<'a> {
        self.iter().copied().enumerate().map(|(x, y)| (x as Scalar, y))
    }

    fn min(&self) -> (Scalar, Scalar, Scalar) {
        let mut iter = self.iter();
        if let Some(first) = iter.next() {
            let mut min = *first;
            for item in iter {
                min = if PartialOrd::lt(&min, item) { min } else { *item };
            }

            (0.0, min, Scalar::zero())
        } else {
            (0.0, Scalar::zero(), Scalar::zero())
        }
    }

    fn max(&self) -> (Scalar, Scalar, Scalar) {
        let mut iter = self.iter();
        if let Some(first) = iter.next() {
            let mut max = *first;
            for item in iter {
                max = if PartialOrd::gt(&max, item) { max } else { *item };
            }

            (self.len() as Scalar, max, Scalar::zero())
        } else {
            (self.len() as Scalar, Scalar::one(), Scalar::zero())
        }
    }
}

/*impl<T: DataPoint + Debug + Copy + 'static> DataSet for T  {
    type Item<'a> = T;
    type Iter<'a> = Once<T> where T: 'a;

    fn points<'a>(&'a self) -> Self::Iter<'a> {
        once(*self)
    }

    fn min(&self) -> (T::X, T::Y, T::Z) {
        let point = self.points().next().unwrap();
        (point.x(), point.y(), point.z())
    }

    fn max(&self) -> (T::X, T::Y, T::Z) {
        let point = self.points().next().unwrap();
        (point.x(), point.y(), point.z())
    }
}*/