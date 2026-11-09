use carbide::draw::Scalar;
use indexmap::{IndexMap, IndexSet};

use crate::scale::{Axis, LinearScale, Scale, ScaleId};

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

/// The bounds collected for each scale id. Controllers report their datasets' ranges into it
/// with `DatasetController::collect_bounds`, under the ids the datasets are bound to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScaleBounds {
    bounds: IndexMap<ScaleId, DataBounds>,
    offsets: IndexSet<ScaleId>,
}

impl ScaleBounds {
    /// Asks for the scale `id` to be offset on this fit (see `Scale::set_offset`). Bar
    /// controllers request it for the scale their bars are spread along.
    pub fn request_offset(&mut self, id: ScaleId) {
        self.offsets.insert(id);
    }

    /// Whether any controller requested an offset for the scale `id` on this fit.
    pub fn offset_requested(&self, id: ScaleId) -> bool {
        self.offsets.contains(&id)
    }

    /// Grows the bounds of the scale `id` to include `min..=max`.
    pub fn include(&mut self, id: ScaleId, min: Scalar, max: Scalar) {
        self.bounds.entry(id).or_default().include(min, max);
    }

    /// The bounds collected for the scale `id`; empty when nothing was reported for it.
    pub fn get(&self, id: ScaleId) -> DataBounds {
        self.bounds.get(&id).copied().unwrap_or_default()
    }

    /// Every scale id something was reported for, in the order they were first reported.
    pub fn ids(&self) -> impl Iterator<Item=ScaleId> + '_ {
        self.bounds.keys().copied()
    }
}

/// The scales a chart owns, keyed by id, in insertion order: the order the chart draws and lays
/// them out in. For now only the default `ScaleId::X` and `ScaleId::Y` scales; custom scales come
/// later (see `docs/multiple-controllers-and-scales.md`).
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
    pub fn get(&self, id: ScaleId) -> Option<&dyn Scale> {
        self.scales.get(&id).map(|scale| scale.as_ref())
    }

    pub fn contains(&self, id: ScaleId) -> bool {
        self.scales.contains_key(&id)
    }

    /// Every scale, in the order the chart draws them.
    pub fn iter(&self) -> impl Iterator<Item=(ScaleId, &dyn Scale)> + '_ {
        self.scales.iter().map(|(id, scale)| (*id, scale.as_ref()))
    }

    /// Sets each scale's range to the bounds collected for its id, and its offset to whether one
    /// was requested. A scale without any reported data gets the range `0..1`.
    pub fn fit(&mut self, bounds: &ScaleBounds) {
        for (id, scale) in &mut self.scales {
            let (min, max) = bounds.get(*id).range().unwrap_or((0.0, 1.0));
            scale.set_offset(bounds.offset_requested(*id));
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
        assert_eq!(ScaleBounds::default().get(ScaleId::X).range(), None);
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

        let order: Vec<ScaleId> = scales.iter().map(|(id, _)| id).collect();
        assert_eq!(order, vec![ScaleId::X, ScaleId::Y, ScaleId::new("right")]);
        assert_eq!(scales.get(ScaleId::X).unwrap().axis_position(), AxisPosition::Top);
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
        assert_eq!(horizontal.axis_position(), AxisPosition::Bottom);

        let vertical = LinearScale::new(Axis::Vertical).position(AxisPosition::Top);
        assert_eq!(vertical.axis_position(), AxisPosition::Left);
    }

    #[test]
    fn bounds_are_collected_per_scale_id() {
        let mut bounds = ScaleBounds::default();
        bounds.include(ScaleId::Y, 0.0, 10.0);
        bounds.include(ScaleId::new("right"), 100.0, 200.0);
        bounds.include(ScaleId::Y, -5.0, 5.0);

        assert_eq!(bounds.get(ScaleId::Y).range(), Some((-5.0, 10.0)));
        assert_eq!(bounds.get(ScaleId::new("right")).range(), Some((100.0, 200.0)));
        assert_eq!(bounds.get(ScaleId::X).range(), None);
        assert_eq!(bounds.ids().collect::<Vec<_>>(), vec![ScaleId::Y, ScaleId::new("right")]);
    }
}
