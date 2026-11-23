use std::fmt::Debug;
use carbide::color::Color;
use carbide::draw::{Position, Rect, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataPoint};
use crate::colors::DatasetColors;
use crate::controller::orientation::pixel;
use crate::legend::LegendMarker;
use crate::mark::{LegendContext, Mark, MarkRole, Role};
use crate::scale::{Axis, Coordinates, Scale, ScaleId};
use crate::geometry::{Along, Datum, Shape, Shapes};
use crate::controller::drawing::PolarFrame;
use crate::mark::line_mark::polar_whisker;

/// A mark that can be drawn by a scatter controller, for datasets of type `D`. Implementing it is
/// what lets a dataset with this mark be drawn by a `ScatterController` (see `ScatterDataSet`).
///
/// A primary mark is drawn instead of the controller's points, after the lines, and reports what
/// it draws to the scales' bounds instead of the points: points it doesn't draw are left out,
/// unless a line runs through them (`ScatterController::show_line`). An overlay is drawn on top of
/// every dataset's points, and adds what it draws to the points' bounds.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be drawn by a scatter controller for `{D}`",
    label = "not a scatter mark for this dataset",
    note = "scatter marks are `BubbleMark` (`.bubbles()`) on datasets of `SizedPoint`s, `ErrorBars` (`.error_bars()`) on datasets of `RangePoint`s, and types of your own implementing `ScatterMark`",
)]
pub trait ScatterMark<D: ?Sized>: Mark {
    /// Calls `f` with each x and y the mark draws, for the scale bounds: for a primary mark, the
    /// points it draws in place of the dataset's; for an overlay, anything beyond them.
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar));

    /// Calls `f` with each size the mark draws, so the controller can map every dataset's sizes
    /// through one size scale (see `ScatterCanvas::radius`). By default it draws none.
    fn sizes(&self, _data: &D, _f: &mut dyn FnMut(Scalar)) {}

    /// Draws the mark for each of the dataset's points.
    fn draw(&self, data: &D, canvas: &mut ScatterCanvas);

    /// The colour and marker of the dataset's legend entry, when the mark changes how the dataset
    /// looks, or `None`, the default, for the controller's own entry. The first mark of a dataset
    /// that gives one decides it.
    fn legend(&self, _data: &D, _legend: &LegendContext) -> Option<(Color, LegendMarker)> {
        None
    }
}

/// A scatter mark together with the dataset it draws, as a scatter controller reads it from a
/// dataset's `ScatterDataSet::scatter_marks`.
pub trait DatasetScatterMark {
    fn role(&self) -> MarkRole;

    fn bounds(&self, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar));

    fn sizes(&self, f: &mut dyn FnMut(Scalar));

    fn draw(&self, canvas: &mut ScatterCanvas);

    fn legend(&self, legend: &LegendContext) -> Option<(Color, LegendMarker)>;
}

pub(crate) struct BoundScatterMark<'a, M, D> {
    pub(crate) mark: &'a M,
    pub(crate) data: &'a D,
}

impl<M: ScatterMark<D>, D> DatasetScatterMark for BoundScatterMark<'_, M, D> {
    fn role(&self) -> MarkRole {
        <M::Role as Role>::ROLE
    }

    fn bounds(&self, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar)) {
        self.mark.bounds(self.data, coordinates, f);
    }

    fn sizes(&self, f: &mut dyn FnMut(Scalar)) {
        self.mark.sizes(self.data, f);
    }

    fn draw(&self, canvas: &mut ScatterCanvas) {
        self.mark.draw(self.data, canvas);
    }

    fn legend(&self, legend: &LegendContext) -> Option<(Color, LegendMarker)> {
        self.mark.legend(self.data, legend)
    }
}

/// How a scatter controller turns sizes into radii, shared by all its datasets so their sizes
/// compare: a circle's area is proportional to its size, the largest size in the controller gets
/// `max_radius`, and no circle is smaller than `min_radius`, so small sizes stay visible.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SizeScale {
    pub(crate) min_radius: Scalar,
    pub(crate) max_radius: Scalar,
    /// The largest size among the controller's datasets, if any is positive.
    pub(crate) max_size: Option<Scalar>,
}

impl SizeScale {
    /// Whether a circle can be drawn for `size`: not for a size that is negative or not a number,
    /// as a size below zero has no area.
    pub fn has_circle(size: Scalar) -> bool {
        size >= 0.0
    }

    /// The radius for `size`, in pixels: `None` for a size without a circle (see `has_circle`). A
    /// size of zero gets the smallest radius.
    pub fn radius(&self, size: Scalar) -> Option<Scalar> {
        if !SizeScale::has_circle(size) {
            return None;
        }

        let radius = match self.max_size {
            Some(max_size) => self.max_radius * (size / max_size).sqrt(),
            None => self.min_radius,
        };

        Some(radius.max(self.min_radius))
    }
}

/// What a scatter mark draws on: the dataset's x and y scales, its colours, and the controller's
/// size scale.
pub struct ScatterCanvas<'x, 'a, 'b, 'c: 'b> {
    pub(crate) ctx: &'x mut CanvasContext<'a, 'b, 'c>,
    pub(crate) coordinates: Coordinates<'x>,
    pub(crate) x_scale: &'x dyn Scale,
    pub(crate) y_scale: &'x dyn Scale,
    pub(crate) area: Rect,
    pub(crate) dataset_index: usize,
    pub(crate) colors: DatasetColors<'x>,
    pub(crate) sizes: SizeScale,
    /// How the dataset lies around the circle, when it is on polar scales.
    pub(crate) polar: Option<&'x PolarFrame>,
    /// The chart's shapes, which the marks add theirs to, for picking.
    pub(crate) shapes: &'x mut Shapes,
    /// The id of the dataset's x scale, its index scale.
    pub(crate) x_id: &'x ScaleId,
}

impl<'x> ScatterCanvas<'x, '_, '_, '_> {
    /// The dataset's colours (see `DatasetColors`), and through them the controller's.
    pub fn colors(&self) -> DatasetColors<'x> {
        self.colors
    }

    /// Where the point's x is placed: a number at itself, and text at its category's index (see
    /// `Coordinates`). Marks read their points' x and y through it, so they draw text as well as
    /// numbers.
    pub fn x<P: DataPoint + ?Sized>(&self, point: &P) -> Option<Scalar> {
        self.coordinates.x(point)
    }

    /// Where the point's y is placed (see `x`).
    pub fn y<P: DataPoint + ?Sized>(&self, point: &P) -> Option<Scalar> {
        self.coordinates.y(point)
    }

    /// Where the point at `x` and `y` on the dataset's scales falls on the screen: around the
    /// chart's circle when they are polar scales.
    pub fn position(&self, x: Scalar, y: Scalar) -> Position {
        if let Some(frame) = self.polar {
            let (angle, distance) = self.polar_frame_position(frame, x, y);
            return frame.polar.at(angle, distance);
        }

        Position::new(
            pixel(Axis::Horizontal, self.x_scale, x, self.area),
            pixel(Axis::Vertical, self.y_scale, y, self.area),
        )
    }

    /// Where the point at `x` and `y` is around the circle: its screen angle and its distance from
    /// the centre.
    fn polar_frame_position(&self, frame: &PolarFrame, x: Scalar, y: Scalar) -> (Scalar, Scalar) {
        let (x, y) = (self.x_scale.normalize(x), self.y_scale.normalize(y));
        let (angle, radius) = if frame.x_is_angle { (x, y) } else { (y, x) };
        (frame.polar.angle(angle), frame.polar.distance(radius))
    }

    /// Adds a circle around `center` as the shape of the point at `point_index`, whose x is `x`,
    /// so picking finds the point by it.
    pub(crate) fn add_marker(&mut self, point_index: usize, x: Scalar, center: Position, radius: Scalar, color: Color) {
        let along = match self.polar {
            Some(frame) => {
                let circle = frame.polar.circle().center;
                let normalized = self.x_scale.normalize(x);
                if frame.x_is_angle {
                    Along::Angle { center: circle, angle: frame.polar.angle(normalized) }
                } else {
                    Along::Distance { center: circle, distance: frame.polar.distance(normalized) }
                }
            }
            None => Along::X(pixel(Axis::Horizontal, self.x_scale, x, self.area)),
        };

        let place = self.shapes.place(self.x_id, x, along);
        self.shapes.add(self.dataset_index, Some(Datum { index: point_index, place: Some(place) }), color, Shape::Marker { center, radius });
    }

    /// Adds a whisker through the point `point`, from `from` to `to`, each an x and a y on the
    /// dataset's scales, with a cap `cap` pixels across at each end, to the path `begin_lines`
    /// started. Along the sides, it is straight, its caps across it. Around a circle, it follows
    /// the coordinate that changes: out along the spoke for the radius, its caps across the spoke,
    /// or around the ring through the point for the angle, its caps across the ring.
    pub fn add_whisker(&mut self, point: (Scalar, Scalar), from: (Scalar, Scalar), to: (Scalar, Scalar), cap: Scalar) {
        if let Some(frame) = self.polar {
            let through = self.polar_frame_position(frame, point.0, point.1);
            let (from, to) = (self.polar_frame_position(frame, from.0, from.1), self.polar_frame_position(frame, to.0, to.1));
            polar_whisker(self.ctx, frame, through, from, to, cap);
            return;
        }

        let half = cap / 2.0;
        let (low, high) = (self.position(from.0, from.1), self.position(to.0, to.1));
        self.add_line(low, high);

        // Caps across the whisker: level across an upright one, upright across a level one.
        if from.0 == to.0 {
            self.add_line(Position::new(low.x - half, low.y), Position::new(low.x + half, low.y));
            self.add_line(Position::new(high.x - half, high.y), Position::new(high.x + half, high.y));
        } else {
            self.add_line(Position::new(low.x, low.y - half), Position::new(low.x, low.y + half));
            self.add_line(Position::new(high.x, high.y - half), Position::new(high.x, high.y + half));
        }
    }

    /// The radius for `size` on the controller's size scale (see `SizeScale::radius`).
    pub fn radius(&self, size: Scalar) -> Option<Scalar> {
        self.sizes.radius(size)
    }

    /// The colour of the point at `point_index`, whose own colour is `color` (see
    /// `DataPoint::color`), as a point would be drawn in.
    pub fn point_color(&self, point_index: usize, color: DataColor) -> Color {
        self.colors.point(point_index, color)
    }

    /// The colour `color` stands for in this dataset: a fixed colour, a palette colour, or, for a
    /// colour by dataset or by point, the dataset's colour.
    pub fn resolve(&self, color: DataColor) -> Color {
        self.colors.resolve(color)
    }

    /// The colour for marks drawn over the datasets' colours, such as error bars: the chart's
    /// label colour, so they stand out on any point.
    pub fn ink(&self) -> Color {
        self.colors.series().ink()
    }

    /// Starts a path of straight lines, added one at a time with `add_line` and stroked together
    /// with `stroke_lines_added`, so a mark can draw many lines without collecting them first.
    pub fn begin_lines(&mut self) {
        self.ctx.begin_path();
    }

    /// Adds the straight line from `from` to `to`, in pixels, to the path `begin_lines` started.
    pub fn add_line(&mut self, from: Position, to: Position) {
        self.ctx.move_to(from.x, from.y);
        self.ctx.line_to(to.x, to.y);
    }

    /// Strokes the lines added since `begin_lines`, `width` wide.
    pub fn stroke_lines_added(&mut self, width: Scalar, color: Color) {
        self.ctx.save();
        self.ctx.set_line_width(width);
        self.ctx.set_stroke_style(color);
        self.ctx.stroke();
        self.ctx.restore();
    }

    /// `color` at the `SeriesStyle`'s fill opacity, for shapes that may overlap, so the ones
    /// behind show through.
    pub fn translucent(&self, color: Color) -> Color {
        self.colors.series().translucent(color)
    }

    /// Fills a circle around `center`, and outlines it `outline_width` wide.
    pub fn circle(&mut self, center: Position, radius: Scalar, fill: Color, outline: Color, outline_width: Scalar) {
        self.ctx.save();
        self.ctx.begin_path();
        // The canvas takes a circle's diameter.
        self.ctx.circle(center.x, center.y, radius * 2.0);
        self.ctx.set_fill_style(fill);
        self.ctx.fill();
        self.ctx.set_line_width(outline_width);
        self.ctx.set_stroke_style(outline);
        self.ctx.stroke();
        self.ctx.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::SizeScale;

    /// Areas are proportional to sizes: the largest size gets the largest radius, a quarter of it
    /// half the radius. Tiny and zero sizes get the smallest radius, and negative or NaN sizes no
    /// circle at all.
    #[test]
    fn sizes_map_to_areas() {
        let scale = SizeScale { min_radius: 4.0, max_radius: 30.0, max_size: Some(100.0) };

        assert_eq!(scale.radius(100.0), Some(30.0));
        assert_eq!(scale.radius(25.0), Some(15.0), "a quarter of the area is half the radius");
        assert_eq!(scale.radius(1.0), Some(4.0), "3px would be too small, so it gets the smallest radius");
        assert_eq!(scale.radius(0.0), Some(4.0));
        assert_eq!(scale.radius(-5.0), None, "a negative size has no area");
        assert_eq!(scale.radius(f64::NAN), None);

        let no_sizes = SizeScale { max_size: None, ..scale };
        assert_eq!(no_sizes.radius(0.0), Some(4.0), "without a positive size, every circle is the smallest");
    }
}
