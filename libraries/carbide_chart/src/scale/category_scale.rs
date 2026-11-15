use std::ops::Range;

use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::IntoReadState;

use crate::scale::{Axis, AxisOptions, AxisPosition, Scale};

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
#[derive(Clone, Debug)]
pub struct CategoryScale {
    axis: Axis,
    axis_options: AxisOptions,
    offset: bool,
    /// Set with `grid_offset`; when `None`, the scale follows what controllers request.
    grid_offset: Option<bool>,
    grid_offset_requested: bool,
    labels: Vec<String>,
    min: Scalar,
    max: Scalar,
    ticks: Vec<Scalar>,
}

impl CategoryScale {
    pub fn new(axis: Axis) -> CategoryScale {
        let mut scale = CategoryScale {
            axis,
            axis_options: AxisOptions::new(axis),
            offset: false,
            grid_offset: None,
            grid_offset_requested: false,
            labels: vec![],
            min: 0.0,
            max: 0.0,
            ticks: vec![],
        };

        scale.recalculate_ticks();
        scale
    }

    /// The names of the categories, in index order. The tick for index `i` shows `labels[i]`;
    /// indices without a label show their number.
    pub fn labels<L: Into<String>>(mut self, labels: impl IntoIterator<Item=L>) -> CategoryScale {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Draws grid lines and tick marks between categories instead of through them, or through
    /// them with `false`. Without it, the scale follows the controllers drawn on it: bar charts
    /// ask for grid lines between categories, so the lines separate the bar groups.
    pub fn grid_offset(mut self, grid_offset: bool) -> CategoryScale {
        self.grid_offset = Some(grid_offset);
        self
    }

    /// See `LinearScale::position`.
    pub fn position(mut self, position: AxisPosition) -> CategoryScale {
        self.axis_options = self.axis_options.with_position(position);
        self
    }

    /// See `LinearScale::weight`.
    pub fn weight(mut self, weight: i32) -> CategoryScale {
        self.axis_options = self.axis_options.with_weight(weight);
        self
    }

    /// See `LinearScale::grid_lines`.
    pub fn grid_lines(mut self, grid_lines: bool) -> CategoryScale {
        self.axis_options = self.axis_options.with_grid_lines(grid_lines);
        self
    }

    /// See `LinearScale::tick_marks`.
    pub fn tick_marks(mut self, tick_marks: bool) -> CategoryScale {
        self.axis_options = self.axis_options.with_tick_marks(tick_marks);
        self
    }

    /// See `LinearScale::title`.
    pub fn title(mut self, title: impl IntoReadState<String>) -> CategoryScale {
        self.axis_options = self.axis_options.with_title(title);
        self
    }

    /// The value at the start of the mapped range and the range's length.
    fn start_and_length(&self) -> (Scalar, Scalar) {
        let count = self.max - self.min + 1.0;
        let length = (count - if self.offset { 0.0 } else { 1.0 }).max(1.0);
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

    fn sync(&mut self, env: &mut Environment) -> bool {
        self.axis_options.sync(env)
    }

    fn normalize(&self, value: Scalar) -> Scalar {
        let (start, length) = self.start_and_length();
        let decimal = (value - start) / length;

        // Vertical category scales are reversed, so the first category is at the top. A
        // normalized 1 is the top of the chart area.
        match self.axis {
            Axis::Vertical => 1.0 - decimal,
            Axis::Horizontal | Axis::Radial => decimal,
        }
    }

    fn set_offset(&mut self, offset: bool) {
        self.offset = offset;
    }

    fn set_grid_offset_requested(&mut self, requested: bool) {
        self.grid_offset_requested = requested;
    }

    /// The category's name, for an index with a label; other indices show their number.
    fn label(&self, value: Scalar) -> Option<String> {
        if value >= 0.0 {
            self.labels.get(value as usize).cloned()
        } else {
            None
        }
    }

    fn grid_line_values(&self) -> Vec<Scalar> {
        if !self.grid_offset.unwrap_or(self.grid_offset_requested) {
            return self.ticks.clone();
        }

        // Between each pair of neighbouring categories, plus one before the first and one after
        // the last; the axis skips those that fall outside the chart area.
        let mut values: Vec<Scalar> = self.ticks.iter().map(|tick| tick - 0.5).collect();
        if let Some(last) = self.ticks.last() {
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

    /// Fits the scale to the data's indices, rounded to whole categories; with labels, to every
    /// label instead.
    fn set_range(&mut self, min: Scalar, max: Scalar) {
        if self.labels.is_empty() {
            let min = if min.is_finite() { min.round().max(0.0) } else { 0.0 };
            let max = if max.is_finite() { max.round() } else { min };

            self.min = min;
            self.max = max.max(min);
        } else {
            self.min = 0.0;
            self.max = (self.labels.len() - 1) as Scalar;
        }

        self.recalculate_ticks();
    }

    fn recalculate_ticks(&mut self) {
        let count = (self.max - self.min) as usize + 1;
        self.ticks = (0..count).map(|index| self.min + index as Scalar).collect();
    }

    fn ticks(&self) -> &[Scalar] {
        &self.ticks
    }

    fn range(&self) -> Range<Scalar> {
        self.min..self.max
    }
}

#[cfg(test)]
mod tests {
    use crate::scale::{Axis, Scale};
    use super::CategoryScale;

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

        assert_eq!(scale.label(0.0).as_deref(), Some("Red"));
        assert_eq!(scale.label(2.0).as_deref(), Some("Yellow"));
        assert_eq!(scale.label(3.0), None, "an index without a label shows its number");
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
}
