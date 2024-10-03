use std::io::Read;
use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::{IntoReadState, ReadState};
use crate::{DataColor, DataPoint, DataSet, DataSetOptions};
use crate::element::Stepped;

#[derive(Debug, Clone)]
pub struct OptionsDataset<T: DataSet + 'static, C: ReadState<T=DataColor>, S: ReadState<T=Stepped>> {
    inner: T,
    color: Option<C>,
    stepped: Option<S>,
}

impl OptionsDataset<Vec<(Scalar, Scalar)>, DataColor, Stepped> {
    pub fn new<T: DataSet + 'static>(dataset: T) -> OptionsDataset<T, DataColor, Stepped> {
        OptionsDataset {
            inner: dataset,
            color: None,
            stepped: None,
        }
    }
}

impl<T: DataSet + 'static, C: ReadState<T=DataColor>, S: ReadState<T=Stepped>> OptionsDataset<T, C, S> {
    pub fn color<C2: IntoReadState<DataColor>>(self, color: C2) -> OptionsDataset<T, C2::Output, S> {
        OptionsDataset {
            inner: self.inner,
            color: Some(color.into_read_state()),
            stepped: self.stepped
        }
    }

    pub fn stepped<S2: IntoReadState<Stepped>>(self, stepped: S2) -> OptionsDataset<T, C, S2::Output> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            stepped: Some(stepped.into_read_state())
        }
    }
}

impl<T: DataSet + 'static, C: ReadState<T=DataColor>, S: ReadState<T=Stepped>> DataSet for OptionsDataset<T, C, S> {
    type X = T::X;
    type Y = T::Y;
    type Z = T::Z;

    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=Self::X, Y=Self::Y, Z=Self::Z>)) {
        self.inner.points(f);
    }

    fn min(&self) -> (Self::X, Self::Y, Self::Z) {
        self.inner.min()
    }

    fn max(&self) -> (Self::X, Self::Y, Self::Z) {
        self.inner.max()
    }

    fn options(&self, env: &mut Environment) -> DataSetOptions {
        let options = self.inner.options(env);

        DataSetOptions {
            color: self.color.clone().map_or(options.color, |mut color| { color.sync(env); *color.value() }),
            stepped: self.stepped.clone().map_or(options.stepped, |mut stepped| { stepped.sync(env); *stepped.value() }),
        }
    }
}