use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::render::matrix::{One, Zero};
use crate::DataColor;
use crate::dataset::datapoint::DataPoint;
use crate::dataset::dataset_options::DataSetOptions;
use crate::dataset::datavalue::DataValue;
use crate::element::Stepped;

pub trait DataSet: Debug + 'static {
    type X: DataValue;
    type Y: DataValue;
    type Z: DataValue;

    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=Self::X, Y=Self::Y, Z=Self::Z>));

    fn min(&self) -> (Self::X, Self::Y, Self::Z);
    fn max(&self) -> (Self::X, Self::Y, Self::Z);

    fn options(&self, env: &mut Environment) -> DataSetOptions {
        DataSetOptions {
            color: DataColor::Inherit,
            stepped: Stepped::None,
        }
    }
}

impl<X: DataValue + PartialOrd, Y: DataValue + PartialOrd, Z: DataValue + PartialOrd, T: DataPoint<X=X, Y=Y, Z=Z> + PartialOrd + Clone + Debug + 'static> DataSet for Vec<T> {
    type X = X;
    type Y = Y;
    type Z = Z;

    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=Self::X, Y=Self::Y, Z=Self::Z>)) {
        for (index, point) in self.iter().enumerate() {
            f(index, point);
        }
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
    type X = Scalar;
    type Y = Scalar;
    type Z = Scalar;

    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=Self::X, Y=Self::Y, Z=Self::Z>)) {
        for (index, point) in self.iter().enumerate() {
            f(index, &(index as Scalar, *point));
        }
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

            ((self.len() - 1) as Scalar, max, Scalar::zero())
        } else {
            ((self.len() - 1) as Scalar, Scalar::one(), Scalar::zero())
        }
    }
}