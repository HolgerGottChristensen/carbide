use carbide::draw::{Position, Scalar};
use crate::dataset::RangePoint;
use crate::mark::{BarCanvas, BarMark, BarSlots, MarkRole};
use crate::sequence::Sequence;

/// Draws a whisker from each point's low to its high value, through the middle of its bar, with a
/// cap at each end, on top of the bars (see `RangeDataSetExt::error_bars`). On a stacked bar the
/// whisker moves with the bar: it keeps its distance from the point's value, around where the bar
/// ends. The whiskers are drawn in the chart's label colour, so they show on bars of any colour. A
/// point missing its x, y, low or high has no whisker, as a missing value has no bar either.
#[derive(Copy, Clone, Debug)]
pub struct ErrorBars {
    /// How wide the caps are, as a share of the bar's thickness.
    pub cap_percentage: Scalar,
    /// How wide the lines are, in pixels.
    pub line_width: Scalar,
}

impl Default for ErrorBars {
    fn default() -> ErrorBars {
        ErrorBars { cap_percentage: 0.4, line_width: 1.5 }
    }
}

impl<D: Sequence<dyn RangePoint>> BarMark<D> for ErrorBars {
    fn role(&self) -> MarkRole {
        MarkRole::Overlay
    }

    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(_), Some(low), Some(high)) = (point.x(), point.y(), point.low(), point.high()) {
                f(x, low.min(high), low.max(high));
            }
        });
    }

    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
        let mut segments = vec![];

        data.foreach(&mut |_index, point| {
            let (Some(x), Some(_), Some(low), Some(high)) = (point.x(), point.y(), point.low(), point.high()) else {
                return;
            };

            let slot = slots.slot(x);
            let middle = slot.middle();
            let cap = slot.thickness * self.cap_percentage / 2.0;
            let (low, high) = (canvas.value_pixel(low + slot.stack_offset), canvas.value_pixel(high + slot.stack_offset));

            segments.push((Position::new(middle, low), Position::new(middle, high)));
            segments.push((Position::new(middle - cap, low), Position::new(middle + cap, low)));
            segments.push((Position::new(middle - cap, high), Position::new(middle + cap, high)));
        });

        let ink = canvas.ink();
        canvas.stroke_lines(&segments, self.line_width, ink);
    }
}
