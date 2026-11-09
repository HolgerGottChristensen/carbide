use carbide::state::IntoReadState;
use crate::{DataColor, DataSet, DataValue};
use crate::dataset::options_dataset::OptionsDataset;
use crate::element::Stepped;
use crate::scale::ScaleId;


pub trait DatasetExt<X: DataValue, Y: DataValue, Z: DataValue>: DataSet<X, Y, Z> + Clone + Sized {
    fn color<C: IntoReadState<DataColor>>(self, color: C) -> OptionsDataset<Self, C::Output, Stepped, ScaleId, ScaleId> {
        OptionsDataset::new(self).color(color)
    }

    fn stepped<S: IntoReadState<Stepped>>(self, stepped: S) -> OptionsDataset<Self, DataColor, S::Output, ScaleId, ScaleId> {
        OptionsDataset::new(self).stepped(stepped)
    }

    /// Draws the dataset on the horizontal scale `x_scale` instead of `ScaleId::X`.
    fn x_scale<XS: IntoReadState<ScaleId>>(self, x_scale: XS) -> OptionsDataset<Self, DataColor, Stepped, XS::Output, ScaleId> {
        OptionsDataset::new(self).x_scale(x_scale)
    }

    /// Draws the dataset on the vertical scale `y_scale` instead of `ScaleId::Y`.
    fn y_scale<YS: IntoReadState<ScaleId>>(self, y_scale: YS) -> OptionsDataset<Self, DataColor, Stepped, ScaleId, YS::Output> {
        OptionsDataset::new(self).y_scale(y_scale)
    }
}

impl<X: DataValue, Y: DataValue, Z: DataValue, T> DatasetExt<X, Y, Z> for T where T: DataSet<X, Y, Z> + Clone + Sized {}
