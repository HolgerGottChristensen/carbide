use std::fmt::Debug;
use std::ops::Range;

use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::{AnyReadState, IntoReadState, ReadStateExt};
use dyn_clone::DynClone;

use crate::scale::{Axis, AxisOptions, AxisPosition, PolarRole, Scale, SnapMode};
use crate::sequence::{Sequence, SequenceItem};

/// A category scale's labels, boxed: any sequence of strings (see `CategoryScale::labels`).
trait Labels: DynClone + Debug {
    fn foreach(&self, f: &mut dyn FnMut(usize, &str));

    fn sync(&mut self, env: &mut Environment) -> bool;

    fn count(&self) -> usize;
}

dyn_clone::clone_trait_object!(Labels);

impl<S: Sequence<str>> Labels for S {
    fn foreach(&self, f: &mut dyn FnMut(usize, &str)) {
        Sequence::<str>::foreach(self, &mut |index, text| f(index, text));
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        Sequence::<str>::sync(self, env)
    }

    fn count(&self) -> usize {
        Sequence::<str>::count(self)
    }
}

/// A label is an item of a sequence of labels.
impl SequenceItem<str> for String {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &str)) {
        f(index, self);
    }
}

/// A label is an item of a sequence of labels.
impl SequenceItem<str> for &'static str {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &str)) {
        f(index, self);
    }
}

/// A scale of evenly spaced categories, one tick per index. Data values are category indices: a
/// `Vec<Scalar>` dataset's points are at 0, 1, 2, ..., so with `labels(["Red", "Blue", ...])` index
/// 0 is shown as "Red". Ticks are never fractional, which is what bar charts need: it is their
/// default x scale (see `BarController`).
///
/// - Without labels, the scale spans the data's indices; with labels, it spans every label, even
///   ones without data.
/// - With an offset (see `Scale::set_offset`), half a category is added at both ends.
/// - With a grid offset, grid lines and tick marks sit between categories instead of on them,
///   while labels stay centred on their category. It is on when set with `grid_offset(true)`, or,
///   when not set, when a controller asks for it (see `Scale::set_grid_offset_requested`).
/// - Vertical category scales run from top to bottom, so they read like a list.
/// - Zoomed, the window snaps to whole categories, unless set otherwise with `snap_mode`.
///
/// The labels, the grid offset and the snap mode, like the axis options, can be states, read as they were last
/// synced: the chart refits its scales on every update, so changed labels change the categories
/// the scale spans.
#[derive(Clone, Debug)]
pub struct CategoryScale {
    axis: Axis,
    axis_options: AxisOptions,
    offset: bool,
    /// Set with `grid_offset`; when `None`, the scale follows what controllers request.
    grid_offset: Option<Box<dyn AnyReadState<T=bool>>>,
    grid_offset_requested: bool,
    /// Set with `snap_mode`; when `None`, the window snaps to whole categories.
    snap_mode: Option<Box<dyn AnyReadState<T=SnapMode>>>,
    labels: Box<dyn Labels>,
    /// The categories on the last fit: the labels, then the data's text values, in the order they
    /// were first seen (see `Scale::set_categories`).
    categories: Vec<String>,
    min: Scalar,
    max: Scalar,
    ticks: Vec<Scalar>,
    /// The categories in the zoom window, when the scale shows only some of them (see `zoom`).
    visible_ticks: Option<Vec<Scalar>>,
}

impl CategoryScale {
    pub fn new(axis: Axis) -> CategoryScale {
        let mut scale = CategoryScale {
            axis,
            axis_options: AxisOptions::new(axis),
            offset: false,
            grid_offset: None,
            grid_offset_requested: false,
            snap_mode: None,
            labels: Box::new(Vec::<String>::new()),
            categories: vec![],
            min: 0.0,
            max: 0.0,
            ticks: vec![],
            visible_ticks: None,
        };

        scale.recalculate_ticks();
        scale
    }

    /// The names of the categories, in index order. The tick for index `i` shows label `i`;
    /// indices without a label show their number.
    ///
    /// The labels are a sequence of strings, as a dataset is a sequence of points: an array or
    /// `Vec` of `&'static str` or `String`, or a state holding a collection of them, such as a
    /// `LocalState<Vec<String>>` or a state mapped from a model, so the categories can change
    /// after the chart is built.
    pub fn labels(mut self, labels: impl Sequence<str>) -> CategoryScale {
        self.labels = Box::new(labels);
        self
    }

    /// Draws grid lines and tick marks between categories instead of through them, or through
    /// them with `false`; a value or a state of one. Without it, the scale follows the
    /// controllers drawn on it: bar charts ask for grid lines between categories, so the lines
    /// separate the bar groups.
    pub fn grid_offset(mut self, grid_offset: impl IntoReadState<bool>) -> CategoryScale {
        self.grid_offset = Some(grid_offset.into_read_state().as_dyn_read());
        self
    }

    /// How the zoom window lines up with the categories, a value or a state of one:
    /// `SnapMode::Categories`, the default, snaps it to whole categories, and `SnapMode::Free`
    /// shows it as asked for, so dragging scrolls smoothly. Either way it shows at least two
    /// categories' bands with an offset, as bars ask for, and one step between categories without.
    pub fn snap_mode(mut self, snap_mode: impl IntoReadState<SnapMode>) -> CategoryScale {
        self.snap_mode = Some(snap_mode.into_read_state().as_dyn_read());
        self
    }

    /// How the zoom window lines up with the categories: as set with `snap_mode`, or else
    /// snapped to them.
    fn snap(&self) -> SnapMode {
        self.snap_mode.as_ref().map_or(SnapMode::Categories, |snap_mode| *snap_mode.value_dyn())
    }

    /// Whether grid lines and tick marks sit between the categories: as set with `grid_offset`,
    /// or else as the controllers asked.
    fn has_grid_offset(&self) -> bool {
        match &self.grid_offset {
            Some(grid_offset) => *grid_offset.value_dyn(),
            None => self.grid_offset_requested,
        }
    }

    /// See `LinearScale::position`.
    pub fn position(mut self, position: impl IntoReadState<AxisPosition>) -> CategoryScale {
        self.axis_options = self.axis_options.with_position(position);
        self
    }

    /// See `LinearScale::weight`.
    pub fn weight(mut self, weight: impl IntoReadState<i32>) -> CategoryScale {
        self.axis_options = self.axis_options.with_weight(weight);
        self
    }

    /// See `LinearScale::grid_lines`.
    pub fn grid_lines(mut self, grid_lines: impl IntoReadState<bool>) -> CategoryScale {
        self.axis_options = self.axis_options.with_grid_lines(grid_lines);
        self
    }

    /// See `LinearScale::tick_marks`.
    pub fn tick_marks(mut self, tick_marks: impl IntoReadState<bool>) -> CategoryScale {
        self.axis_options = self.axis_options.with_tick_marks(tick_marks);
        self
    }

    /// See `LinearScale::title`.
    pub fn title(mut self, title: impl IntoReadState<String>) -> CategoryScale {
        self.axis_options = self.axis_options.with_title(title);
        self
    }

    crate::scale::axis_options::polar_options!(CategoryScale);
    crate::scale::axis_options::zoom_options!(CategoryScale);

    /// The value at the start of the mapped range and the range's length. Around a full circle
    /// the range is one category longer, so the last category sits one step before the first
    /// instead of on it.
    fn start_and_length(&self) -> (Scalar, Scalar) {
        let count = self.max - self.min + 1.0;
        let wraps = self.axis_options.polar_role() == Some(PolarRole::Angle) && self.axis_options.is_full_circle();
        let length = (count - if self.offset || wraps { 0.0 } else { 1.0 }).max(1.0);
        let start = self.min - if self.offset { 0.5 } else { 0.0 };

        (start, length)
    }
}

impl Scale for CategoryScale {
    fn axis(&self) -> Axis {
        self.axis
    }

    fn axis_options(&self) -> &AxisOptions {
        &self.axis_options
    }

    fn axis_options_mut(&mut self) -> Option<&mut AxisOptions> {
        Some(&mut self.axis_options)
    }

    fn set_polar_role(&mut self, role: Option<crate::scale::PolarRole>) {
        self.axis_options.set_polar_role(role);
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = self.axis_options.sync(env);
        changed |= self.labels.sync(env);
        changed |= self.grid_offset.as_mut().is_some_and(|grid_offset| grid_offset.sync(env));
        changed |= self.snap_mode.as_mut().is_some_and(|snap_mode| snap_mode.sync(env));
        changed
    }

    fn normalize(&self, value: Scalar) -> Scalar {
        let (start, length) = self.start_and_length();
        let decimal = self.axis_options.window().zoom((value - start) / length);

        // Vertical category scales are reversed, so the first category is at the top. A
        // normalized 1 is the top of the chart area.
        match self.axis {
            Axis::Vertical => 1.0 - decimal,
            Axis::Horizontal | Axis::Radial => decimal,
        }
    }

    fn is_reversed(&self) -> bool {
        self.axis == Axis::Vertical
    }

    fn denormalize(&self, normalized: Scalar) -> Scalar {
        let (start, length) = self.start_and_length();
        let decimal = match self.axis {
            Axis::Vertical => 1.0 - normalized,
            Axis::Horizontal | Axis::Radial => normalized,
        };

        start + self.axis_options.window().unzoom(decimal) * length
    }

    /// The nearest category, kept within the first and the last. Around a full circle, where the
    /// last category's band wraps round to the first's, a value past the last half band belongs
    /// to the first category.
    fn category_at(&self, value: Scalar) -> Option<(Scalar, Scalar)> {
        if !value.is_finite() {
            return None;
        }

        let count = self.max - self.min + 1.0;
        let wraps = !self.offset && self.axis_options.polar_role() == Some(PolarRole::Angle) && self.axis_options.is_full_circle();

        let nearest = value.round();
        let index = if wraps {
            self.min + (nearest - self.min).rem_euclid(count)
        } else {
            nearest.clamp(self.min, self.max)
        };
        let fraction = (value - (nearest - 0.5)).clamp(0.0, 1.0);
        let fraction = if wraps || index == nearest { fraction } else if value < index { 0.0 } else { 1.0 };

        Some((index, fraction))
    }

    fn set_offset(&mut self, offset: bool) {
        self.offset = offset;
    }

    fn set_grid_offset_requested(&mut self, requested: bool) {
        self.grid_offset_requested = requested;
    }

    fn categories(&self, f: &mut dyn FnMut(&str)) {
        self.labels.foreach(&mut |_index, text| f(text));
    }

    fn set_categories(&mut self, categories: &[String]) {
        self.categories = categories.to_vec();
    }

    /// The category's index: among the categories of the last fit, or, before the first, among
    /// the labels.
    fn has_categories(&self) -> bool {
        true
    }

    fn category_index(&self, text: &str) -> Option<Scalar> {
        if let Some(index) = self.categories.iter().position(|category| category == text) {
            return Some(index as Scalar);
        }

        let mut found = None;
        self.labels.foreach(&mut |index, label| {
            if found.is_none() && label == text {
                found = Some(index as Scalar);
            }
        });
        found
    }

    /// The category's name, for an index with a label or a category the data named; other
    /// indices show their number.
    fn label(&self, value: Scalar, _env: &mut Environment) -> Option<String> {
        if value < 0.0 {
            return None;
        }

        if let Some(category) = self.categories.get(value as usize) {
            return Some(category.clone());
        }

        let wanted = value as usize;
        let mut label = None;
        self.labels.foreach(&mut |index, text| {
            if index == wanted {
                label = Some(text.to_string());
            }
        });

        label
    }

    fn grid_line_values(&self) -> Vec<Scalar> {
        let ticks = self.ticks();
        if !self.has_grid_offset() {
            return ticks.to_vec();
        }

        // Between each pair of neighbouring categories, plus one before the first and one after
        // the last; the axis skips those that fall outside the chart area.
        let mut values: Vec<Scalar> = ticks.iter().map(|tick| tick - 0.5).collect();
        if let Some(last) = ticks.last() {
            values.push(last + 0.5);
        }

        values
    }

    fn min(&self) -> Scalar {
        self.min
    }

    fn max(&self) -> Scalar {
        self.max
    }

    /// Fits the scale to the data's indices, rounded to whole categories; with labels or
    /// categories named by the data, to every one of them instead.
    fn set_range(&mut self, min: Scalar, max: Scalar) {
        let label_count = self.labels.count().max(self.categories.len());

        if label_count == 0 {
            let min = if min.is_finite() { min.round().max(0.0) } else { 0.0 };
            let max = if max.is_finite() { max.round() } else { min };

            self.min = min;
            self.max = max.max(min);
        } else {
            self.min = 0.0;
            self.max = (label_count - 1) as Scalar;
        }

        self.recalculate_ticks();
    }

    /// Every category is a tick. Zoomed, the window snaps to whole categories, unless the snap
    /// mode is free: to the edges of their bands with an offset, so bars are never cut, or to the
    /// categories themselves without one. It shows at least two bands with an offset, and one step
    /// between categories without; the ticks shown are the categories inside it.
    fn recalculate_ticks(&mut self) {
        let count = (self.max - self.min) as usize + 1;
        self.ticks = (0..count).map(|index| self.min + index as Scalar).collect();

        let (start, length) = self.start_and_length();
        let min_steps = if self.offset { 2.0 } else { 1.0 };
        let window = match self.snap() {
            SnapMode::Categories => self.axis_options.zoom().snapped(length, min_steps),
            SnapMode::Free => self.axis_options.zoom().limited(min_steps / length),
        };
        self.axis_options.set_window(window);
        self.axis_options.set_min_span((min_steps / length).min(1.0));

        self.visible_ticks = (!window.is_whole()).then(|| {
            let (low, high) = (start + window.start * length, start + window.end * length);
            self.ticks.iter().copied().filter(|tick| *tick >= low - 1e-9 && *tick <= high + 1e-9).collect()
        });
    }

    fn ticks(&self) -> &[Scalar] {
        self.visible_ticks.as_deref().unwrap_or(&self.ticks)
    }

    fn range(&self) -> Range<Scalar> {
        self.min..self.max
    }
}

#[cfg(test)]
mod tests {
    use carbide::environment::Environment;
    use carbide::state::{LocalState, State};
    use carbide::draw::Scalar;
    use crate::scale::{Axis, Scale, SnapMode};
    use super::CategoryScale;

    /// The labels can be a state: after a sync, the scale names its ticks with the new labels,
    /// and a refit spans as many categories as there are labels now.
    #[test]
    fn labels_can_be_a_state() {
        let mut labels = LocalState::new(vec!["Low".to_string(), "High".to_string()]);
        let mut scale = CategoryScale::new(Axis::Horizontal).labels(labels.clone());
        scale.sync(&mut Environment::new());
        scale.set_range(0.0, 0.0);
        assert_eq!((scale.min(), scale.max()), (0.0, 1.0));
        assert_eq!(scale.label(1.0, &mut Environment::new()).as_deref(), Some("High"));

        labels.value_mut().insert(1, "Medium".to_string());
        scale.sync(&mut Environment::new());
        scale.set_range(0.0, 0.0);
        assert_eq!((scale.min(), scale.max()), (0.0, 2.0), "three labels span three categories");
        assert_eq!(scale.label(1.0, &mut Environment::new()).as_deref(), Some("Medium"));
        assert_eq!(scale.label(2.0, &mut Environment::new()).as_deref(), Some("High"));
        assert_eq!(scale.label(3.0, &mut Environment::new()), None, "an index past the labels has none");
    }

    /// The grid offset can be a state, and set either way it wins over what controllers ask for.
    #[test]
    fn grid_offset_can_be_a_state() {
        let mut grid_offset = LocalState::new(false);
        let mut scale = CategoryScale::new(Axis::Horizontal).grid_offset(grid_offset.clone());
        scale.set_grid_offset_requested(true);
        scale.sync(&mut Environment::new());
        scale.set_range(0.0, 2.0);
        assert_eq!(scale.grid_line_values(), vec![0.0, 1.0, 2.0], "set to false, the lines go through the categories");

        *grid_offset.value_mut() = true;
        scale.sync(&mut Environment::new());
        assert_eq!(scale.grid_line_values(), vec![-0.5, 0.5, 1.5, 2.5]);
    }

    /// One tick per index, never fractional: the case that gave `LinearScale` ticks every 0.5.
    #[test]
    fn ticks_are_whole_categories() {
        let mut scale = CategoryScale::new(Axis::Horizontal);
        scale.set_range(0.0, 5.0);

        assert_eq!(scale.ticks(), &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    /// Without an offset the first and last categories sit on the edges; with one, half a
    /// category is added at both ends: six categories span 0..=5 or -0.5..=5.5.
    #[test]
    fn offset_adds_half_a_category_at_both_ends() {
        let mut scale = CategoryScale::new(Axis::Horizontal);
        scale.set_range(0.0, 5.0);
        assert_eq!(scale.normalize(0.0), 0.0);
        assert_eq!(scale.normalize(5.0), 1.0);

        scale.set_offset(true);
        assert!((scale.normalize(0.0) - 0.5 / 6.0).abs() < 1e-12);
        assert!((scale.normalize(5.0) - 5.5 / 6.0).abs() < 1e-12);
    }

    /// Labels name the categories by index, and the scale spans every label even when the data
    /// covers only some of them.
    #[test]
    fn labels_name_categories_and_set_the_span() {
        let mut scale = CategoryScale::new(Axis::Horizontal).labels(["Red", "Blue", "Yellow"]);
        scale.set_range(0.0, 1.0);

        assert_eq!(scale.ticks(), &[0.0, 1.0, 2.0]);

        assert_eq!(scale.label(0.0, &mut Environment::new()).as_deref(), Some("Red"));
        assert_eq!(scale.label(2.0, &mut Environment::new()).as_deref(), Some("Yellow"));
        assert_eq!(scale.label(3.0, &mut Environment::new()), None, "an index without a label shows its number");
    }

    /// With `grid_offset`, grid lines sit between categories and one past each end; with an
    /// offset those outer lines land exactly on the chart area's edges.
    #[test]
    fn grid_offset_puts_lines_between_categories() {
        let mut scale = CategoryScale::new(Axis::Horizontal).grid_offset(true);
        scale.set_offset(true);
        scale.set_range(0.0, 2.0);

        assert_eq!(scale.grid_line_values(), vec![-0.5, 0.5, 1.5, 2.5]);
        assert_eq!(scale.normalize(-0.5), 0.0);
        assert_eq!(scale.normalize(2.5), 1.0);
    }

    /// Vertical category scales run top to bottom: the first category maps to the top (normalized
    /// 1).
    #[test]
    fn vertical_categories_run_top_to_bottom() {
        let mut scale = CategoryScale::new(Axis::Vertical);
        scale.set_range(0.0, 3.0);

        assert_eq!(scale.normalize(0.0), 1.0);
        assert_eq!(scale.normalize(3.0), 0.0);
    }

    /// `denormalize` undoes `normalize`, also on a vertical scale, which runs from the top, and
    /// with the offset bars ask for.
    #[test]
    fn denormalize_is_the_inverse_of_normalize() {
        for axis in [Axis::Horizontal, Axis::Vertical] {
            for offset in [false, true] {
                let mut scale = CategoryScale::new(axis).labels(["A", "B", "C", "D"]);
                scale.set_offset(offset);
                scale.set_range(0.0, 3.0);

                for value in [-0.5, 0.0, 1.25, 3.0, 3.5] {
                    assert!((scale.denormalize(scale.normalize(value)) - value).abs() < 1e-9, "{value} on {axis:?}, offset {offset}");
                }
            }
        }
    }

    /// A value is in the nearest category, a fraction of the way across its band in the order the
    /// values run; past the first or last category it stays in that one, at the band's end.
    #[test]
    fn values_fall_in_the_nearest_category() {
        let mut scale = CategoryScale::new(Axis::Horizontal).labels(["A", "B", "C"]);
        scale.set_offset(true);
        scale.set_range(0.0, 2.0);

        assert_eq!(scale.category_at(1.0), Some((1.0, 0.5)), "on a category, halfway across its band");
        assert_eq!(scale.category_at(0.75), Some((1.0, 0.25)));
        let (index, fraction) = scale.category_at(1.4).unwrap();
        assert!(index == 1.0 && (fraction - 0.9).abs() < 1e-9, "near the band's end: {index}, {fraction}");
        assert_eq!(scale.category_at(-0.5), Some((0.0, 0.0)), "the start of the first band");
        assert_eq!(scale.category_at(-3.0), Some((0.0, 0.0)), "before the first category, its band's start");
        assert_eq!(scale.category_at(9.0), Some((2.0, 1.0)), "after the last, its band's end");
        assert_eq!(scale.label(scale.category_at(1.9).unwrap().0, &mut Environment::new()).as_deref(), Some("C"));
    }

    /// Around a full circle without an offset, the last category's band wraps round to the first
    /// category: a value just before the end belongs to the first.
    #[test]
    fn around_a_circle_the_last_band_wraps_to_the_first() {
        let mut scale = CategoryScale::new(Axis::Radial).labels(["N", "E", "S", "W"]);
        scale.set_polar_role(Some(crate::scale::PolarRole::Angle));
        scale.set_range(0.0, 3.0);

        assert_eq!(scale.category_at(scale.denormalize(0.0)), Some((0.0, 0.5)), "the top is the first category's middle");
        assert_eq!(scale.category_at(scale.denormalize(0.25)), Some((1.0, 0.5)), "a quarter round, the second");

        let (index, fraction) = scale.category_at(scale.denormalize(0.95)).unwrap();
        assert_eq!(index, 0.0, "just before a full turn is the first category");
        assert!((fraction - 0.3).abs() < 1e-9, "its band started an eighth of a turn before the top: {fraction}");
    }

    /// A zoomed category scale snaps its window to whole categories: with an offset, to the
    /// edges of their bands, so bars aren't cut, showing at least two; its ticks are the
    /// categories shown. On a vertical scale the window runs from the first category, at the top.
    #[test]
    fn zoom_snaps_to_whole_categories() {
        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct"];

        let mut scale = CategoryScale::new(Axis::Horizontal).labels(months).zoom(crate::ZoomWindow::new(0.23, 0.48));
        scale.set_offset(true);
        scale.set_range(0.0, 9.0);
        assert_eq!(scale.axis_options().window(), crate::ZoomWindow::new(0.2, 0.5), "from Mar's band to Jun's band");
        assert_eq!(scale.ticks(), &[2.0, 3.0, 4.0]);
        assert_eq!(scale.normalize(1.5), 0.0, "the window starts at Mar's band");

        let mut narrow = CategoryScale::new(Axis::Horizontal).labels(months).zoom(crate::ZoomWindow::new(0.51, 0.52));
        narrow.set_offset(true);
        narrow.set_range(0.0, 9.0);
        assert_eq!(narrow.ticks(), &[4.0, 5.0], "at least two categories");

        let mut vertical = CategoryScale::new(Axis::Vertical).labels(months).zoom(crate::ZoomWindow::new(0.0, 0.3));
        vertical.set_offset(true);
        vertical.set_range(0.0, 9.0);
        assert_eq!(vertical.ticks(), &[0.0, 1.0, 2.0]);
        assert_eq!(vertical.normalize(-0.5), 1.0, "Jan's band starts at the top");
        assert_eq!(vertical.normalize(2.5), 0.0);
    }

    /// In the free snap mode, a zoomed category scale shows its window as asked for, cutting the
    /// categories at its ends, and still at least two categories' bands wide; its ticks are the
    /// categories whose middles are inside it. The snap mode can be a state: snapping again after
    /// a sync and a refit.
    #[test]
    fn free_zoom_does_not_snap() {
        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct"];
        let close = |a: Scalar, b: Scalar| (a - b).abs() < 1e-9;

        let mut snap_mode = LocalState::new(SnapMode::Free);
        let mut scale = CategoryScale::new(Axis::Horizontal).labels(months).snap_mode(snap_mode.clone()).zoom(crate::ZoomWindow::new(0.23, 0.48));
        scale.sync(&mut Environment::new());
        scale.set_offset(true);
        scale.set_range(0.0, 9.0);
        assert_eq!(scale.axis_options().window(), crate::ZoomWindow::new(0.23, 0.48), "as asked for");
        assert_eq!(scale.ticks(), &[2.0, 3.0, 4.0], "Mar to May, whose middles are inside");
        assert!(close(scale.normalize(1.8), 0.0), "the window starts inside Mar's band: {}", scale.normalize(1.8));

        let mut narrow = CategoryScale::new(Axis::Horizontal).labels(months).snap_mode(SnapMode::Free).zoom(crate::ZoomWindow::new(0.51, 0.52));
        narrow.sync(&mut Environment::new());
        narrow.set_offset(true);
        narrow.set_range(0.0, 9.0);
        assert!(close(narrow.axis_options().window().span(), 0.2), "widened to two bands: {:?}", narrow.axis_options().window());
        assert_eq!(narrow.ticks(), &[4.0, 5.0]);

        *snap_mode.value_mut() = SnapMode::Categories;
        scale.sync(&mut Environment::new());
        scale.set_range(0.0, 9.0);
        assert_eq!(scale.axis_options().window(), crate::ZoomWindow::new(0.2, 0.5), "snapped again");
    }
}
