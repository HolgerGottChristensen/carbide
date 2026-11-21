use carbide::draw::Scalar;
use crate::dataset::RangePoint;
use crate::mark::{BarCanvas, BarMark, BarSlots, LineCanvas, LineMark, MarkRole};
use crate::sequence::Sequence;

/// Draws a dataset's ranges, from each point's low to its high value (see `RangeDataSetExt::range`):
/// in a bar controller, each point as a floating bar in place of a bar growing from zero; in a
/// line controller, a band between the line through the lows and the line through the highs, in
/// place of the line. A point missing its x, low or high isn't drawn, and leaves a gap in a band.
#[derive(Copy, Clone, Debug, Default)]
pub struct RangeMark;

impl<D: Sequence<dyn RangePoint>> BarMark<D> for RangeMark {
    fn role(&self) -> MarkRole {
        MarkRole::Primary
    }

    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(low), Some(high)) = (point.x(), point.low(), point.high()) {
                f(x, low.min(high), low.max(high));
            }
        });
    }

    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
        data.foreach(&mut |point_index, point| {
            let (Some(x), Some(low), Some(high)) = (point.x(), point.low(), point.high()) else {
                return;
            };

            let slot = slots.slot(x);
            let color = canvas.point_color(point_index, point.color());
            let (from, to) = (canvas.value_pixel(low), canvas.value_pixel(high));
            canvas.fill_rect(slot.side, slot.thickness, from, to, color);
        });
    }
}

/// A band in place of the line: filled between the lows and the highs, translucent, with both
/// edges stroked in the dataset's colour. The edges follow the dataset's `interpolation`, and the
/// band breaks at missing points as the dataset's `span_gaps` says, as a line does.
impl<D: Sequence<dyn RangePoint>> LineMark<D> for RangeMark {
    fn role(&self) -> MarkRole {
        MarkRole::Primary
    }

    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(low), Some(high)) = (point.x(), point.low(), point.high()) {
                f(x, low.min(high), low.max(high));
            }
        });
    }

    fn draw(&self, data: &D, canvas: &mut LineCanvas) {
        let mut xs = vec![];
        let mut highs = vec![];
        let mut lows = vec![];

        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(low), Some(high)) = (point.x(), point.low(), point.high()) {
                xs.push(Some(x));
                highs.push(canvas.position(x, high));
                lows.push(canvas.position(x, low));
            } else {
                xs.push(None);
            }
        });

        let color = canvas.color();
        let fill = canvas.translucent(color);
        let runs = canvas.runs(&xs);

        for run in &runs {
            canvas.fill_between(&highs[run.clone()], &lows[run.clone()], fill);
        }

        for run in &runs {
            canvas.stroke_line(&highs[run.clone()], color);
            canvas.stroke_line(&lows[run.clone()], color);
        }
    }
}
