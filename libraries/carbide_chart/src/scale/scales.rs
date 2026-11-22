use carbide::draw::{Position, Rect, Scalar};
use carbide::environment::Environment;
use std::cell::RefCell;
use std::collections::HashSet;
use indexmap::{IndexMap, IndexSet};

use crate::Value;
use crate::scale::{Axis, CategoryScale, GridShape, LinearScale, PolarRole, Scale, ScaleId, TimeScale};
use crate::scale::polar::{Circle, Polar};
use crate::controller::orientation::pixel_normalized;

/// The data range reported for one scale while the chart collects bounds: the union of every
/// range included so far, or `None` when no dataset has reported anything yet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DataBounds {
    range: Option<(Scalar, Scalar)>,
    smallest_positive: Option<Scalar>,
}

impl DataBounds {
    /// Grows the bounds to include `min..=max`, and keeps the smallest end above zero (see
    /// `smallest_positive`).
    pub fn include(&mut self, min: Scalar, max: Scalar) {
        self.range = Some(match self.range {
            Some((current_min, current_max)) => (current_min.min(min), current_max.max(max)),
            None => (min, max),
        });

        for end in [min, max] {
            if end > 0.0 && self.smallest_positive.is_none_or(|smallest| end < smallest) {
                self.smallest_positive = Some(end);
            }
        }
    }

    pub fn range(&self) -> Option<(Scalar, Scalar)> {
        self.range
    }

    /// The smallest end above zero of any range included, or `None` when none was above zero,
    /// for scales without a place for zero and below (see `Scale::set_smallest_positive`). Ends,
    /// not the insides of ranges: a bar includes `0..12`, its base and its value, and its
    /// smallest positive end is its value, 12.
    pub fn smallest_positive(&self) -> Option<Scalar> {
        self.smallest_positive
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
/// - the grid shape is a circle if anyone asks for one, and a polygon if all who ask want one;
/// - a radial scale's role is the first one asked for, and asking for the other is a conflict;
/// - the data's kind is the one the first value reported calls for, in controller and dataset
///   order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScaleRequest {
    bounds: DataBounds,
    offset: bool,
    grid_offset: bool,
    kind: Option<ScaleKind>,
    grid_shape: Option<GridShape>,
    polar_role: Option<PolarRole>,
    /// Whether a dataset asked for the other role than the first.
    polar_role_conflict: bool,
    /// Whether a lying controller's categories run out along the scale (see `request_lying`).
    lying: bool,
    /// Whether a controller asked for the scale's grid, ticks and labels not to be drawn (see
    /// `request_hidden_decoration`).
    hidden_decoration: bool,
    /// The kind of scale the first value reported calls for: `Some(None)` when it was a number,
    /// which leaves the kind to the controllers.
    data_kind: Option<Option<ScaleKind>>,
}

impl ScaleRequest {
    /// The union of the data ranges included, or `None` when nothing was included.
    pub fn range(&self) -> Option<(Scalar, Scalar)> {
        self.bounds.range()
    }

    /// The smallest end above zero of the data ranges included (see `DataBounds::smallest_positive`).
    pub fn smallest_positive(&self) -> Option<Scalar> {
        self.bounds.smallest_positive()
    }

    /// Whether a controller asked for the scale's decoration to be hidden (see
    /// `ScaleBounds::request_hidden_decoration`).
    pub fn hidden_decoration(&self) -> bool {
        self.hidden_decoration
    }

    /// Whether anyone asked for the scale to be offset (see `Scale::set_offset`).
    pub fn offset(&self) -> bool {
        self.offset
    }

    /// Whether anyone asked for grid lines between ticks (see `Scale::set_grid_offset_requested`).
    pub fn grid_offset(&self) -> bool {
        self.grid_offset
    }

    /// The shape asked for a radius scale's rings, if anyone asked (see
    /// `ScaleBounds::request_grid_shape`).
    pub fn grid_shape(&self) -> Option<GridShape> {
        self.grid_shape
    }

    /// The role asked for a radial scale, the first if several were (see
    /// `ScaleBounds::request_polar_role`).
    pub fn polar_role(&self) -> Option<PolarRole> {
        self.polar_role
    }

    /// Whether datasets asked for both roles of a radial scale: bound as the angle by one and as
    /// the radius by another.
    pub fn polar_role_conflict(&self) -> bool {
        self.polar_role_conflict
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
    /// The ids of the chart's radial scales, set before the controllers collect (see
    /// `is_radial`).
    radial: IndexSet<ScaleId>,
    /// The values shown by each zoomed scale, lowest first, when some scale fits to the visible
    /// data only (see `is_hidden`).
    windows: IndexMap<ScaleId, (Scalar, Scalar)>,
    /// The scales that fit to the visible data only (see `AxisOptions::fit_visible`).
    fitting_visible: IndexSet<ScaleId>,
}

impl ScaleBounds {
    /// The bounds for a chart whose radial scales have the ids `radial`, so controllers can read
    /// polar datasets as polar while collecting (see `is_radial`).
    pub(crate) fn with_radial(radial: IndexSet<ScaleId>) -> ScaleBounds {
        ScaleBounds { radial, ..ScaleBounds::default() }
    }

    /// The bounds for a second fit, to the data visible in `windows`, the values each zoomed scale
    /// shows, for the scales in `fitting_visible` (see `is_hidden`).
    pub(crate) fn with_windows(radial: IndexSet<ScaleId>, windows: IndexMap<ScaleId, (Scalar, Scalar)>, fitting_visible: IndexSet<ScaleId>) -> ScaleBounds {
        ScaleBounds { radial, windows, fitting_visible, ..ScaleBounds::default() }
    }

    /// Whether a value of a point at `x` on the scale `index_id` is left out of the bounds of
    /// the scale `value_id`: when `value_id` fits to the visible data only (see
    /// `LinearScale::fit_visible`) and `x` is outside the zoom window of `index_id`. Controllers
    /// still include `x` itself, so the zoomed scale keeps its whole range.
    pub fn is_hidden(&self, index_id: &ScaleId, value_id: &ScaleId, x: Scalar) -> bool {
        self.fitting_visible.contains(value_id)
            && self.windows.get(index_id).is_some_and(|(low, high)| x < *low || x > *high)
    }

    /// Whether the chart's scale `id` is radial. A dataset on two radial scales reads its points
    /// as a polar dataset: its x on its x scale and its y on its y scale, even when its controller
    /// is `horizontal`, which swaps their roles instead (see `request_polar_role`).
    pub fn is_radial(&self, id: &ScaleId) -> bool {
        self.radial.contains(id)
    }

    /// Asks for the scale `id` not to draw its grid, its ticks and its labels around a circle, as a
    /// pie draws only its slices. A scale whose grid lines were set (`grid_lines`) keeps drawing
    /// them, with its labels: `.grid_lines(true)` brings a pie's grid back.
    pub fn request_hidden_decoration(&self, id: &ScaleId) {
        self.change(id, |request| request.hidden_decoration = true);
    }

    /// Notes that a lying controller (`horizontal`) has its categories on the radial scale `id`,
    /// out along the radius, so its labels give way to the bars that start beside them (see
    /// `Scales::is_lying`).
    pub(crate) fn request_lying(&self, id: &ScaleId) {
        self.change(id, |request| request.lying = true);
    }

    /// Asks for the radial scale `id` to be the angle or the radius of its polar chart on this
    /// fit. Controllers ask for a polar dataset's x scale to be the angle and its y scale the
    /// radius, or the other way round when `horizontal`. The first role asked for wins; asking for
    /// the other is a conflict, which the chart logs.
    pub fn request_polar_role(&self, id: &ScaleId, role: PolarRole) {
        self.change(id, |request| match request.polar_role {
            None => request.polar_role = Some(role),
            Some(first) if first != role => request.polar_role_conflict = true,
            Some(_) => {}
        });
    }

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

    /// Asks for the rings of the radius scale `id` to be drawn as `shape`, on this fit: line
    /// controllers ask for polygons, the radar convention, and scatter controllers for circles.
    /// Any circle wins. A scale whose grid shape was set explicitly keeps it.
    pub fn request_grid_shape(&self, id: &ScaleId, shape: GridShape) {
        self.change(id, |request| {
            request.grid_shape = match (request.grid_shape, shape) {
                (Some(GridShape::Circle), _) | (_, GridShape::Circle) => Some(GridShape::Circle),
                _ => Some(GridShape::Polygon),
            };
        });
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
    /// The ring shapes the controllers asked for on the last fit, by radius scale.
    grid_shapes: IndexMap<ScaleId, GridShape>,
    /// The circle the polar scales share, fitted inside the chart area on every render; `None`
    /// before the first.
    circle: Option<Circle>,
    /// Each radial scale's role on the last fit.
    roles: IndexMap<ScaleId, PolarRole>,
    /// The radial scales a lying controller's categories ran out along on the last fit.
    lying: HashSet<ScaleId>,
    /// The scales whose grid, ticks and labels aren't drawn (see `set_decoration_hidden`).
    hidden_decoration: HashSet<ScaleId>,
    /// What was already logged about a dataset's pair of scales, so each is logged once.
    warned_pairs: RefCell<HashSet<(&'static str, ScaleId, ScaleId)>>,
}

impl Scales {
    pub fn new() -> Scales {
        let mut scales: IndexMap<ScaleId, Box<dyn Scale>> = IndexMap::new();
        scales.insert(ScaleId::X, Box::new(LinearScale::new(Axis::Horizontal)));
        scales.insert(ScaleId::Y, Box::new(LinearScale::new(Axis::Vertical)));

        Scales { scales, grid_shapes: IndexMap::new(), circle: None, roles: IndexMap::new(), lying: HashSet::new(), hidden_decoration: HashSet::new(), warned_pairs: RefCell::new(HashSet::new()) }
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

    /// The scale with the given id, to change, if the chart has one.
    pub(crate) fn get_mut(&mut self, id: &ScaleId) -> Option<&mut dyn Scale> {
        self.scales.get_mut(id).map(|scale| scale.as_mut())
    }

    pub fn contains(&self, id: &ScaleId) -> bool {
        self.scales.contains_key(id)
    }

    /// Every scale, in the order the chart draws them.
    pub fn iter(&self) -> impl Iterator<Item=(&ScaleId, &dyn Scale)> + '_ {
        self.scales.iter().map(|(id, scale)| (id, scale.as_ref()))
    }

    /// The circle the chart's polar scales share, as fitted for the last render.
    pub fn circle(&self) -> Option<Circle> {
        self.circle
    }

    pub(crate) fn set_circle(&mut self, circle: Option<Circle>) {
        self.circle = circle;
    }

    /// The role of the radial scale `id` on the last fit, or `None` when it isn't radial or no
    /// dataset binds it.
    pub fn polar_role(&self, id: &ScaleId) -> Option<PolarRole> {
        self.roles.get(id).copied()
    }

    /// Whether the scale `id`'s grid, ticks and labels are left out (see
    /// `ScaleBounds::request_hidden_decoration`).
    pub(crate) fn is_decoration_hidden(&self, id: &ScaleId) -> bool {
        self.hidden_decoration.contains(id)
    }

    /// Sets which scales leave out their grid, ticks and labels, on every fit.
    pub(crate) fn set_decoration_hidden(&mut self, ids: HashSet<ScaleId>) {
        self.hidden_decoration = ids;
    }

    /// Whether a lying controller's categories ran out along the radial scale `id` on the last fit.
    pub(crate) fn is_lying(&self, id: &ScaleId) -> bool {
        self.lying.contains(id)
    }

    /// The scales with `role`, in the chart's order.
    pub(crate) fn with_role(&self, role: PolarRole) -> impl Iterator<Item=(&ScaleId, &dyn Scale)> + '_ {
        self.iter().filter(move |(id, _)| self.polar_role(id) == Some(role))
    }

    /// The ids of the radial scales.
    pub(crate) fn radial_ids(&self) -> IndexSet<ScaleId> {
        self.iter().filter(|(_, scale)| scale.axis() == Axis::Radial).map(|(id, _)| id.clone()).collect()
    }

    /// Whether the first angle scale leaves out its spokes and labels (see `is_decoration_hidden`).
    pub(crate) fn first_angle_hidden(&self) -> bool {
        self.with_role(PolarRole::Angle).next().is_some_and(|(id, _)| self.is_decoration_hidden(id))
    }

    /// Whether the first radius scale leaves out its rings, labels and rim.
    pub(crate) fn first_radius_hidden(&self) -> bool {
        self.with_role(PolarRole::Radius).next().is_some_and(|(id, _)| self.is_decoration_hidden(id))
    }

    /// The first angle scale and the first radius scale, in the chart's order, which the circle
    /// is fitted to and whose grid is drawn around it.
    pub(crate) fn first_polar_pair(&self) -> (Option<&dyn Scale>, Option<&dyn Scale>) {
        let first = |role: PolarRole| self.with_role(role).next().map(|(_, scale)| scale);
        (first(PolarRole::Angle), first(PolarRole::Radius))
    }

    /// The shape of the radius scale `id`'s rings: as set on the scale, or as the controllers
    /// asked, or circles.
    pub(crate) fn grid_shape(&self, id: &ScaleId) -> GridShape {
        self.get(id)
            .and_then(|scale| scale.axis_options().grid_shape())
            .or_else(|| self.grid_shapes.get(id).copied())
            .unwrap_or_default()
    }

    /// The mapping onto the chart's circle for a point placed on `angle_id` around it and
    /// `radius_id` out from the centre, or `None` when they aren't the chart's scales in those
    /// roles (see `polar_role`), or no circle was fitted yet.
    pub fn polar(&self, angle_id: &ScaleId, radius_id: &ScaleId) -> Option<Polar> {
        if self.polar_role(angle_id) != Some(PolarRole::Angle) || self.polar_role(radius_id) != Some(PolarRole::Radius) {
            return None;
        }

        Some(Polar::new(self.get(angle_id)?, self.get(radius_id)?, self.circle?))
    }

    /// What lies under `position` on the scale `id`, for a chart area `area`: the value drawn
    /// there, and on a scale with categories the category (see `Scale::category_at`). A scale
    /// along a side reads the position's x or y; a radial scale reads its angle around the circle
    /// or its distance from the centre, as its role says. `None` when the chart has no such scale,
    /// or it is radial without a role or a circle yet. Positions outside the chart area give
    /// values past the scale's ends.
    pub(crate) fn value_at(&self, id: &ScaleId, position: Position, area: Rect, env: &mut Environment) -> Option<ScaleValue> {
        let scale = self.get(id)?;

        let normalized = match scale.axis() {
            Axis::Horizontal => pixel_normalized(Axis::Horizontal, position.x, area),
            Axis::Vertical => pixel_normalized(Axis::Vertical, position.y, area),
            Axis::Radial => {
                // The scale alone decides its own role's mapping: an angle scale's start, end and
                // direction, or a radius scale's inner radius.
                let polar = Polar::new(scale, scale, self.circle?);
                let (angle, distance) = polar.locate(position);
                match self.polar_role(id)? {
                    PolarRole::Angle => polar.normalized_angle(angle),
                    PolarRole::Radius => polar.normalized_distance(distance),
                }
            }
        };

        let value = scale.denormalize(normalized);
        let category = scale.category_at(value).map(|(index, fraction)| CategoryValue {
            index,
            name: scale.label(index, env),
            fraction,
        });

        Some(ScaleValue { value, category })
    }

    /// How a dataset bound to `x_id` and `y_id` is drawn: on a side each, around and out from the
    /// circle, or not at all, which is logged once per pair: when only one of them is radial, or
    /// both have the same role, as when a radial scale is the angle for one dataset and the radius
    /// for another. A scale the chart doesn't have is left to the caller, as before.
    pub(crate) fn system(&self, x_id: &ScaleId, y_id: &ScaleId) -> Option<System> {
        let (x, y) = (self.get(x_id)?.axis(), self.get(y_id)?.axis());

        match (x == Axis::Radial, y == Axis::Radial) {
            (false, false) => return Some(System::Cartesian),
            (true, true) => match (self.polar_role(x_id), self.polar_role(y_id)) {
                (Some(PolarRole::Angle), Some(PolarRole::Radius)) => return Some(System::Polar { x_is_angle: true }),
                (Some(PolarRole::Radius), Some(PolarRole::Angle)) => return Some(System::Polar { x_is_angle: false }),
                _ => {}
            },
            _ => {}
        }

        self.warn_once("mixed", x_id, y_id, || format!("A dataset on scales {:?} ({:?}) and {:?} ({:?}) is not drawn: a polar dataset needs two radial scales, one the angle and one the radius, and a cartesian one a scale along each side", x_id.name(), x, y_id.name(), y));
        None
    }

    /// Logs `message` the first time it is asked for `reason` and the pair `x_id` and `y_id`.
    pub(crate) fn warn_once(&self, reason: &'static str, x_id: &ScaleId, y_id: &ScaleId, message: impl FnOnce() -> String) {
        if self.warned_pairs.borrow_mut().insert((reason, x_id.clone(), y_id.clone())) {
            log::warn!("{}", message());
        }
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
        self.grid_shapes.clear();
        self.roles.clear();
        self.lying.clear();

        for (id, scale) in &mut self.scales {
            let request = bounds.get(id);

            if let Some(shape) = request.grid_shape() {
                self.grid_shapes.insert(id.clone(), shape);
            }

            if request.lying && scale.axis() == Axis::Radial {
                self.lying.insert(id.clone());
            }

            let role = request.polar_role().filter(|_| scale.axis() == Axis::Radial);
            if let Some(role) = role {
                self.roles.insert(id.clone(), role);
            }
            scale.set_polar_role(role);

            let (min, max) = request.range().unwrap_or((0.0, 1.0));

            scale.set_categories(&bounds.categories(id));
            scale.set_smallest_positive(request.smallest_positive());
            scale.set_offset(request.offset());
            scale.set_grid_offset_requested(request.grid_offset());
            scale.set_range(min, max);
        }
    }
}

/// What lies under a position in the chart on one scale (see `Pick`).
#[derive(Clone, Debug, PartialEq)]
pub struct ScaleValue {
    /// The value on the scale: a number, a time in seconds, or a position in categories.
    pub value: Scalar,
    /// The category there, on a scale with categories.
    pub category: Option<CategoryValue>,
}

/// The category under a position, and how far across its band the position is.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryValue {
    /// The category's index, as a value on its scale.
    pub index: Scalar,
    /// The category's name, when the scale has one for it.
    pub name: Option<String>,
    /// How far across the category's band the position is, from 0 at its start to 1 at its end,
    /// in the order the values run, which is the order bars share the band in.
    pub fraction: Scalar,
}

/// The coordinate system a dataset is drawn in, from the axes of the scales it is bound to.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum System {
    /// Along the sides of the chart area.
    Cartesian,
    /// Around the chart's circle and out from its centre, with the x scale the angle or the
    /// radius.
    Polar { x_is_angle: bool },
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

    /// The smallest positive end of any range included is kept, for log scales: a bar's base at
    /// zero doesn't count, its value does, and so does the lower end of a range above zero.
    #[test]
    fn bounds_keep_the_smallest_positive_end() {
        let mut bounds = DataBounds::default();
        assert_eq!(bounds.smallest_positive(), None);

        bounds.include(0.0, 12.0);
        bounds.include(-4.0, 0.0);
        assert_eq!(bounds.smallest_positive(), Some(12.0), "a bar from zero to 12, and a negative one");

        bounds.include(3.0, 40.0);
        assert_eq!(bounds.smallest_positive(), Some(3.0));
        assert_eq!(bounds.range(), Some((-4.0, 40.0)), "the range itself is unchanged");
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

    /// Positions map back to values along each side, and around the circle by each radial
    /// scale's role; categories come with their names and the fraction across their band.
    #[test]
    fn positions_map_back_to_values() {
        use carbide::draw::{Dimension, Position, Rect};
        use crate::scale::Circle;

        let area = Rect::new(Position::new(100.0, 50.0), Dimension::new(400.0, 200.0));
        let mut env = Environment::new();

        let mut scales = Scales::new();
        let mut x = CategoryScale::new(Axis::Horizontal).labels(["A", "B", "C", "D"]);
        x.set_offset(true);
        x.set_range(0.0, 3.0);
        scales.insert(ScaleId::X, x);
        let mut y = LinearScale::new(Axis::Vertical);
        y.set_range(0.0, 100.0);
        scales.insert(ScaleId::Y, y);

        // The second category's band is from 200 to 300 along x; y runs up from 250 to 50.
        let value = scales.value_at(&ScaleId::X, Position::new(225.0, 0.0), area, &mut env).unwrap();
        let category = value.category.unwrap();
        assert_eq!((category.index, category.name.as_deref(), category.fraction), (1.0, Some("B"), 0.25));

        let y = scales.value_at(&ScaleId::Y, Position::new(0.0, 100.0), area, &mut env).unwrap();
        assert!((y.value - 75.0).abs() < 1e-9 && y.category.is_none(), "{y:?}");

        // Around a circle: the angle scale reads the angle, the radius scale the distance.
        let (angle_id, radius_id) = (ScaleId::new("angle"), ScaleId::new("radius"));
        // A fixed range, as a fitted one would round 360 out to 400.
        let mut angle = LinearScale::new(Axis::Radial).min(0.0).max(360.0);
        angle.set_polar_role(Some(PolarRole::Angle));
        angle.set_range(0.0, 360.0);
        let mut radius = LinearScale::new(Axis::Radial);
        radius.set_polar_role(Some(PolarRole::Radius));
        radius.set_range(0.0, 10.0);
        scales.insert(angle_id.clone(), angle);
        scales.insert(radius_id.clone(), radius);
        scales.roles.insert(angle_id.clone(), PolarRole::Angle);
        scales.roles.insert(radius_id.clone(), PolarRole::Radius);
        scales.set_circle(Some(Circle { center: Position::new(300.0, 150.0), radius: 100.0 }));

        // 50 to the right of the centre: a quarter turn, halfway out.
        let at = Position::new(350.0, 150.0);
        let turned = scales.value_at(&angle_id, at, area, &mut env).unwrap().value;
        let out = scales.value_at(&radius_id, at, area, &mut env).unwrap().value;
        assert!((turned - 90.0).abs() < 1e-9 && (out - 5.0).abs() < 1e-9, "{turned}, {out}");
    }
}
