use std::fmt::Debug;
use dyn_clone::DynClone;
use carbide::draw::Scalar;
use carbide::environment::Environment;
use crate::DataColor;
use crate::dataset::datapoint::DataPoint;
use crate::dataset::dataset_options::DataSetOptions;
use crate::element::{Fill, Interpolation, SpanGaps};
use crate::scale::ScaleId;
use crate::Value;
use crate::sequence::{Sequence, SequenceItem};

/// One series: a sequence of data points, with options for how it is drawn.
///
/// Every `Sequence` of data points is a dataset, so `Vec<Position>`, `Vec<Scalar>` (each value
/// at its index), `Vec<Option<Scalar>>` (with gaps where a value is `None`) and
/// `LocalState<Vec<(Scalar, Scalar)>>` all are. `OptionsDataset`, which `DatasetExt` wraps a
/// dataset in, implements it directly to override `options`.
///
/// Points are read as numbers (see `DataPoint`), whatever their coordinate types, so datasets of
/// any point types share a controller: a `Vec<Scalar>` and a `Vec<Option<Scalar>>` can sit in
/// one tuple. Datasets of different types can also be mixed in a `Vec` or a state of a `Vec`,
/// boxed with `DatasetExt::boxed`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a dataset",
    label = "not a dataset",
    note = "a dataset is a sequence of data points: a `Vec`, array or other sequence of `Position`, `(x, y)` or `(x, y, z)` points, or of plain `Scalar` or `Option<Scalar>` y values at their index",
    note = "sequences are tuples of up to 12 items; collections (`Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>`, `&'static [_]`, or any type implementing `Collection`); and states holding a collection: `LocalState`, `GlobalState`, `ValueState`, `FieldState`, mapped states, or any read state boxed with `.as_dyn_read()`",
)]
pub trait DataSet: DynClone + Debug + 'static {
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint));

    /// The smallest x and y of the dataset's points, leaving out missing ones, or `None` when it
    /// has none.
    fn min(&self) -> Option<(Scalar, Scalar)> {
        extreme(self, Scalar::min)
    }

    /// The largest x and y of the dataset's points, leaving out missing ones, or `None` when it
    /// has none.
    fn max(&self) -> Option<(Scalar, Scalar)> {
        extreme(self, Scalar::max)
    }

    /// Syncs the states in the dataset with the environment, and returns whether anything may
    /// have changed. A dataset that is a sequence of points syncs as a sequence: the state
    /// holding the points, and any states inside it.
    fn sync(&mut self, _env: &mut Environment) -> bool {
        false
    }

    /// How the dataset is drawn. Options held in states are read as they were last synced.
    fn options(&self) -> DataSetOptions {
        DataSetOptions {
            color: DataColor::ByDataSet,
            interpolation: Interpolation::Linear,
            fill: Fill::None,
            span_gaps: SpanGaps::Never,
            x_scale: ScaleId::X,
            y_scale: ScaleId::Y,
            stack: None,
            label: None,
            visible: true,
        }
    }

    /// Shows or hides the dataset, as a click on its legend entry does, and returns whether it
    /// could: a dataset with options (`DatasetExt`) writes its visibility state, while a dataset
    /// without one, the default, can't be hidden and returns `false`. It takes `&self`, as a
    /// visibility state is shared with its clones, so writing through one writes them all.
    fn set_visible(&self, _visible: bool) -> bool {
        false
    }
}

dyn_clone::clone_trait_object!(DataSet);

/// A boxed dataset is a dataset, so a `Vec<Box<dyn DataSet>>` is a sequence of datasets of
/// mixed types.
impl DataSet for Box<dyn DataSet> {
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint)) {
        (**self).points(f);
    }

    fn min(&self) -> Option<(Scalar, Scalar)> {
        (**self).min()
    }

    fn max(&self) -> Option<(Scalar, Scalar)> {
        (**self).max()
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        (**self).sync(env)
    }

    fn options(&self) -> DataSetOptions {
        (**self).options()
    }

    fn set_visible(&self, visible: bool) -> bool {
        (**self).set_visible(visible)
    }
}

impl<S: Sequence<dyn DataPoint>> DataSet for S {
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint)) {
        // The sequence hands out `'static` trait objects; `f` takes them for any lifetime.
        self.foreach(&mut |index, point| f(index, point));
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        Sequence::<dyn DataPoint>::sync(self, env)
    }
}

/// A point is an item of a sequence of points.
///
/// The `+ 'static` in `visit` matches the `&T` in `SequenceItem<T>`: as a type argument, the
/// trait object is `'static`, while a bare `&dyn DataPoint` would only live as long as the
/// reference.
impl<P: DataPoint + Clone + Debug + 'static> SequenceItem<dyn DataPoint> for P {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn DataPoint + 'static))) {
        f(index, self);
    }
}

/// A plain y value is the point `(index, value)`.
impl SequenceItem<dyn DataPoint> for Scalar {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn DataPoint + 'static))) {
        f(index, &(index as Scalar, *self));
    }
}

/// An optional y value is the point `(index, value)` whose y may be missing, so
/// `vec![Some(3.0), None, Some(5.0)]` is a dataset with a gap in the middle.
impl SequenceItem<dyn DataPoint> for Option<Scalar> {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn DataPoint + 'static))) {
        f(index, &(index as Scalar, *self));
    }
}

/// A dataset is an item of a sequence of datasets.
impl<D: DataSet + Clone> SequenceItem<dyn DataSet> for D {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn DataSet + 'static))) {
        f(index, self);
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        DataSet::sync(self, env)
    }
}

/// The x and y of the dataset's points that aren't missing, each combined across the points with
/// `pick` (`Scalar::min` or `Scalar::max`), or `None` when it has none. Only numbers and times
/// count: a category's name has no number until a scale places it (see `Coordinates`).
fn extreme<D: DataSet + ?Sized>(dataset: &D, pick: fn(Scalar, Scalar) -> Scalar) -> Option<(Scalar, Scalar)> {
    let mut result: Option<(Scalar, Scalar)> = None;

    dataset.points(&mut |_index, point| {
        let (Some(x), Some(y)) = (point.x().and_then(Value::number), point.y().and_then(Value::number)) else {
            return;
        };

        result = Some(match result {
            Some((rx, ry)) => (pick(rx, x), pick(ry, y)),
            None => (x, y),
        });
    });

    result
}

#[cfg(test)]
mod tests {
    use carbide::draw::Scalar;
    use crate::{DataSet, Value};

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
        assert_eq!(values.min(), Some((0.0, -1.0)));
        assert_eq!(values.max(), Some((2.0, 7.0)));
    }

    /// `None` in a dataset of `Option`s, and NaN in a plain dataset, are missing points, which a
    /// dataset's bounds leave out. A dataset of only missing points has no bounds.
    #[test]
    fn missing_points_are_left_out_of_the_bounds() {
        let options: Vec<Option<Scalar>> = vec![Some(3.0), None, Some(-1.0)];
        assert_eq!(options.min(), Some((0.0, -1.0)));
        assert_eq!(options.max(), Some((2.0, 3.0)));

        let mut points = vec![];
        options.points(&mut |index, point| points.push((index, point.x().and_then(Value::number), point.y().and_then(Value::number))));
        assert_eq!(points, vec![(0, Some(0.0), Some(3.0)), (1, Some(1.0), None), (2, Some(2.0), Some(-1.0))], "a missing value keeps its index, but has no y");

        let nan: Vec<Scalar> = vec![5.0, Scalar::NAN, 7.0];
        assert_eq!(nan.max(), Some((2.0, 7.0)), "a NaN is missing too, and doesn't poison the bounds");

        let optional_points: Vec<Option<(Scalar, Scalar)>> = vec![None, Some((4.0, 2.0)), None];
        assert_eq!(optional_points.min(), Some((4.0, 2.0)));

        let all_missing: Vec<Option<Scalar>> = vec![None, None];
        assert_eq!(all_missing.min(), None);
    }
}
