use std::collections::HashMap;
use carbide::color::Color;
use carbide::draw::{Position, Rect, Scalar};
use carbide::draw::color::{rgba, Rgba};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataSetOptions};
use crate::controller::band::{clip, flatten, strips};
use crate::controller::drawing::{dataset_color, resolve_color, split_runs, Palette};
use crate::controller::line_path::{trace_line, trace_line_reversed, PathSink};
use crate::controller::orientation::{pixel, Oriented, Orientation};
use crate::legend::LegendMarker;
use crate::mark::{LegendContext, Mark, MarkRole, Role};
use crate::scale::Scale;
use crate::style::{axis_style, series_style};

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
    note = "line marks are `RangeMark` (`.range()`) and `ErrorBars` (`.error_bars()`) on datasets of `RangePoint`s, and types of your own implementing `LineMark`",
)]
pub trait LineMark<D: ?Sized>: Mark {
    /// Calls `f` with each point's x and the lowest and highest value the mark draws for it, for
    /// the scale bounds, before any stacking: the controller moves an overlay's values with its
    /// stacked line. Points the mark doesn't draw are left out.
    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

    /// Draws the mark for the dataset's points.
    fn draw(&self, data: &D, canvas: &mut LineCanvas);

    /// The colour and marker of the dataset's legend entry, when the mark changes how the dataset
    /// looks, or `None`, the default, for the controller's own entry. The first mark of a dataset
    /// that gives one decides it.
    fn legend(&self, _data: &D, _legend: &LegendContext) -> Option<(Color, LegendMarker)> {
        None
    }
}

/// A line mark together with the dataset it draws, as a line controller reads it from a dataset's
/// `LineDataSet::line_marks`.
pub trait DatasetLineMark {
    fn role(&self) -> MarkRole;

    fn bounds(&self, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

    fn draw(&self, canvas: &mut LineCanvas);

    fn legend(&self, legend: &LegendContext) -> Option<(Color, LegendMarker)>;
}

pub(crate) struct BoundLineMark<'a, M, D> {
    pub(crate) mark: &'a M,
    pub(crate) data: &'a D,
}

impl<M: LineMark<D>, D> DatasetLineMark for BoundLineMark<'_, M, D> {
    fn role(&self) -> MarkRole {
        <M::Role as Role>::ROLE
    }

    fn bounds(&self, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        self.mark.bounds(self.data, f);
    }

    fn draw(&self, canvas: &mut LineCanvas) {
        self.mark.draw(self.data, canvas);
    }

    fn legend(&self, legend: &LegendContext) -> Option<(Color, LegendMarker)> {
        self.mark.legend(self.data, legend)
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
    pub(crate) stack_offsets: Option<&'x HashMap<u64, Scalar>>,
}

impl LineCanvas<'_, '_, '_, '_> {
    /// How far stacking moves the dataset's line at `x` along the value axis, in values: where
    /// the line is drawn, less the point's own value, or zero when it doesn't stack. A mark drawn
    /// on a stacked line adds it to the values it draws, so it moves with the line.
    pub fn stack_offset(&self, x: Scalar) -> Scalar {
        crate::controller::stacking::stack_offset(self.stack_offsets, x)
    }

    /// The colour for marks drawn over the datasets' colours, such as error bars: the chart's
    /// label colour, so they stand out on any line or area.
    pub fn ink(&mut self) -> Color {
        let env = self.ctx.env();
        axis_style(env).label_color(env)
    }

    /// Starts a path of straight lines, added one at a time with `add_line` and stroked together
    /// with `stroke_lines_added`, so a mark can draw many lines without collecting them first.
    pub fn begin_lines(&mut self) {
        self.ctx.begin_path();
    }

    /// Adds the straight line from `from` to `to`, in the index frame, to the path `begin_lines`
    /// started.
    pub fn add_line(&mut self, from: Position, to: Position) {
        let mut path = Oriented { path: &mut *self.ctx, orientation: self.orientation };
        path.move_to(from);
        path.line_to(to);
    }

    /// Strokes the lines added since `begin_lines`, `width` wide.
    pub fn stroke_lines_added(&mut self, width: Scalar, color: Color) {
        self.ctx.save();
        self.ctx.set_line_width(width);
        self.ctx.set_stroke_style(color);
        self.ctx.stroke();
        self.ctx.restore();
    }

    /// Strokes the straight lines between each pair of `segments`, `width` wide.
    pub fn stroke_lines(&mut self, segments: &[(Position, Position)], width: Scalar, color: Color) {
        self.begin_lines();
        for (from, to) in segments {
            self.add_line(*from, *to);
        }
        self.stroke_lines_added(width, color);
    }

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

    /// The colour `color` stands for in this dataset: a fixed colour, a palette colour, or, for a
    /// colour by dataset or by point, the dataset's colour.
    pub fn resolve(&self, color: DataColor) -> Color {
        resolve_color(color, self.options, self.dataset_index, self.palette)
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

    /// Fills the band between the line through `highs` and the line through `lows`, as
    /// `fill_between` does, in `color`. With an `inverted` colour, the parts where the line through
    /// the highs runs below the line through the lows are filled in it instead. A band that never
    /// inverts is filled exactly as by `fill_between`.
    pub fn fill_band(&mut self, highs: &[Position], lows: &[Position], color: Color, inverted: Option<Color>) {
        let Some(inverted) = inverted else {
            self.fill_between(highs, lows, color);
            return;
        };

        let area = self.orientation.frame_area(self.area);
        let interpolation = self.options.interpolation;
        let (high_line, low_line) = (flatten(highs, interpolation, area), flatten(lows, interpolation, area));

        // Which way the values grow along y in the frame: up the screen for upright lines, whose
        // pixels grow downwards, and along it for lying ones.
        let higher_is_greater = self.position(0.0, self.value_scale.max()).y >= self.position(0.0, self.value_scale.min()).y;
        let strips = strips(&high_line, &low_line, higher_is_greater);

        if !strips.iter().any(|strip| strip.inverted) {
            self.fill_between(highs, lows, color);
            return;
        }

        for strip in strips {
            let (top, bottom) = (clip(&high_line, strip.start, strip.end), clip(&low_line, strip.start, strip.end));
            let (Some(first), Some(last)) = (top.first(), bottom.last()) else {
                continue;
            };

            self.ctx.begin_path();
            let mut path = Oriented { path: &mut *self.ctx, orientation: self.orientation };
            path.move_to(*first);
            for point in &top[1..] {
                path.line_to(*point);
            }
            path.line_to(*last);
            for point in bottom.iter().rev().skip(1) {
                path.line_to(*point);
            }
            self.ctx.close_path();
            self.ctx.set_fill_style(if strip.inverted { inverted } else { color });
            self.ctx.fill();
        }
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
