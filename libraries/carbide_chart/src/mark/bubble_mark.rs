use carbide::color::Color;
use carbide::draw::Scalar;
use crate::DataColor;
use crate::legend::LegendMarker;
use crate::dataset::SizedPoint;
use crate::mark::{LegendContext, Mark, Primary, ScatterCanvas, ScatterMark, SizeScale};
use crate::scale::Coordinates;
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
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(y), Some(size)) = (coordinates.x(point), coordinates.y(point), point.size()) && SizeScale::has_circle(size) {
                f(x, y);
            }
        });
    }

    fn sizes(&self, data: &D, f: &mut dyn FnMut(Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (false, Some(size)) = (point.is_missing(), point.size()) {
                f(size);
            }
        });
    }

    /// A bubble in the dataset's colour, in place of a point.
    fn legend(&self, _data: &D, legend: &LegendContext) -> Option<(Color, LegendMarker)> {
        Some((legend.resolve(DataColor::ByDataSet), LegendMarker::Bubble))
    }

    fn draw(&self, data: &D, canvas: &mut ScatterCanvas) {
        let coordinates = canvas.coordinates;
        data.foreach(&mut |point_index, point| {
            let (Some(x), Some(y), Some(size)) = (coordinates.x(point), coordinates.y(point), point.size()) else {
                return;
            };

            let Some(radius) = canvas.radius(size) else {
                return;
            };

            let color = canvas.point_color(point_index, point.color());
            let fill = canvas.translucent(color);
            let center = canvas.position(x, y);
            canvas.add_marker(point_index, x, center, radius, color);
            canvas.circle(center, radius, fill, color, self.outline_width);
        });
    }
}
