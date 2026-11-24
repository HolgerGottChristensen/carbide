use std::fmt::Debug;
use dyn_clone::DynClone;
use carbide::environment::Environment;
use crate::DataColor;
use crate::DataPoint;
use crate::DataSetOptions;
use crate::element::{Fill, Interpolation, SpanGaps};
use crate::scale::ScaleId;
use crate::Sequence;

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
    /// Method providing internal iteration of all the data points of the dataset
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint));

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

#[cfg(test)]
mod tests {
    use carbide::draw::Scalar;
    use crate::{DataSet, Value};

    /// A missing value in a dataset of `Option`s keeps its index, as its x, but has no y.
    #[test]
    fn missing_values_keep_their_index() {
        let options: Vec<Option<Scalar>> = vec![Some(3.0), None, Some(-1.0)];

        let mut points = vec![];
        options.points(&mut |index, point| points.push((index, point.x().and_then(Value::number), point.y().and_then(Value::number))));
        assert_eq!(points, vec![(0, Some(0.0), Some(3.0)), (1, Some(1.0), None), (2, Some(2.0), Some(-1.0))]);
    }
}
