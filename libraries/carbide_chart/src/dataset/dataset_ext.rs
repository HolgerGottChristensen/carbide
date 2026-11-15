use carbide::state::IntoReadState;
use crate::{DataColor, DataSet, DataValue};
use crate::dataset::options_dataset::OptionsDataset;
use crate::element::Interpolation;
use crate::scale::ScaleId;


pub trait DatasetExt<X: DataValue, Y: DataValue, Z: DataValue>: DataSet<X, Y, Z> + Clone + Sized {
    fn color<C: IntoReadState<DataColor>>(self, color: C) -> OptionsDataset<Self, C::Output, Interpolation, ScaleId, ScaleId> {
        OptionsDataset::new(self).color(color)
    }

    /// Draws the line between the points with `interpolation` instead of straight lines: smooth,
    /// monotone or stepped (see `Interpolation`).
    fn interpolation<S: IntoReadState<Interpolation>>(self, interpolation: S) -> OptionsDataset<Self, DataColor, S::Output, ScaleId, ScaleId> {
        OptionsDataset::new(self).interpolation(interpolation)
    }

    /// Draws the dataset on the horizontal scale `x_scale` instead of `ScaleId::X`.
    fn x_scale<XS: IntoReadState<ScaleId>>(self, x_scale: XS) -> OptionsDataset<Self, DataColor, Interpolation, XS::Output, ScaleId> {
        OptionsDataset::new(self).x_scale(x_scale)
    }

    /// Draws the dataset on the vertical scale `y_scale` instead of `ScaleId::Y`.
    fn y_scale<YS: IntoReadState<ScaleId>>(self, y_scale: YS) -> OptionsDataset<Self, DataColor, Interpolation, ScaleId, YS::Output> {
        OptionsDataset::new(self).y_scale(y_scale)
    }

    /// Boxes the dataset, so datasets of different types can share a `Vec` or a state of a
    /// `Vec`, as the datasets of one controller. The point types stay part of the type: only
    /// datasets with the same `X`, `Y` and `Z` can be mixed.
    fn boxed(self) -> Box<dyn DataSet<X, Y, Z>> {
        Box::new(self)
    }
}

impl<X: DataValue, Y: DataValue, Z: DataValue, T> DatasetExt<X, Y, Z> for T where T: DataSet<X, Y, Z> + Clone + Sized {}
