use carbide::draw::Scalar;
use carbide::environment::{Environment};
use carbide::state::{IntoReadState, ReadState};
use crate::{DataColor, DataPoint, DataSet, DataSetOptions, DataValue};
use crate::element::Stepped;
use crate::scale::ScaleId;

/// A dataset with options set through `DatasetExt`. Every option is a state, read from the
/// environment each time the chart asks for the options.
#[derive(Debug, Clone)]
pub struct OptionsDataset<T, C, S, XS, YS>
where
    C: ReadState<T=DataColor>,
    S: ReadState<T=Stepped>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
{
    inner: T,
    color: Option<C>,
    stepped: Option<S>,
    x_scale: Option<XS>,
    y_scale: Option<YS>,
}

impl OptionsDataset<Vec<(Scalar, Scalar)>, DataColor, Stepped, ScaleId, ScaleId> {
    pub fn new<T>(dataset: T) -> OptionsDataset<T, DataColor, Stepped, ScaleId, ScaleId> {
        OptionsDataset {
            inner: dataset,
            color: None,
            stepped: None,
            x_scale: None,
            y_scale: None,
        }
    }
}

impl<T, C, S, XS, YS> OptionsDataset<T, C, S, XS, YS>
where
    C: ReadState<T=DataColor>,
    S: ReadState<T=Stepped>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
{
    pub fn color<C2: IntoReadState<DataColor>>(self, color: C2) -> OptionsDataset<T, C2::Output, S, XS, YS> {
        OptionsDataset {
            inner: self.inner,
            color: Some(color.into_read_state()),
            stepped: self.stepped,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        }
    }

    pub fn stepped<S2: IntoReadState<Stepped>>(self, stepped: S2) -> OptionsDataset<T, C, S2::Output, XS, YS> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            stepped: Some(stepped.into_read_state()),
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        }
    }

    /// Draws the dataset on the horizontal scale `x_scale` instead of `ScaleId::X`.
    pub fn x_scale<XS2: IntoReadState<ScaleId>>(self, x_scale: XS2) -> OptionsDataset<T, C, S, XS2::Output, YS> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            stepped: self.stepped,
            x_scale: Some(x_scale.into_read_state()),
            y_scale: self.y_scale,
        }
    }

    /// Draws the dataset on the vertical scale `y_scale` instead of `ScaleId::Y`.
    pub fn y_scale<YS2: IntoReadState<ScaleId>>(self, y_scale: YS2) -> OptionsDataset<T, C, S, XS, YS2::Output> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            stepped: self.stepped,
            x_scale: self.x_scale,
            y_scale: Some(y_scale.into_read_state()),
        }
    }
}

/// Implemented directly rather than through `Sequence`, so it can override `options`: an
/// `OptionsDataset` is not a sequence of points itself.
impl<X: DataValue, Y: DataValue, Z: DataValue, T, C, S, XS, YS> DataSet<X, Y, Z> for OptionsDataset<T, C, S, XS, YS>
where
    T: DataSet<X, Y, Z>,
    C: ReadState<T=DataColor>,
    S: ReadState<T=Stepped>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
{
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint<X=X, Y=Y, Z=Z>)) {
        self.inner.points(f);
    }

    fn min(&self) -> Option<(X, Y, Z)> {
        self.inner.min()
    }

    fn max(&self) -> Option<(X, Y, Z)> {
        self.inner.max()
    }

    fn options(&self, env: &mut Environment) -> DataSetOptions {
        let options = self.inner.options(env);

        DataSetOptions {
            color: self.color.clone().map_or(options.color, |mut color| { color.sync(env); *color.value() }),
            stepped: self.stepped.clone().map_or(options.stepped, |mut stepped| { stepped.sync(env); *stepped.value() }),
            x_scale: self.x_scale.clone().map_or(options.x_scale, |mut x_scale| { x_scale.sync(env); x_scale.value().clone() }),
            y_scale: self.y_scale.clone().map_or(options.y_scale, |mut y_scale| { y_scale.sync(env); y_scale.value().clone() }),
        }
    }
}
