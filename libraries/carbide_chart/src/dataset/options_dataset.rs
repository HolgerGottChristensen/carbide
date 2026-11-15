use carbide::draw::Scalar;
use carbide::environment::{Environment};
use carbide::state::{IntoReadState, ReadState};
use crate::{DataColor, DataPoint, DataSet, DataSetOptions, DataValue};
use crate::element::Interpolation;
use crate::scale::ScaleId;

/// A dataset with options set through `DatasetExt`. Every option is a state, synced with the
/// rest of the dataset (see `DataSet::sync`) and read as it was last synced when the chart asks
/// for the options.
#[derive(Debug, Clone)]
pub struct OptionsDataset<T, C, S, XS, YS>
where
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
{
    inner: T,
    color: Option<C>,
    interpolation: Option<S>,
    x_scale: Option<XS>,
    y_scale: Option<YS>,
}

impl OptionsDataset<Vec<(Scalar, Scalar)>, DataColor, Interpolation, ScaleId, ScaleId> {
    pub fn new<T>(dataset: T) -> OptionsDataset<T, DataColor, Interpolation, ScaleId, ScaleId> {
        OptionsDataset {
            inner: dataset,
            color: None,
            interpolation: None,
            x_scale: None,
            y_scale: None,
        }
    }
}

impl<T, C, S, XS, YS> OptionsDataset<T, C, S, XS, YS>
where
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
{
    pub fn color<C2: IntoReadState<DataColor>>(self, color: C2) -> OptionsDataset<T, C2::Output, S, XS, YS> {
        OptionsDataset {
            inner: self.inner,
            color: Some(color.into_read_state()),
            interpolation: self.interpolation,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        }
    }

    /// Draws the line between the points with `interpolation` instead of straight lines (see
    /// `Interpolation`).
    pub fn interpolation<S2: IntoReadState<Interpolation>>(self, interpolation: S2) -> OptionsDataset<T, C, S2::Output, XS, YS> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: Some(interpolation.into_read_state()),
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        }
    }

    /// Draws the dataset on the horizontal scale `x_scale` instead of `ScaleId::X`.
    pub fn x_scale<XS2: IntoReadState<ScaleId>>(self, x_scale: XS2) -> OptionsDataset<T, C, S, XS2::Output, YS> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: self.interpolation,
            x_scale: Some(x_scale.into_read_state()),
            y_scale: self.y_scale,
        }
    }

    /// Draws the dataset on the vertical scale `y_scale` instead of `ScaleId::Y`.
    pub fn y_scale<YS2: IntoReadState<ScaleId>>(self, y_scale: YS2) -> OptionsDataset<T, C, S, XS, YS2::Output> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: self.interpolation,
            x_scale: self.x_scale,
            y_scale: Some(y_scale.into_read_state()),
        }
    }
}

/// Implemented directly rather than through `Sequence`, so it can override `options`: an
/// `OptionsDataset` is not a sequence of points itself.
impl<X: DataValue, Y: DataValue, Z: DataValue, T, C, S, XS, YS> DataSet<X, Y, Z> for OptionsDataset<T, C, S, XS, YS>
where
    T: DataSet<X, Y, Z> + Clone,
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
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

    /// Syncs the inner dataset and every option state.
    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = self.inner.sync(env);

        if let Some(color) = &mut self.color {
            changed |= color.sync(env);
        }

        if let Some(interpolation) = &mut self.interpolation {
            changed |= interpolation.sync(env);
        }

        if let Some(x_scale) = &mut self.x_scale {
            changed |= x_scale.sync(env);
        }

        if let Some(y_scale) = &mut self.y_scale {
            changed |= y_scale.sync(env);
        }

        changed
    }

    /// The inner dataset's options, with each option set here read from its state as it was last
    /// synced.
    fn options(&self) -> DataSetOptions {
        let options = self.inner.options();

        DataSetOptions {
            color: self.color.as_ref().map_or(options.color, |color| *color.value()),
            interpolation: self.interpolation.as_ref().map_or(options.interpolation, |interpolation| *interpolation.value()),
            x_scale: self.x_scale.as_ref().map_or(options.x_scale, |x_scale| x_scale.value().clone()),
            y_scale: self.y_scale.as_ref().map_or(options.y_scale, |y_scale| y_scale.value().clone()),
        }
    }
}
