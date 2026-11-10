use std::fmt::Debug;
use dyn_clone::DynClone;
use carbide::draw::Scalar;
use carbide::environment::Environment;
use crate::DataColor;
use crate::dataset::datapoint::DataPoint;
use crate::dataset::dataset_options::DataSetOptions;
use crate::dataset::datavalue::DataValue;
use crate::element::Stepped;
use crate::scale::ScaleId;
use crate::sequence::{Sequence, SequenceItem};

/// One series: a sequence of data points, with options for how it is drawn.
///
/// Every `Sequence` of data points is a dataset, so `Vec<Position>`, `Vec<Scalar>` (each value
/// at its index) and `LocalState<Vec<(Scalar, Scalar)>>` all are. `OptionsDataset`, which
/// `DatasetExt` wraps a dataset in, implements it directly to override `options`.
///
/// The point types are type parameters rather than associated types: a type could be a sequence
/// of several kinds of point, so the blanket implementation can't pick one.
///
/// Datasets of different types can be mixed in a tuple, or, boxed with `DatasetExt::boxed`, in
/// a `Vec` or a state of a `Vec`. The point types are part of the boxed type,
/// `Box<dyn DataSet<X, Y, Z>>`, so every dataset mixed this way has the same point types.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a dataset of points with `{X}`, `{Y}` and `{Z}` coordinates",
    label = "not a dataset",
    note = "a dataset is a sequence of data points: a `Vec`, array or other sequence of `Position`, `(x, y)` or `(x, y, z)` points, or of plain `Scalar` y values at their index",
    note = "sequences are tuples of up to 12 items; collections (`Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>`, `&'static [_]`, or any type implementing `Collection`); and states holding a collection: `LocalState`, `GlobalState`, `ValueState`, `FieldState`, mapped states, or any read state boxed with `.as_dyn_read()`",
)]
pub trait DataSet<X: DataValue, Y: DataValue, Z: DataValue>: DynClone + Debug + 'static {
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=X, Y=Y, Z=Z>));

    /// The smallest x, y and z of the dataset's points, or `None` when it has no points.
    fn min(&self) -> Option<(X, Y, Z)> {
        extreme(self, Extreme::Min)
    }

    /// The largest x, y and z of the dataset's points, or `None` when it has no points.
    fn max(&self) -> Option<(X, Y, Z)> {
        extreme(self, Extreme::Max)
    }

    fn options(&self, _env: &mut Environment) -> DataSetOptions {
        DataSetOptions {
            color: DataColor::Inherit,
            stepped: Stepped::None,
            x_scale: ScaleId::X,
            y_scale: ScaleId::Y,
        }
    }
}

dyn_clone::clone_trait_object!(<X, Y, Z> DataSet<X, Y, Z> where X: DataValue, Y: DataValue, Z: DataValue);

/// A boxed dataset is a dataset, so a `Vec<Box<dyn DataSet<X, Y, Z>>>` is a sequence of datasets
/// of mixed types, all with points of the same types.
impl<X: DataValue, Y: DataValue, Z: DataValue> DataSet<X, Y, Z> for Box<dyn DataSet<X, Y, Z>> {
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=X, Y=Y, Z=Z>)) {
        (**self).points(f);
    }

    fn min(&self) -> Option<(X, Y, Z)> {
        (**self).min()
    }

    fn max(&self) -> Option<(X, Y, Z)> {
        (**self).max()
    }

    fn options(&self, env: &mut Environment) -> DataSetOptions {
        (**self).options(env)
    }
}

impl<X: DataValue, Y: DataValue, Z: DataValue, S: Sequence<dyn DataPoint<X=X, Y=Y, Z=Z>>> DataSet<X, Y, Z> for S {
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=X, Y=Y, Z=Z>)) {
        // The sequence hands out `'static` trait objects; `f` takes them for any lifetime.
        self.foreach(&mut |index, point| f(index, point));
    }
}

/// A point is an item of a sequence of points.
///
/// The `+ 'static` in `visit` matches the `&T` in `SequenceItem<T>`: as a type argument, the
/// trait object is `'static`, while a bare `&dyn DataPoint<…>` would only live as long as the
/// reference.
impl<X: DataValue, Y: DataValue, Z: DataValue, P: DataPoint<X=X, Y=Y, Z=Z> + Clone + Debug + 'static> SequenceItem<dyn DataPoint<X=X, Y=Y, Z=Z>> for P {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn DataPoint<X=X, Y=Y, Z=Z> + 'static))) {
        f(index, self);
    }
}

/// A plain y value is the point `(index, value)`.
impl SequenceItem<dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>> for Scalar {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar> + 'static))) {
        f(index, &(index as Scalar, *self));
    }
}

/// A dataset is an item of a sequence of datasets.
impl<X: DataValue, Y: DataValue, Z: DataValue, D: DataSet<X, Y, Z> + Clone> SequenceItem<dyn DataSet<X, Y, Z>> for D {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn DataSet<X, Y, Z> + 'static))) {
        f(index, self);
    }
}

#[derive(Copy, Clone)]
enum Extreme {
    Min,
    Max,
}

/// The component-wise smallest or largest x, y and z of the dataset's points, or `None` when it
/// has no points.
fn extreme<X: DataValue, Y: DataValue, Z: DataValue, D: DataSet<X, Y, Z> + ?Sized>(dataset: &D, extreme: Extreme) -> Option<(X, Y, Z)> {
    fn pick<T: PartialOrd>(current: T, other: T, extreme: Extreme) -> T {
        let replace = match extreme {
            Extreme::Min => other < current,
            Extreme::Max => other > current,
        };

        if replace { other } else { current }
    }

    let mut result: Option<(X, Y, Z)> = None;

    dataset.points(&mut |_index, point| {
        let (x, y, z) = (point.x(), point.y(), point.z());

        result = Some(match result {
            Some((rx, ry, rz)) => (pick(rx, x, extreme), pick(ry, y, extreme), pick(rz, z, extreme)),
            None => (x, y, z),
        });
    });

    result
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
