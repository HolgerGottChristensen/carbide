use std::ops::Range;

use carbide::draw::Scalar;
use carbide::environment::Environment;

use crate::scale::{format_tick, Axis, AxisPosition, Scale};

/// A scale of evenly spaced categories, one tick per index. Data values are category indices: a
/// `Vec<Scalar>` dataset's points are at 0, 1, 2, ..., so with `labels(["Red", "Blue", ...])` index
/// 0 is shown as "Red". Ticks are never fractional, which is what bar charts need: it is their
/// default x scale (see `BarController`).
///
/// - Without labels, the scale spans the data's indices; with labels, it spans every label, even
///   ones without data.
/// - With an offset (see `Scale::set_offset`), half a category is added at both ends.
/// - With `grid_offset`, grid lines and tick marks sit between categories instead of on them,
///   while labels stay centred on their category.
/// - Vertical category scales run from top to bottom, so they read like a list.
#[derive(Clone, Debug)]
pub struct CategoryScale {
    axis: Axis,
    position: AxisPosition,
    weight: i32,
    grid_lines: bool,
    tick_marks: bool,
    offset: bool,
    grid_offset: bool,
    labels: Vec<String>,
    min: Scalar,
    max: Scalar,
    ticks: Vec<Scalar>,
}

impl CategoryScale {
    pub fn new(axis: Axis) -> CategoryScale {
        let mut scale = CategoryScale {
            axis,
            position: axis.default_position(),
            weight: 0,
            grid_lines: true,
            tick_marks: true,
            offset: false,
            grid_offset: false,
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

    /// Draws grid lines and tick marks between categories instead of through them. Bar charts'
    /// default category scale turns it on, so the lines separate the bar groups.
    pub fn grid_offset(mut self, grid_offset: bool) -> CategoryScale {
        self.grid_offset = grid_offset;
        self
    }

    /// See `LinearScale::position`.
    pub fn position(mut self, position: AxisPosition) -> CategoryScale {
        if self.axis.allows(position) {
            self.position = position;
        } else {
            log::warn!("A {:?} scale can't be positioned {:?}; keeping {:?}", self.axis, position, self.position);
        }

        self
    }

    /// See `Scale::weight`.
    pub fn weight(mut self, weight: i32) -> CategoryScale {
        self.weight = weight;
        self
    }

    /// See `LinearScale::grid_lines`.
    pub fn grid_lines(mut self, grid_lines: bool) -> CategoryScale {
        self.grid_lines = grid_lines;
        self
    }

    /// See `LinearScale::tick_marks`.
    pub fn tick_marks(mut self, tick_marks: bool) -> CategoryScale {
        self.tick_marks = tick_marks;
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

    fn axis_position(&self) -> AxisPosition {
        self.position
    }

    fn weight(&self) -> i32 {
        self.weight
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

    fn tick_label(&self, value: Scalar, env: &mut Environment) -> String {
        if value >= 0.0 {
            if let Some(label) = self.labels.get(value as usize) {
                return label.clone();
            }
        }

        format_tick(value, env)
    }

    fn grid_line_values(&self) -> Vec<Scalar> {
        if !self.grid_offset {
            return self.ticks.clone();
        }

        // Between each pair of neighbouring categories, plus one before the first and one after
        // the last; `draw_grid` skips those that fall outside the chart area.
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

    fn display_ticks(&self) -> bool {
        self.tick_marks
    }

    fn display_grid(&self) -> bool {
        self.grid_lines
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
    use carbide::environment::Environment;
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

        let mut env = Environment::new();
        assert_eq!(scale.tick_label(0.0, &mut env), "Red");
        assert_eq!(scale.tick_label(2.0, &mut env), "Yellow");
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
