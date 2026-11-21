use carbide::draw::Scalar;
use carbide::environment::Environment;
use std::cell::RefCell;
use indexmap::{IndexMap, IndexSet};

use crate::Value;
use crate::scale::{Axis, CategoryScale, LinearScale, Scale, ScaleId, TimeScale};

/// The data range reported for one scale while the chart collects bounds: the union of every
/// range included so far, or `None` when no dataset has reported anything yet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DataBounds {
    range: Option<(Scalar, Scalar)>,
}

impl DataBounds {
    /// Grows the bounds to include `min..=max`.
    pub fn include(&mut self, min: Scalar, max: Scalar) {
        self.range = Some(match self.range {
            Some((current_min, current_max)) => (current_min.min(min), current_max.max(max)),
            None => (min, max),
        });
    }

    pub fn range(&self) -> Option<(Scalar, Scalar)> {
        self.range
    }
}

/// A kind of scale a controller can ask for (see `ScaleBounds::request_kind`). The chart builds
/// its default `ScaleId::X` and `ScaleId::Y` scales from the kind asked for; a scale set with
/// `Chart::scale` is never replaced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleKind {
    /// A `LinearScale`, for numeric values. The default when no controller asks.
    Linear,
    /// A `CategoryScale`, for values that are indices into a list.
    Category,
    /// A `TimeScale`, for dates. The default for a scale whose data are dates, whatever a
    /// controller asks for (see `ScaleRequest::default_kind`).
    Time,
}

impl ScaleKind {
    /// A new scale of this kind along `axis`, with its default options.
    pub(crate) fn build(self, axis: Axis) -> Box<dyn Scale> {
        match self {
            ScaleKind::Linear => Box::new(LinearScale::new(axis)),
            ScaleKind::Category => Box::new(CategoryScale::new(axis)),
            ScaleKind::Time => Box::new(TimeScale::new(axis)),
        }
    }
}

/// What the controllers asked of one scale on one fit. Each request has its own rule for
/// combining what several datasets and controllers ask:
///
/// - the range is the union of every range included;
/// - the offset and the grid offset are on if anyone asks for them;
/// - the kind is the first one asked for, in controller and dataset order;
/// - the data's kind is the one the first value reported calls for, in controller and dataset
///   order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScaleRequest {
    bounds: DataBounds,
    offset: bool,
    grid_offset: bool,
    kind: Option<ScaleKind>,
    /// The kind of scale the first value reported calls for: `Some(None)` when it was a number,
    /// which leaves the kind to the controllers.
    data_kind: Option<Option<ScaleKind>>,
}

impl ScaleRequest {
    /// The union of the data ranges included, or `None` when nothing was included.
    pub fn range(&self) -> Option<(Scalar, Scalar)> {
        self.bounds.range()
    }

    /// Whether anyone asked for the scale to be offset (see `Scale::set_offset`).
    pub fn offset(&self) -> bool {
        self.offset
    }

    /// Whether anyone asked for grid lines between ticks (see `Scale::set_grid_offset_requested`).
    pub fn grid_offset(&self) -> bool {
        self.grid_offset
    }

    /// The first kind of scale asked for, if any.
    pub fn kind(&self) -> Option<ScaleKind> {
        self.kind
    }

    /// The kind of scale the data on the scale calls for, from the first value reported (see
    /// `ScaleBounds::report_value`): a time scale for times, a category scale for names, and
    /// `None` for numbers, or when nothing was reported.
    pub fn data_kind(&self) -> Option<ScaleKind> {
        self.data_kind.flatten()
    }

    /// The kind of scale the chart builds for a default scale with this request: a time scale
    /// when the data on it are dates and a category scale when they are text, as ag-charts picks
    /// a time or a category axis from the data, otherwise the kind a controller asked for, or a
    /// linear scale when none did.
    pub fn default_kind(&self) -> ScaleKind {
        self.data_kind().or(self.kind).unwrap_or(ScaleKind::Linear)
    }
}

/// What the controllers ask of each scale on one fit, keyed by scale id. Controllers fill it in
/// `DatasetController::collect_bounds`, under the ids their datasets are bound to; the chart then
/// fits each scale to its `ScaleRequest`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScaleBounds {
    /// Behind a `RefCell`, as the categories are, so controllers can add to the bounds while they
    /// read points through them (see `Coordinates`): every method takes `&self`.
    requests: RefCell<IndexMap<ScaleId, ScaleRequest>>,
    /// Each scale's categories, in the order they were first seen: the scale's own labels, then
    /// the text values controllers registered (see `register_category`). Behind a `RefCell` so
    /// values can be placed (`Coordinates`) while the bounds are only borrowed.
    categories: RefCell<IndexMap<ScaleId, IndexSet<String>>>,
}

impl ScaleBounds {
    /// Changes the request for `id` with `f`, created empty the first time `id` is seen.
    fn change(&self, id: &ScaleId, f: impl FnOnce(&mut ScaleRequest)) {
        let mut requests = self.requests.borrow_mut();

        // Looked up by reference first, so an owned id is only cloned the first time it is seen.
        if !requests.contains_key(id) {
            requests.insert(id.clone(), ScaleRequest::default());
        }

        f(requests.get_mut(id).expect("inserted above"));
    }

    /// Grows the range of the scale `id` to include `min..=max`.
    pub fn include(&self, id: &ScaleId, min: Scalar, max: Scalar) {
        self.change(id, |request| request.bounds.include(min, max));
    }

    /// Asks for the scale `id` to be offset on this fit, by half a step at both ends (see
    /// `Scale::set_offset`). Bar controllers ask for it on the scale their bars are spread along.
    pub fn request_offset(&self, id: &ScaleId) {
        self.change(id, |request| request.offset = true);
    }

    /// Asks for the scale `id` to draw its grid lines between ticks instead of on them, on this
    /// fit (see `Scale::set_grid_offset_requested`). Bar controllers ask for it so grid lines
    /// separate the bar groups. A scale whose grid offset was set explicitly keeps it.
    pub fn request_grid_offset(&self, id: &ScaleId) {
        self.change(id, |request| request.grid_offset = true);
    }

    /// Asks for the scale `id` to be of `kind`, if it is a default scale: bar and line charts ask
    /// for a category scale along x. The first kind asked for wins.
    pub fn request_kind(&self, id: &ScaleId, kind: ScaleKind) {
        self.change(id, |request| {
            request.kind.get_or_insert(kind);
        });
    }

    /// Reports a value placed on the scale `id`, so the chart can pick a default scale for what
    /// the data are (see `ScaleRequest::default_kind`): a time calls for a time scale, a name for
    /// a category scale, and a number leaves the kind to the controllers. Controllers report each
    /// dataset's first point that isn't missing, its x for the scale its x is placed on and its y
    /// for the scale its y is placed on. The first value reported wins.
    pub fn report_value(&self, id: &ScaleId, value: Value) {
        let kind = match value {
            Value::Number(_) => None,
            Value::Time(_) => Some(ScaleKind::Time),
            Value::Text(_) => Some(ScaleKind::Category),
        };

        self.change(id, |request| {
            request.data_kind.get_or_insert(kind);
        });
    }

    /// Adds the category `text` to the scale `id`, after the categories it already has, unless it
    /// has it, and returns its index. Indices are only ever added, so a category keeps its index
    /// while the bounds are collected. Controllers register every text value of their datasets
    /// before reading any (see `register_values` in `controller/drawing.rs`), so categories are
    /// numbered in controller, dataset and point order, as Chart.js and matplotlib number them as
    /// they first see them. The chart registers each scale's own labels first.
    pub fn register_category(&self, id: &ScaleId, text: &str) -> usize {
        let mut categories = self.categories.borrow_mut();
        let categories = categories.entry(id.clone()).or_default();

        match categories.get_index_of(text) {
            Some(index) => index,
            None => categories.insert_full(text.to_string()).0,
        }
    }

    /// The categories of the scale `id`, in index order: its own labels, then the text values
    /// registered (see `register_category`).
    pub fn categories(&self, id: &ScaleId) -> Vec<String> {
        self.categories.borrow().get(id).map(|categories| categories.iter().cloned().collect()).unwrap_or_default()
    }

    /// What was asked of the scale `id`; empty when nothing was.
    pub fn get(&self, id: &ScaleId) -> ScaleRequest {
        self.requests.borrow().get(id).copied().unwrap_or_default()
    }

    /// Every scale id something was asked of, in the order they were first seen.
    pub fn ids(&self) -> Vec<ScaleId> {
        self.requests.borrow().keys().cloned().collect()
    }
}

/// The scales a chart owns, keyed by id, in insertion order: the order the chart draws and lays
/// them out in: the default `ScaleId::X` and `ScaleId::Y` scales, and any added with
/// `Chart::scale`.
#[derive(Clone, Debug)]
pub struct Scales {
    scales: IndexMap<ScaleId, Box<dyn Scale>>,
}

impl Scales {
    pub fn new() -> Scales {
        let mut scales: IndexMap<ScaleId, Box<dyn Scale>> = IndexMap::new();
        scales.insert(ScaleId::X, Box::new(LinearScale::new(Axis::Horizontal)));
        scales.insert(ScaleId::Y, Box::new(LinearScale::new(Axis::Vertical)));

        Scales { scales }
    }

    /// Adds `scale` under `id`. A scale already under `id` is replaced in place, so replacing a
    /// default scale keeps its place in the draw order.
    pub fn insert(&mut self, id: ScaleId, scale: impl Scale) {
        self.insert_boxed(id, Box::new(scale));
    }

    /// `insert` for a scale that is already boxed.
    pub fn insert_boxed(&mut self, id: ScaleId, scale: Box<dyn Scale>) {
        self.scales.insert(id, scale);
    }

    /// The scale with the given id, if the chart has one.
    pub fn get(&self, id: &ScaleId) -> Option<&dyn Scale> {
        self.scales.get(id).map(|scale| scale.as_ref())
    }

    pub fn contains(&self, id: &ScaleId) -> bool {
        self.scales.contains_key(id)
    }

    /// Every scale, in the order the chart draws them.
    pub fn iter(&self) -> impl Iterator<Item=(&ScaleId, &dyn Scale)> + '_ {
        self.scales.iter().map(|(id, scale)| (id, scale.as_ref()))
    }

    /// Syncs the states in every scale with the environment (see `Scale::sync`), and returns
    /// whether anything may have changed.
    pub fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = false;
        for scale in self.scales.values_mut() {
            changed |= scale.sync(env);
        }
        changed
    }

    /// Fits each scale to what was asked of its id: its offset and grid offset, then its range.
    /// A scale without any reported data gets the range `0..1`.
    pub fn fit(&mut self, bounds: &ScaleBounds) {
        for (id, scale) in &mut self.scales {
            let request = bounds.get(id);
            let (min, max) = request.range().unwrap_or((0.0, 1.0));

            scale.set_categories(&bounds.categories(id));
            scale.set_offset(request.offset());
            scale.set_grid_offset_requested(request.grid_offset());
            scale.set_range(min, max);
        }
    }
}

impl Default for Scales {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::AxisPosition;

    #[test]
    fn bounds_without_data_have_no_range() {
        assert_eq!(DataBounds::default().range(), None);
        assert_eq!(ScaleBounds::default().get(&ScaleId::X).range(), None);
    }

    #[test]
    fn bounds_are_the_union_of_every_included_range() {
        let mut bounds = DataBounds::default();
        bounds.include(-5.0, 10.0);
        bounds.include(0.0, 20.0);
        bounds.include(-8.0, 3.0);

        assert_eq!(bounds.range(), Some((-8.0, 20.0)));
    }

    /// Replacing a default scale keeps its place in the draw order; a new scale goes last.
    #[test]
    fn inserting_replaces_in_place_and_appends_new_ids() {
        let mut scales = Scales::new();
        scales.insert(ScaleId::new("right"), LinearScale::new(Axis::Vertical).position(AxisPosition::Right));
        scales.insert(ScaleId::X, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top));

        let order: Vec<ScaleId> = scales.iter().map(|(id, _)| id.clone()).collect();
        assert_eq!(order, vec![ScaleId::X, ScaleId::Y, ScaleId::new("right")]);
        assert_eq!(scales.get(&ScaleId::X).unwrap().axis_options().position(), AxisPosition::Top);
    }

    /// An offset pads the mapping by half a tick step at both ends without changing the ticks:
    /// for ticks 0..=6 (step 1) the mapped range becomes -0.5..6.5.
    #[test]
    fn offset_pads_half_a_tick_step_at_both_ends() {
        let mut scale = LinearScale::new(Axis::Horizontal);
        scale.set_range(0.0, 6.0);
        assert_eq!(scale.ticks(), &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(scale.normalize(0.0), 0.0);

        scale.set_offset(true);
        scale.set_range(0.0, 6.0);

        assert_eq!(scale.ticks(), &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0], "the ticks don't change");
        assert!((scale.normalize(0.0) - 0.5 / 7.0).abs() < 1e-12);
        assert!((scale.normalize(6.0) - 6.5 / 7.0).abs() < 1e-12);
    }

    /// A side the axis can't use is ignored, keeping the axis default.
    #[test]
    fn positions_must_fit_the_axis() {
        let horizontal = LinearScale::new(Axis::Horizontal).position(AxisPosition::Left);
        assert_eq!(horizontal.axis_options().position(), AxisPosition::Bottom);

        let vertical = LinearScale::new(Axis::Vertical).position(AxisPosition::Top);
        assert_eq!(vertical.axis_options().position(), AxisPosition::Left);
    }

    #[test]
    fn bounds_are_collected_per_scale_id() {
        let bounds = ScaleBounds::default();
        bounds.include(&ScaleId::Y, 0.0, 10.0);
        bounds.include(&ScaleId::new("right"), 100.0, 200.0);
        bounds.include(&ScaleId::Y, -5.0, 5.0);

        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((-5.0, 10.0)));
        assert_eq!(bounds.get(&ScaleId::new("right")).range(), Some((100.0, 200.0)));
        assert_eq!(bounds.get(&ScaleId::X).range(), None);
        assert_eq!(bounds.ids(), vec![ScaleId::Y, ScaleId::new("right")]);
    }

    /// Each request combines by its own rule: offsets and grid offsets are on if anyone asks,
    /// the first kind asked for wins, and asking without data leaves the range empty.
    #[test]
    fn requests_combine_by_their_own_rules() {
        let bounds = ScaleBounds::default();
        bounds.request_kind(&ScaleId::X, ScaleKind::Category);
        bounds.request_kind(&ScaleId::X, ScaleKind::Linear);
        bounds.request_grid_offset(&ScaleId::X);
        bounds.request_offset(&ScaleId::X);
        bounds.request_offset(&ScaleId::X);

        let x = bounds.get(&ScaleId::X);
        assert_eq!(x.kind(), Some(ScaleKind::Category), "the first kind asked for wins");
        assert!(x.offset() && x.grid_offset());
        assert_eq!(x.range(), None, "requests alone include no data");

        let y = bounds.get(&ScaleId::Y);
        assert_eq!((y.kind(), y.offset(), y.grid_offset()), (None, false, false));
    }
}
