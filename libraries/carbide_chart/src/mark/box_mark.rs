use carbide::draw::{Position, Scalar};
use crate::dataset::BoxPoint;
use crate::mark::{BarCanvas, BarMark, BarSlots, Mark, Primary};
use crate::sequence::Sequence;

/// Draws each point as a box plot, in place of a bar (see `BoxDataSetExt::box_plot`): a box from
/// the first to the third quartile with a line across it at the median, and a whisker from each
/// end of the box out to the minimum and the maximum, ending in a cap. The box is filled in the
/// point's colour at the `SeriesStyle`'s fill opacity, and its outline, median, whiskers and caps
/// are stroked in the full colour. A point missing any of its values isn't drawn.
#[derive(Copy, Clone, Debug)]
pub struct BoxMark {
    /// How wide the caps are, as a share of the box's thickness.
    pub cap_percentage: Scalar,
    /// How wide the lines are, in pixels.
    pub line_width: Scalar,
}

impl Default for BoxMark {
    fn default() -> BoxMark {
        BoxMark { cap_percentage: 0.5, line_width: 2.0 }
    }
}

impl Mark for BoxMark {
    type Role = Primary;
}

/// The five values of a box point, in order from the smallest, when none is missing.
fn statistics(point: &dyn BoxPoint) -> Option<(Scalar, [Scalar; 5])> {
    Some((point.x()?, [point.min()?, point.q1()?, point.median()?, point.q3()?, point.max()?]))
}

impl<D: Sequence<dyn BoxPoint>> BarMark<D> for BoxMark {
    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let Some((x, values)) = statistics(point) {
                let low = values.iter().copied().fold(Scalar::INFINITY, Scalar::min);
                let high = values.iter().copied().fold(Scalar::NEG_INFINITY, Scalar::max);
                f(x, low, high);
            }
        });
    }

    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
        data.foreach(&mut |point_index, point| {
            let Some((x, [min, q1, median, q3, max])) = statistics(point) else {
                return;
            };

            let slot = slots.slot(x);
            let (side, far_side, middle) = (slot.side, slot.side + slot.thickness, slot.middle());
            let cap = slot.thickness * self.cap_percentage / 2.0;
            let [min, q1, median, q3, max] = [min, q1, median, q3, max].map(|value| canvas.value_pixel(value));

            let color = canvas.point_color(point_index, point.color());
            let fill = canvas.translucent(color);
            canvas.fill_rect(side, slot.thickness, q1, q3, fill);

            let segments = [
                // The box's outline and the median across it.
                (Position::new(side, q1), Position::new(far_side, q1)),
                (Position::new(far_side, q1), Position::new(far_side, q3)),
                (Position::new(far_side, q3), Position::new(side, q3)),
                (Position::new(side, q3), Position::new(side, q1)),
                (Position::new(side, median), Position::new(far_side, median)),
                // The whiskers out to the minimum and the maximum, and their caps.
                (Position::new(middle, q1), Position::new(middle, min)),
                (Position::new(middle, q3), Position::new(middle, max)),
                (Position::new(middle - cap, min), Position::new(middle + cap, min)),
                (Position::new(middle - cap, max), Position::new(middle + cap, max)),
            ];

            canvas.stroke_lines(&segments, self.line_width, color);
        });
    }
}
