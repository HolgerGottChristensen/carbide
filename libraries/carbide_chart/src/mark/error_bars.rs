use carbide::draw::{Position, Scalar};
use crate::dataset::RangePoint;
use crate::mark::{BarCanvas, BarMark, BarSlots, LineCanvas, LineMark, Mark, Overlay};
use crate::sequence::Sequence;

/// Draws a whisker from each point's low to its high value, with a cap at each end, on top of the
/// dataset's glyph (see `RangeDataSetExt::error_bars`): through the middle of its bar in a bar
/// controller, and through its point in a line controller. On a stacked bar or line the whisker
/// moves with it: it keeps its distance from the point's value, around where the bar ends or the
/// line is drawn. The whiskers are drawn in the chart's label colour, so they show on bars of any colour. A
/// point missing its x, y, low or high has no whisker, as a missing value has no bar either.
#[derive(Copy, Clone, Debug)]
pub struct ErrorBars {
    /// How wide the caps are on bars, as a share of the bar's thickness.
    pub cap_percentage: Scalar,
    /// How wide the caps are on lines, in pixels, about as wide as a line's points.
    pub line_cap_length: Scalar,
    /// How wide the lines are, in pixels.
    pub line_width: Scalar,
}

impl Default for ErrorBars {
    fn default() -> ErrorBars {
        ErrorBars { cap_percentage: 0.4, line_cap_length: 8.0, line_width: 2.0 }
    }
}

impl Mark for ErrorBars {
    type Role = Overlay;
}

impl<D: Sequence<dyn RangePoint>> BarMark<D> for ErrorBars {
    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(_), Some(low), Some(high)) = (point.x(), point.y(), point.low(), point.high()) {
                f(x, low.min(high), low.max(high));
            }
        });
    }

    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
        canvas.begin_lines();

        data.foreach(&mut |_index, point| {
            let (Some(x), Some(_), Some(low), Some(high)) = (point.x(), point.y(), point.low(), point.high()) else {
                return;
            };

            let slot = slots.slot(x);
            let middle = slot.middle();
            let cap = slot.thickness * self.cap_percentage / 2.0;
            let (low, high) = (canvas.value_pixel(low + slot.stack_offset), canvas.value_pixel(high + slot.stack_offset));

            canvas.add_line(Position::new(middle, low), Position::new(middle, high));
            canvas.add_line(Position::new(middle - cap, low), Position::new(middle + cap, low));
            canvas.add_line(Position::new(middle - cap, high), Position::new(middle + cap, high));
        });

        let ink = canvas.ink();
        canvas.stroke_lines_added(self.line_width, ink);
    }
}

/// Whiskers through a line's points, each the same length across, on top of every dataset's lines
/// and points.
impl<D: Sequence<dyn RangePoint>> LineMark<D> for ErrorBars {
    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(_), Some(low), Some(high)) = (point.x(), point.y(), point.low(), point.high()) {
                f(x, low.min(high), low.max(high));
            }
        });
    }

    fn draw(&self, data: &D, canvas: &mut LineCanvas) {
        let cap = self.line_cap_length / 2.0;
        canvas.begin_lines();

        data.foreach(&mut |_index, point| {
            let (Some(x), Some(_), Some(low), Some(high)) = (point.x(), point.y(), point.low(), point.high()) else {
                return;
            };

            let offset = canvas.stack_offset(x);
            let (low, high) = (canvas.position(x, low + offset), canvas.position(x, high + offset));

            canvas.add_line(low, high);
            canvas.add_line(Position::new(low.x - cap, low.y), Position::new(low.x + cap, low.y));
            canvas.add_line(Position::new(high.x - cap, high.y), Position::new(high.x + cap, high.y));
        });

        let ink = canvas.ink();
        canvas.stroke_lines_added(self.line_width, ink);
    }
}
