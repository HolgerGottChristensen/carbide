use carbide::state::IntoReadState;
use crate::{DataColor, DataSet};
use crate::dataset::options_dataset::OptionsDataset;
use crate::element::Stepped;


pub trait DatasetExt: DataSet + Clone + Sized {
    fn color<C: IntoReadState<DataColor>>(self, color: C) -> OptionsDataset<Self, C::Output, Stepped> {
        OptionsDataset::new(self).color(color)
    }

    fn stepped<S: IntoReadState<Stepped>>(self, stepped: S) -> OptionsDataset<Self, DataColor, S::Output> {
        OptionsDataset::new(self).stepped(stepped)
    }
}

impl<T> DatasetExt for T where T: DataSet + Clone + Sized {}