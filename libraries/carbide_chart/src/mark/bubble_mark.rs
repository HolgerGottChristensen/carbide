use carbide::draw::Scalar;
use crate::dataset::SizedPoint;
use crate::mark::{Mark, Primary, ScatterCanvas, ScatterMark, SizeScale};
use crate::sequence::Sequence;

/// Draws each point as a bubble whose area is proportional to its size, in place of a point (see
/// `SizedDataSetExt::bubbles`). The controller maps the sizes of all its datasets through one size
/// scale (see `ScatterController::radius_range`), so bubbles compare across datasets. A bubble is
/// filled translucent, so bubbles behind it show through, and outlined in its colour. A point
/// missing its x, y or size, or with a negative size, isn't drawn, and isn't in the scales' bounds.
#[derive(Copy, Clone, Debug)]
pub struct BubbleMark {
    /// How wide the outline is, in pixels.
    pub outline_width: Scalar,
}

impl Default for BubbleMark {
    fn default() -> BubbleMark {
        BubbleMark { outline_width: 2.0 }
    }
}

impl Mark for BubbleMark {
    type Role = Primary;
}

impl<D: Sequence<dyn SizedPoint>> ScatterMark<D> for BubbleMark {
    /// The centres of the bubbles it draws.
    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(y), Some(size)) = (point.x(), point.y(), point.size()) && SizeScale::has_circle(size) {
                f(x, y);
            }
        });
    }

    fn sizes(&self, data: &D, f: &mut dyn FnMut(Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(_), Some(_), Some(size)) = (point.x(), point.y(), point.size()) {
                f(size);
            }
        });
    }

    fn draw(&self, data: &D, canvas: &mut ScatterCanvas) {
        data.foreach(&mut |point_index, point| {
            let (Some(x), Some(y), Some(size)) = (point.x(), point.y(), point.size()) else {
                return;
            };

            let Some(radius) = canvas.radius(size) else {
                return;
            };

            let color = canvas.point_color(point_index, point.color());
            let fill = canvas.translucent(color);
            canvas.circle(canvas.position(x, y), radius, fill, color, self.outline_width);
        });
    }
}
