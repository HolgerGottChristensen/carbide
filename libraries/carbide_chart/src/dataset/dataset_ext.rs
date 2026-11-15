use carbide::state::IntoReadState;
use crate::{DataColor, DataSet, DataValue, StackId};
use crate::dataset::options_dataset::OptionsDataset;
use crate::element::{Fill, Interpolation};
use crate::scale::ScaleId;


pub trait DatasetExt<X: DataValue, Y: DataValue, Z: DataValue>: DataSet<X, Y, Z> + Clone + Sized {
    fn color<C: IntoReadState<DataColor>>(self, color: C) -> OptionsDataset<Self, C::Output, Interpolation, Fill, ScaleId, ScaleId, StackId> {
        OptionsDataset::new(self).color(color)
    }

    /// Draws the line between the points with `interpolation` instead of straight lines: smooth,
    /// monotone or stepped (see `Interpolation`).
    fn interpolation<S: IntoReadState<Interpolation>>(self, interpolation: S) -> OptionsDataset<Self, DataColor, S::Output, Fill, ScaleId, ScaleId, StackId> {
        OptionsDataset::new(self).interpolation(interpolation)
    }

    /// Fills the area between the line and `fill`: zero, a value, or the bottom or top of the
    /// chart area, making a line chart an area chart (see `Fill`). Bar controllers ignore it.
    fn fill<F: IntoReadState<Fill>>(self, fill: F) -> OptionsDataset<Self, DataColor, Interpolation, F::Output, ScaleId, ScaleId, StackId> {
        OptionsDataset::new(self).fill(fill)
    }

    /// Draws the dataset on the horizontal scale `x_scale` instead of `ScaleId::X`.
    fn x_scale<XS: IntoReadState<ScaleId>>(self, x_scale: XS) -> OptionsDataset<Self, DataColor, Interpolation, Fill, XS::Output, ScaleId, StackId> {
        OptionsDataset::new(self).x_scale(x_scale)
    }

    /// Draws the dataset on the vertical scale `y_scale` instead of `ScaleId::Y`.
    fn y_scale<YS: IntoReadState<ScaleId>>(self, y_scale: YS) -> OptionsDataset<Self, DataColor, Interpolation, Fill, ScaleId, YS::Output, StackId> {
        OptionsDataset::new(self).y_scale(y_scale)
    }

    /// Stacks the dataset on the datasets before it in the same controller with the same
    /// `stack` (see `StackId`). Bars stack on top of each other instead of beside each other;
    /// lines are drawn at the running total, and with `Fill::Stack` fill down to the line below.
    /// Scatter controllers don't stack, and ignore it.
    fn stack<ST: IntoReadState<StackId>>(self, stack: ST) -> OptionsDataset<Self, DataColor, Interpolation, Fill, ScaleId, ScaleId, ST::Output> {
        OptionsDataset::new(self).stack(stack)
    }

    /// Boxes the dataset, so datasets of different types can share a `Vec` or a state of a
    /// `Vec`, as the datasets of one controller. The point types stay part of the type: only
    /// datasets with the same `X`, `Y` and `Z` can be mixed.
    fn boxed(self) -> Box<dyn DataSet<X, Y, Z>> {
        Box::new(self)
    }
}

impl<X: DataValue, Y: DataValue, Z: DataValue, T> DatasetExt<X, Y, Z> for T where T: DataSet<X, Y, Z> + Clone + Sized {}
