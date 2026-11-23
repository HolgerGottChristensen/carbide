use std::collections::HashMap;
use carbide::color::Color;
use carbide::draw::{Position, Rect, Scalar};
use carbide::draw::color::{rgba, Rgba};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataPoint, DataSetOptions};
use crate::controller::band::{clip, flatten, strips};
use crate::controller::drawing::{closes, dataset_color, resolve_color, split_runs, Palette, PolarFrame};
use crate::controller::polar_path::{trace_polar_line, trace_polar_line_reversed, PolarPoint};
use crate::controller::line_path::{trace_line, trace_line_reversed, PathSink};
use crate::controller::orientation::{pixel, Oriented, Orientation};
use crate::legend::LegendMarker;
use crate::mark::{LegendContext, Mark, MarkRole, Role};
use crate::scale::{Coordinates, Scale};
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
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

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

    fn bounds(&self, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

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

    fn bounds(&self, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        self.mark.bounds(self.data, coordinates, f);
    }

    fn draw(&self, canvas: &mut LineCanvas) {
        self.mark.draw(self.data, canvas);
    }

    fn legend(&self, legend: &LegendContext) -> Option<(Color, LegendMarker)> {
        self.mark.legend(self.data, legend)
    }
}

/// What a line mark draws on. Positions are in the index frame: x along the categories and y along
/// the values, both in pixels, so a mark draws the same code for vertical and horizontal lines. Lines
/// and areas traced through it follow the dataset's `interpolation`.
///
/// Around a polar chart's circle, the frame is the circle's own: x is the screen angle, in radians
/// clockwise from the top, and y the distance from the centre, in pixels. A mark that draws
/// through the canvas's methods (`position`, `stroke_line`, `fill_band`, `add_whisker`) draws
/// around the circle unchanged; lines close into loops where `closes` says, and turn around the
/// grid's rings where they step.
pub struct LineCanvas<'x, 'a, 'b, 'c: 'b> {
    pub(crate) ctx: &'x mut CanvasContext<'a, 'b, 'c>,
    pub(crate) coordinates: Coordinates<'x>,
    pub(crate) orientation: Orientation,
    pub(crate) index_scale: &'x dyn Scale,
    pub(crate) value_scale: &'x dyn Scale,
    pub(crate) area: Rect,
    pub(crate) options: &'x DataSetOptions,
    pub(crate) dataset_index: usize,
    pub(crate) palette: &'x Palette,
    pub(crate) stack_offsets: Option<&'x HashMap<u64, Scalar>>,
    /// How the dataset lies around the circle, when it is on polar scales.
    pub(crate) polar: Option<&'x PolarFrame>,
}

impl LineCanvas<'_, '_, '_, '_> {
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
        if let Some(polar) = self.polar {
            let (from, to) = (screen(polar, from), screen(polar, to));
            self.ctx.move_to(from.x, from.y);
            self.ctx.line_to(to.x, to.y);
            return;
        }

        let mut path = Oriented { path: &mut *self.ctx, orientation: self.orientation };
        path.move_to(from);
        path.line_to(to);
    }

    /// Adds a whisker through the point at the category `x` and `value`, from the value `low` to
    /// the value `high`, with a cap `cap` pixels across at each end, to the path `begin_lines`
    /// started. Along a side, it is straight along the value axis, its caps along the index axis.
    /// Around a circle, it follows the values: out along the spoke when they run along the radius,
    /// its caps across the spoke, or around the ring through the point when they run around the
    /// circle, its caps across the ring.
    pub fn add_whisker(&mut self, x: Scalar, value: Scalar, low: Scalar, high: Scalar, cap: Scalar) {
        let (from, to) = (self.position(x, low), self.position(x, high));

        let Some(polar) = self.polar else {
            let half = cap / 2.0;
            self.add_line(from, to);
            self.add_line(Position::new(from.x - half, from.y), Position::new(from.x + half, from.y));
            self.add_line(Position::new(to.x - half, to.y), Position::new(to.x + half, to.y));
            return;
        };

        let through = self.position(x, value);
        polar_whisker(self.ctx, polar, (through.x, through.y), (from.x, from.y), (to.x, to.y), cap);
    }

    /// Whether a line through the points in `runs` (see `runs`) closes into a loop: only around a
    /// full circle, with the categories around it, in one run without a break where it would
    /// close. `points` holds, for each point, its x, or `None` when it isn't drawn.
    pub fn closes(&self, runs: &[std::ops::Range<usize>], points: &[Option<Scalar>]) -> bool {
        self.polar.is_some_and(|polar| polar.x_is_angle && polar.polar.is_full_circle() && closes(runs, points, self.options.span_gaps))
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

    /// Where the point at `x`, its category, and `value` falls, in the index frame: around a
    /// circle, its screen angle and its distance from the centre.
    pub fn position(&self, x: Scalar, value: Scalar) -> Position {
        if let Some(frame) = self.polar {
            let (x, value) = (self.index_scale.normalize(x), self.value_scale.normalize(value));
            let (angle, radius) = if frame.x_is_angle { (x, value) } else { (value, x) };
            return Position::new(frame.polar.angle(angle), frame.polar.distance(radius));
        }

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
    /// the index frame, in order along the categories, and of the same length. Around a circle,
    /// the two lines are open; `fill_loops_between` fills between two loops.
    pub fn fill_between(&mut self, upper: &[Position], lower: &[Position], color: Color) {
        if let Some(polar) = self.polar {
            let interpolation = self.options.interpolation;
            self.ctx.begin_path();
            trace_polar_line(&mut *self.ctx, &polar_points(polar, upper), None, interpolation, polar.rings());
            trace_polar_line_reversed(&mut *self.ctx, &polar_points(polar, lower), interpolation, polar.rings());
            self.ctx.close_path();
            self.ctx.set_fill_style(color);
            self.ctx.fill();
            return;
        }

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
    ///
    /// Around a circle, with `closed` (see `closes`), the two lines are loops and the band is the
    /// ring between them. Inverted parts aren't told apart around a circle yet: the whole band is
    /// filled in `color`.
    pub fn fill_band(&mut self, highs: &[Position], lows: &[Position], color: Color, inverted: Option<Color>, closed: bool) {
        if let Some(polar) = self.polar {
            if !closed {
                self.fill_between(highs, lows, color);
                return;
            }

            let interpolation = self.options.interpolation;
            let (highs, lows) = (polar_points(polar, highs), polar_points(polar, lows));
            self.ctx.begin_path();
            trace_polar_line(&mut *self.ctx, &highs, closing(polar, &highs), interpolation, polar.rings());

            // The loop of lows the other way round, so the ring between fills under either rule.
            let reversed: Vec<PolarPoint> = lows.iter().rev().copied().collect();
            trace_polar_line(&mut *self.ctx, &reversed, closing(polar, &reversed), reversed_steps(interpolation), polar.rings());
            self.ctx.close_path();
            self.ctx.set_fill_style(color);
            self.ctx.fill();
            return;
        }

        let Some(inverted) = inverted else {
            self.fill_between(highs, lows, color);
            return;
        };

        let area = self.orientation.frame_area(self.area);
        let interpolation = self.options.interpolation;
        let (high_line, low_line) = (flatten(highs, interpolation, area), flatten(lows, interpolation, area));

        // Which way the values grow along y in the frame: up the screen for upright lines, whose
        // pixels grow downwards, and along it for horizontal ones.
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

    /// Strokes the line through `points`, in the index frame, closed into a loop around a circle
    /// with `closed` (see `closes`).
    pub fn stroke_line(&mut self, points: &[Position], color: Color, closed: bool) {
        if let Some(polar) = self.polar {
            let points = polar_points(polar, points);
            let closing = if closed { closing(polar, &points) } else { None };
            self.ctx.begin_path();
            trace_polar_line(&mut *self.ctx, &points, closing, self.options.interpolation, polar.rings());
            self.ctx.set_stroke_style(color);
            self.ctx.stroke();
            return;
        }

        let area = self.orientation.frame_area(self.area);

        self.ctx.begin_path();
        let mut path = Oriented { path: &mut *self.ctx, orientation: self.orientation };
        trace_line(&mut path, points, self.options.interpolation, area);
        self.ctx.set_stroke_style(color);
        self.ctx.stroke();
    }
}

/// Where a position in a polar frame, a screen angle and a distance from the centre, is on the
/// screen.
fn screen(frame: &PolarFrame, position: Position) -> Position {
    frame.polar.at(position.x, position.y)
}

/// The points of a polar line from positions in its frame (see `LineCanvas`).
fn polar_points(frame: &PolarFrame, positions: &[Position]) -> Vec<PolarPoint> {
    positions.iter()
        .map(|position| PolarPoint { position: screen(frame, *position), angle: position.x, distance: position.y })
        .collect()
}

/// The first of `points` again, a full turn on in the direction the angle runs, for a loop to
/// close on.
fn closing(frame: &PolarFrame, points: &[PolarPoint]) -> Option<PolarPoint> {
    let first = *points.first()?;
    let (start, end) = frame.polar.angle_range();
    let ahead = points.last().is_some_and(|last| last.angle >= first.angle);
    let turn = if (end >= start) == ahead { std::f64::consts::TAU } else { -std::f64::consts::TAU };
    Some(PolarPoint { angle: first.angle + turn, ..first })
}

/// The interpolation a line traced backwards uses to follow the same path: steps turn the other
/// way round.
fn reversed_steps(interpolation: crate::Interpolation) -> crate::Interpolation {
    use crate::{Interpolation, Stepped};
    match interpolation {
        Interpolation::Stepped(Stepped::Before) => Interpolation::Stepped(Stepped::After),
        Interpolation::Stepped(Stepped::After) => Interpolation::Stepped(Stepped::Before),
        interpolation => interpolation,
    }
}

/// Adds a whisker around a polar chart's circle from `from` to `to`, each a screen angle and a
/// distance from the centre, with caps `cap` pixels across at both ends: along the spoke when
/// the angles are the same, its caps across it, and otherwise around the ring through `through`,
/// the point the whisker belongs to, its caps across the ring. A point between two corners of a
/// polygon ring lies outside the ring at its own distance, whose edge cuts inside the circle, so
/// the whisker runs along the larger ring whose edge passes through the point.
pub(crate) fn polar_whisker(ctx: &mut CanvasContext, frame: &PolarFrame, through: (Scalar, Scalar), from: (Scalar, Scalar), to: (Scalar, Scalar), cap: Scalar) {
    let half = cap / 2.0;
    let (start, end) = (frame.polar.at(from.0, from.1), frame.polar.at(to.0, to.1));

    if (from.0 - to.0).abs() < 1e-12 {
        // Out along the spoke; the caps lie across it, along the ring's tangent.
        let across = Position::new(from.0.cos() * half, from.0.sin() * half);
        PathSink::move_to(ctx, start);
        PathSink::line_to(ctx, end);
        for at in [start, end] {
            PathSink::move_to(ctx, Position::new(at.x - across.x, at.y - across.y));
            PathSink::line_to(ctx, Position::new(at.x + across.x, at.y + across.y));
        }
        return;
    }

    // Around the ring; each cap lies across the ring where the whisker ends: along the spoke
    // around a circle, and across the edge the whisker runs along around a polygon.
    let ring = frame.rings().at(through.1 / frame.rings().at(1.0).reach(through.0));
    PathSink::move_to(ctx, ring.point(from.0));
    ring.trace(ctx, from.0, to.0);
    let heading = to.0 - from.0;
    for (angle, towards) in [(from.0, heading), (to.0, -heading)] {
        let at = ring.point(angle);
        let along = ring.tangent(angle, towards);
        let across = Position::new(-along.y * half, along.x * half);
        PathSink::move_to(ctx, Position::new(at.x - across.x, at.y - across.y));
        PathSink::line_to(ctx, Position::new(at.x + across.x, at.y + across.y));
    }
}
