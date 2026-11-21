use carbide::draw::Scalar;
use crate::dataset::RangePoint;
use crate::mark::{BarCanvas, BarMark, BarSlots, MarkRole};
use crate::sequence::Sequence;

/// Draws each point as a floating bar from its low to its high value, in place of a bar growing
/// from zero (see `RangeDataSetExt::range`). A point missing its x, low or high isn't drawn.
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
