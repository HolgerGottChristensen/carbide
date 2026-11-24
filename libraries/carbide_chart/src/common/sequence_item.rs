use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::environment::Environment;
use crate::{DataPoint, DataSet, DatasetController};

/// A value that can be visited as a `T`, at `index` in its sequence. Containers (collections,
/// states holding a collection, tuples) are sequences of anything that is an item, so they are
/// written once for points, datasets and controllers alike.
///
/// The index is passed in so an item can depend on its position: a plain `Scalar` is the point
/// `(index, value)`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be an item in a sequence of `{T}`",
    label = "not an item of a sequence of `{T}`",
    note = "the items are data points or plain `Scalar` y values in a dataset, datasets in a controller, and controllers in a chart",
)]
pub trait SequenceItem<T: ?Sized>: Debug + Clone + 'static {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &T));

    /// Syncs the states in the item with the environment, and returns whether anything may have
    /// changed. By default there is nothing to sync: a data point or a plain `Scalar` holds no
    /// states. Datasets and controllers sync theirs.
    fn sync(&mut self, _env: &mut Environment) -> bool {
        false
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

/// A controller is an item of a sequence of controllers.
impl<C: DatasetController + Clone> SequenceItem<dyn DatasetController> for C {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn DatasetController + 'static))) {
        f(index, self);
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        DatasetController::sync(self, env)
    }
}
