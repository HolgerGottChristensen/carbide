use std::fmt::Debug;
use carbide::color::Color;
use carbide::draw::{Position, Rect, Scalar};
use carbide::draw::color::{rgba, Rgba};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataSetOptions};
use crate::controller::drawing::{point_color, resolve_color, Palette};
use crate::controller::orientation::pixel;
use crate::legend::LegendMarker;
use crate::mark::{LegendContext, Mark, MarkRole, Role};
use crate::scale::{Axis, Scale};
use crate::style::{axis_style, series_style};

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
    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar));

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

    fn bounds(&self, f: &mut dyn FnMut(Scalar, Scalar));

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

    fn bounds(&self, f: &mut dyn FnMut(Scalar, Scalar)) {
        self.mark.bounds(self.data, f);
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
    pub(crate) x_scale: &'x dyn Scale,
    pub(crate) y_scale: &'x dyn Scale,
    pub(crate) area: Rect,
    pub(crate) options: &'x DataSetOptions,
    pub(crate) dataset_index: usize,
    pub(crate) palette: &'x Palette,
    pub(crate) sizes: SizeScale,
}

impl ScatterCanvas<'_, '_, '_, '_> {
    /// Where the point at `x` and `y` on the dataset's scales falls on the screen.
    pub fn position(&self, x: Scalar, y: Scalar) -> Position {
        Position::new(
            pixel(Axis::Horizontal, self.x_scale, x, self.area),
            pixel(Axis::Vertical, self.y_scale, y, self.area),
        )
    }

    /// The radius for `size` on the controller's size scale (see `SizeScale::radius`).
    pub fn radius(&self, size: Scalar) -> Option<Scalar> {
        self.sizes.radius(size)
    }

    /// The colour of the point at `point_index`, whose own colour is `color` (see
    /// `DataPoint::color`), as a point would be drawn in.
    pub fn point_color(&self, point_index: usize, color: DataColor) -> Color {
        point_color(color, self.options, self.dataset_index, point_index, self.palette)
    }

    /// The colour `color` stands for in this dataset: a fixed colour, a palette colour, or, for a
    /// colour by dataset or by point, the dataset's colour.
    pub fn resolve(&self, color: DataColor) -> Color {
        resolve_color(color, self.options, self.dataset_index, self.palette)
    }

    /// The colour for marks drawn over the datasets' colours, such as error bars: the chart's
    /// label colour, so they stand out on any point.
    pub fn ink(&mut self) -> Color {
        let env = self.ctx.env();
        axis_style(env).label_color(env)
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
    pub fn translucent(&mut self, color: Color) -> Color {
        let fill_opacity = series_style(self.ctx.env()).fill_opacity();
        let Rgba(r, g, b, a) = Rgba::from(color);
        rgba(r, g, b, a * fill_opacity)
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
