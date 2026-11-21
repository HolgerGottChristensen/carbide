use std::fmt::Debug;
use carbide::color::Color;
use carbide::draw::{Position, Rect, Scalar};
use carbide::draw::color::{rgba, Rgba};
use carbide_core::widget::canvas::CanvasContext;
use crate::DataSetOptions;
use crate::controller::drawing::{dataset_color, split_runs, Palette};
use crate::controller::line_path::{trace_line, trace_line_reversed};
use crate::controller::orientation::{pixel, Oriented, Orientation};
use crate::mark::MarkRole;
use crate::scale::Scale;
use crate::style::series_style;

/// A mark that can be drawn by a line controller, for datasets of type `D`. Implementing it is
/// what lets a dataset with this mark be drawn by a `LineController` (see `LineDataSet`).
///
/// A primary mark is drawn instead of the dataset's line and points, before every dataset's lines,
/// as it is usually an area, and reports what it draws to the scales' bounds instead of the
/// dataset's points. A dataset drawn by a primary mark doesn't stack. An overlay is drawn on top
/// of every dataset's lines and points, and adds what it draws to the points' bounds.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be drawn by a line controller for `{D}`",
    label = "not a line mark for this dataset",
    note = "line marks are `RangeMark` (`.range()`) on datasets of `RangePoint`s, and types of your own implementing `LineMark`",
)]
pub trait LineMark<D: ?Sized>: Clone + Debug + 'static {
    /// Whether the mark replaces the line or is drawn on top of it.
    fn role(&self) -> MarkRole;

    /// Calls `f` with each point's x and the lowest and highest value the mark draws for it, for
    /// the scale bounds. Points the mark doesn't draw are left out.
    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

    /// Draws the mark for the dataset's points.
    fn draw(&self, data: &D, canvas: &mut LineCanvas);
}

/// A line mark together with the dataset it draws, as a line controller reads it from a dataset's
/// `LineDataSet::line_marks`.
pub trait DatasetLineMark {
    fn role(&self) -> MarkRole;

    fn bounds(&self, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

    fn draw(&self, canvas: &mut LineCanvas);
}

pub(crate) struct BoundLineMark<'a, M, D> {
    pub(crate) mark: &'a M,
    pub(crate) data: &'a D,
}

impl<M: LineMark<D>, D> DatasetLineMark for BoundLineMark<'_, M, D> {
    fn role(&self) -> MarkRole {
        self.mark.role()
    }

    fn bounds(&self, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        self.mark.bounds(self.data, f);
    }

    fn draw(&self, canvas: &mut LineCanvas) {
        self.mark.draw(self.data, canvas);
    }
}

/// What a line mark draws on. Positions are in the index frame: x along the categories and y along
/// the values, both in pixels, so a mark draws the same code for upright and lying lines. Lines
/// and areas traced through it follow the dataset's `interpolation`.
pub struct LineCanvas<'x, 'a, 'b, 'c: 'b> {
    pub(crate) ctx: &'x mut CanvasContext<'a, 'b, 'c>,
    pub(crate) orientation: Orientation,
    pub(crate) index_scale: &'x dyn Scale,
    pub(crate) value_scale: &'x dyn Scale,
    pub(crate) area: Rect,
    pub(crate) options: &'x DataSetOptions,
    pub(crate) dataset_index: usize,
    pub(crate) palette: &'x Palette,
}

impl LineCanvas<'_, '_, '_, '_> {
    /// Where the point at `x`, its category, and `value` falls, in the index frame.
    pub fn position(&self, x: Scalar, value: Scalar) -> Position {
        Position::new(
            pixel(self.orientation.index_axis(), self.index_scale, x, self.area),
            pixel(self.orientation.value_axis(), self.value_scale, value, self.area),
        )
    }

    /// The runs a line through the points is drawn in without a break, as the dataset's
    /// `span_gaps` says. `points` holds, for each of the dataset's points in order, its x, or
    /// `None` when the mark doesn't draw it; the runs are ranges over the points it draws.
    pub fn runs(&self, points: &[Option<Scalar>]) -> Vec<std::ops::Range<usize>> {
        split_runs(points, self.options.span_gaps)
    }

    /// The dataset's colour, as its line would be drawn in.
    pub fn color(&self) -> Color {
        dataset_color(self.options, self.dataset_index, self.palette)
    }

    /// `color` at the `SeriesStyle`'s fill opacity, for areas, so the grid and areas behind show
    /// through.
    pub fn translucent(&mut self, color: Color) -> Color {
        let fill_opacity = series_style(self.ctx.env()).fill_opacity();
        let Rgba(r, g, b, a) = Rgba::from(color);
        rgba(r, g, b, a * fill_opacity)
    }

    /// Fills the area between the line through `upper` and the line through `lower`, both in
    /// the index frame, in order along the categories, and of the same length.
    pub fn fill_between(&mut self, upper: &[Position], lower: &[Position], color: Color) {
        let area = self.orientation.frame_area(self.area);
        let interpolation = self.options.interpolation;

        self.ctx.begin_path();
        let mut path = Oriented { path: &mut *self.ctx, orientation: self.orientation };
        trace_line(&mut path, upper, interpolation, area);
        trace_line_reversed(&mut path, lower, interpolation, area);
        self.ctx.close_path();
        self.ctx.set_fill_style(color);
        self.ctx.fill();
    }

    /// Strokes the line through `points`, in the index frame.
    pub fn stroke_line(&mut self, points: &[Position], color: Color) {
        let area = self.orientation.frame_area(self.area);

        self.ctx.begin_path();
        let mut path = Oriented { path: &mut *self.ctx, orientation: self.orientation };
        trace_line(&mut path, points, self.options.interpolation, area);
        self.ctx.set_stroke_style(color);
        self.ctx.stroke();
    }
}
