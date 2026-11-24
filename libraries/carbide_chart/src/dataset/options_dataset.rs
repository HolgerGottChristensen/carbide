use carbide::draw::Scalar;
use carbide::environment::{Environment};
use carbide::state::{AnyState, IntoReadState, IntoState, LocalState, ReadState, StateExtNew};
use crate::{DataColor, DataPoint, DataSet, DataSetOptions, StackId};
use crate::dataset::{BoxPoint, OhlcPoint, RangePoint, SizedPoint};
use crate::element::{Fill, Interpolation, SpanGaps};
use crate::scale::ScaleId;
use crate::Sequence;

/// A dataset with options set through `DatasetExt`. Every option is a state, synced with the
/// rest of the dataset (see `DataSet::sync`) and read as it was last synced when the chart asks
/// for the options.
///
/// Its visibility is a writable state, its own one, shown to start with, unless set with
/// `visible`, so a click on its legend entry can hide it.
#[derive(Debug, Clone)]
pub struct OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
where
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    F: ReadState<T=Fill>,
    G: ReadState<T=SpanGaps>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
    ST: ReadState<T=StackId>,
    L: ReadState<T=String>,
{
    inner: T,
    color: Option<C>,
    interpolation: Option<S>,
    fill: Option<F>,
    span_gaps: Option<G>,
    x_scale: Option<XS>,
    y_scale: Option<YS>,
    stack: Option<ST>,
    label: Option<L>,
    /// Whether the dataset is shown: kept type-erased rather than as one more type parameter, and
    /// writable, so the legend can flip it.
    visible: Box<dyn AnyState<T=bool>>,
}

impl OptionsDataset<Vec<(Scalar, Scalar)>, DataColor, Interpolation, Fill, SpanGaps, ScaleId, ScaleId, StackId, String> {
    pub fn new<T>(dataset: T) -> OptionsDataset<T, DataColor, Interpolation, Fill, SpanGaps, ScaleId, ScaleId, StackId, String> {
        OptionsDataset {
            inner: dataset,
            color: None,
            interpolation: None,
            fill: None,
            span_gaps: None,
            x_scale: None,
            y_scale: None,
            stack: None,
            label: None,
            visible: LocalState::new(true).as_dyn(),
        }
    }
}

impl<T, C, S, F, G, XS, YS, ST, L> OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
where
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    F: ReadState<T=Fill>,
    G: ReadState<T=SpanGaps>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
    ST: ReadState<T=StackId>,
    L: ReadState<T=String>,
{
    pub fn color<C2: IntoReadState<DataColor>>(self, color: C2) -> OptionsDataset<T, C2::Output, S, F, G, XS, YS, ST, L> {
        OptionsDataset {
            inner: self.inner,
            color: Some(color.into_read_state()),
            interpolation: self.interpolation,
            fill: self.fill,
            span_gaps: self.span_gaps,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
            stack: self.stack,
            label: self.label,
            visible: self.visible,
        }
    }

    /// Draws the line between the points with `interpolation` instead of straight lines (see
    /// `Interpolation`).
    pub fn interpolation<S2: IntoReadState<Interpolation>>(self, interpolation: S2) -> OptionsDataset<T, C, S2::Output, F, G, XS, YS, ST, L> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: Some(interpolation.into_read_state()),
            fill: self.fill,
            span_gaps: self.span_gaps,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
            stack: self.stack,
            label: self.label,
            visible: self.visible,
        }
    }

    /// Fills the area between the line and `fill` (see `Fill`).
    pub fn fill<F2: IntoReadState<Fill>>(self, fill: F2) -> OptionsDataset<T, C, S, F2::Output, G, XS, YS, ST, L> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: self.interpolation,
            fill: Some(fill.into_read_state()),
            span_gaps: self.span_gaps,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
            stack: self.stack,
            label: self.label,
            visible: self.visible,
        }
    }

    /// Whether the line breaks or joins across gaps (see `SpanGaps`).
    pub fn span_gaps<G2: IntoReadState<SpanGaps>>(self, span_gaps: G2) -> OptionsDataset<T, C, S, F, G2::Output, XS, YS, ST, L> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: self.interpolation,
            fill: self.fill,
            span_gaps: Some(span_gaps.into_read_state()),
            x_scale: self.x_scale,
            y_scale: self.y_scale,
            stack: self.stack,
            label: self.label,
            visible: self.visible,
        }
    }

    /// Draws the dataset on the horizontal scale `x_scale` instead of `ScaleId::X`.
    pub fn x_scale<XS2: IntoReadState<ScaleId>>(self, x_scale: XS2) -> OptionsDataset<T, C, S, F, G, XS2::Output, YS, ST, L> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: self.interpolation,
            fill: self.fill,
            span_gaps: self.span_gaps,
            x_scale: Some(x_scale.into_read_state()),
            y_scale: self.y_scale,
            stack: self.stack,
            label: self.label,
            visible: self.visible,
        }
    }

    /// Draws the dataset on the vertical scale `y_scale` instead of `ScaleId::Y`.
    pub fn y_scale<YS2: IntoReadState<ScaleId>>(self, y_scale: YS2) -> OptionsDataset<T, C, S, F, G, XS, YS2::Output, ST, L> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: self.interpolation,
            fill: self.fill,
            span_gaps: self.span_gaps,
            x_scale: self.x_scale,
            y_scale: Some(y_scale.into_read_state()),
            stack: self.stack,
            label: self.label,
            visible: self.visible,
        }
    }

    /// Stacks the dataset's bars on those of the other datasets in its controller with the same
    /// `stack` (see `StackId`).
    pub fn stack<ST2: IntoReadState<StackId>>(self, stack: ST2) -> OptionsDataset<T, C, S, F, G, XS, YS, ST2::Output, L> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: self.interpolation,
            fill: self.fill,
            span_gaps: self.span_gaps,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
            stack: Some(stack.into_read_state()),
            label: self.label,
            visible: self.visible,
        }
    }

    /// Names the dataset, for the legend (see `DatasetExt::label`).
    pub fn label<L2: IntoReadState<String>>(self, label: L2) -> OptionsDataset<T, C, S, F, G, XS, YS, ST, L2::Output> {
        OptionsDataset {
            inner: self.inner,
            color: self.color,
            interpolation: self.interpolation,
            fill: self.fill,
            span_gaps: self.span_gaps,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
            stack: self.stack,
            label: Some(label.into_read_state()),
            visible: self.visible,
        }
    }

    /// Shows or hides the dataset with `visible`, a writable state of a `bool` (see
    /// `DatasetExt::visible`).
    pub fn visible(mut self, visible: impl IntoState<bool>) -> OptionsDataset<T, C, S, F, G, XS, YS, ST, L> {
        self.visible = visible.into_state().as_dyn();
        self
    }
}

impl<T, C, S, F, G, XS, YS, ST, L> OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
where
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    F: ReadState<T=Fill>,
    G: ReadState<T=SpanGaps>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
    ST: ReadState<T=StackId>,
    L: ReadState<T=String>,
{
    /// The dataset the options are set on.
    pub(crate) fn inner(&self) -> &T {
        &self.inner
    }
}

/// A dataset with options is still a sequence of whatever kind of point the dataset inside is, so
/// a mark for that kind can be added after the options.
macro_rules! point_sequences {
    ($($point:ident),+) => {
        $(
            impl<T, C, S, F, G, XS, YS, ST, L> Sequence<dyn $point> for OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
            where
                T: Sequence<dyn $point>,
                C: ReadState<T=DataColor>,
                S: ReadState<T=Interpolation>,
                F: ReadState<T=Fill>,
                G: ReadState<T=SpanGaps>,
                XS: ReadState<T=ScaleId>,
                YS: ReadState<T=ScaleId>,
                ST: ReadState<T=StackId>,
                L: ReadState<T=String>,
            {
                fn foreach(&self, f: &mut dyn FnMut(usize, &(dyn $point + 'static))) {
                    self.inner.foreach(f);
                }

                /// Syncs the points; the option states are synced with the dataset (`DataSet::sync`).
                fn sync(&mut self, env: &mut Environment) -> bool {
                    Sequence::<dyn $point>::sync(&mut self.inner, env)
                }
            }
        )+
    };
}

point_sequences!(RangePoint, SizedPoint, BoxPoint, OhlcPoint);

/// Implemented directly rather than through `Sequence`, so it can override `options`: an
/// `OptionsDataset` is not a sequence of points itself.
impl<T, C, S, F, G, XS, YS, ST, L> DataSet for OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
where
    T: DataSet + Clone,
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    F: ReadState<T=Fill>,
    G: ReadState<T=SpanGaps>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
    ST: ReadState<T=StackId>,
    L: ReadState<T=String>,
{
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint)) {
        self.inner.points(f);
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

        if let Some(fill) = &mut self.fill {
            changed |= fill.sync(env);
        }

        if let Some(span_gaps) = &mut self.span_gaps {
            changed |= span_gaps.sync(env);
        }

        if let Some(x_scale) = &mut self.x_scale {
            changed |= x_scale.sync(env);
        }

        if let Some(y_scale) = &mut self.y_scale {
            changed |= y_scale.sync(env);
        }

        if let Some(stack) = &mut self.stack {
            changed |= stack.sync(env);
        }

        if let Some(label) = &mut self.label {
            changed |= label.sync(env);
        }

        changed |= self.visible.sync(env);

        changed
    }

    /// The inner dataset's options, with each option set here read from its state as it was last
    /// synced.
    fn options(&self) -> DataSetOptions {
        let options = self.inner.options();

        DataSetOptions {
            color: self.color.as_ref().map_or(options.color, |color| *color.value()),
            interpolation: self.interpolation.as_ref().map_or(options.interpolation, |interpolation| *interpolation.value()),
            fill: self.fill.as_ref().map_or(options.fill, |fill| *fill.value()),
            span_gaps: self.span_gaps.as_ref().map_or(options.span_gaps, |span_gaps| *span_gaps.value()),
            x_scale: self.x_scale.as_ref().map_or(options.x_scale, |x_scale| x_scale.value().clone()),
            y_scale: self.y_scale.as_ref().map_or(options.y_scale, |y_scale| y_scale.value().clone()),
            stack: self.stack.as_ref().map_or(options.stack, |stack| Some(stack.value().clone())),
            label: self.label.as_ref().map_or(options.label, |label| Some(label.value().clone())),
            // Shown only when both this visibility and that of the dataset inside are.
            visible: options.visible && *self.visible.value_dyn(),
        }
    }

    /// Writes the visibility state, through a clone of it, which shares its value.
    fn set_visible(&self, visible: bool) -> bool {
        if *self.visible.value_dyn() != visible {
            let mut state = self.visible.clone();
            state.set_value_dyn(visible);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use carbide::environment::Environment;
    use carbide::state::{LocalState, ReadState, State};
    use crate::{DataSet, DatasetExt};

    /// A dataset with options is shown by default, through a visibility state of its own, which
    /// `set_visible` writes. With `visible(state)`, it reads the app's state, and `set_visible`
    /// writes through to it, as a click on its legend entry does. A plain dataset can't be hidden.
    #[test]
    fn visibility_is_a_state() {
        let own = vec![1.0, 2.0].label("Own");
        assert!(own.options().visible, "shown to start with");
        assert!(own.set_visible(false));
        assert!(!own.options().visible, "its own state, written");

        let mut shown = LocalState::new(true);
        let mut dataset = vec![1.0, 2.0].label("App's").visible(shown.clone());
        assert!(dataset.set_visible(false));
        assert!(!*shown.value(), "written through to the app's state");

        *shown.value_mut() = true;
        dataset.sync(&mut Environment::new());
        assert!(dataset.options().visible, "the app showing it again");

        let plain = vec![1.0, 2.0];
        assert!(!plain.set_visible(false), "a dataset without options can't be hidden");
        assert!(plain.options().visible);
    }
}
