use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::environment::{Environment};
use carbide::math::Zero;
use crate::DataColor;
use crate::dataset::datapoint::DataPoint;
use crate::dataset::dataset_options::DataSetOptions;
use crate::dataset::datavalue::DataValue;
use crate::element::Stepped;
use crate::scale::ScaleId;

pub trait DataSet: Debug + 'static {
    type X: DataValue;
    type Y: DataValue;
    type Z: DataValue;

    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=Self::X, Y=Self::Y, Z=Self::Z>));

    /// The smallest x, y and z of the dataset's points, or `None` when it has no points.
    fn min(&self) -> Option<(Self::X, Self::Y, Self::Z)>;
    /// The largest x, y and z of the dataset's points, or `None` when it has no points.
    fn max(&self) -> Option<(Self::X, Self::Y, Self::Z)>;

    fn options(&self, _env: &mut Environment) -> DataSetOptions {
        DataSetOptions {
            color: DataColor::Inherit,
            stepped: Stepped::None,
            x_scale: ScaleId::X,
            y_scale: ScaleId::Y,
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

    fn min(&self) -> Option<(X, Y, Z)> {
        let mut iter = self.iter();
        let item = iter.next()?;

        let mut min_x = item.x();
        let mut min_y = item.y();
        let mut min_z = item.z();

        for item in iter {
            min_x = if PartialOrd::lt(&min_x, &item.x()) { min_x } else { item.x() };
            min_y = if PartialOrd::lt(&min_y, &item.y()) { min_y } else { item.y() };
            min_z = if PartialOrd::lt(&min_z, &item.z()) { min_z } else { item.z() };
        }

        Some((min_x, min_y, min_z))
    }

    fn max(&self) -> Option<(X, Y, Z)> {
        let mut iter = self.iter();
        let item = iter.next()?;

        let mut max_x = item.x();
        let mut max_y = item.y();
        let mut max_z = item.z();

        for item in iter {
            max_x = if PartialOrd::gt(&max_x, &item.x()) { max_x } else { item.x() };
            max_y = if PartialOrd::gt(&max_y, &item.y()) { max_y } else { item.y() };
            max_z = if PartialOrd::gt(&max_z, &item.z()) { max_z } else { item.z() };
        }

        Some((max_x, max_y, max_z))
    }
}

/// A plain list of y values; each value's x is its index.
impl DataSet for Vec<Scalar> {
    type X = Scalar;
    type Y = Scalar;
    type Z = Scalar;

    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=Self::X, Y=Self::Y, Z=Self::Z>)) {
        for (index, point) in self.iter().enumerate() {
            f(index, &(index as Scalar, *point));
        }
    }

    fn min(&self) -> Option<(Scalar, Scalar, Scalar)> {
        let mut iter = self.iter();
        let mut min = *iter.next()?;

        for item in iter {
            min = if PartialOrd::lt(&min, item) { min } else { *item };
        }

        Some((0.0, min, Scalar::zero()))
    }

    fn max(&self) -> Option<(Scalar, Scalar, Scalar)> {
        let mut iter = self.iter();
        let mut max = *iter.next()?;

        for item in iter {
            max = if PartialOrd::gt(&max, item) { max } else { *item };
        }

        // Not empty, so `len() - 1` can't underflow.
        Some(((self.len() - 1) as Scalar, max, Scalar::zero()))
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::Scalar;
    use crate::DataSet;

    /// Empty datasets have no bounds. `Vec<Scalar>::max` used to compute `len() - 1` for the
    /// last index even when empty, which underflowed and panicked.
    #[test]
    fn empty_datasets_have_no_bounds() {
        let values: Vec<Scalar> = vec![];
        assert_eq!(values.min(), None);
        assert_eq!(values.max(), None);

        let points: Vec<(Scalar, Scalar)> = vec![];
        assert_eq!(points.min(), None);
        assert_eq!(points.max(), None);
    }

    #[test]
    fn value_lists_use_the_index_as_x() {
        let values: Vec<Scalar> = vec![3.0, -1.0, 7.0];
        assert_eq!(values.min(), Some((0.0, -1.0, 0.0)));
        assert_eq!(values.max(), Some((2.0, 7.0, 0.0)));
    }
}
