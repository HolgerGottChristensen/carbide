use std::collections::HashMap;
use std::fmt::Debug;
use carbide::color::Color;
use carbide::draw::{Position, Rect, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataPoint};
use crate::legend::LegendMarker;
use crate::colors::DatasetColors;
use crate::controller::drawing::PolarFrame;
use crate::controller::line_path::PathSink;
use crate::controller::orientation::{length, pixel, Oriented, Orientation};
use crate::mark::line_mark::polar_whisker;
use crate::mark::MarkRole;
use crate::{Mark, Role};
use crate::scale::{Coordinates, ScaleId};
use crate::Scale;
use crate::geometry::{Along, Datum, ShapeKind, Shapes};

/// A mark that can be drawn in a bar controller's slots, for datasets of type `D`. Implementing
/// it is what lets a dataset with this mark be drawn by a `BarController` (see `BarDataSet`).
///
/// The controller lays the slots out, grouping and orienting them as it does for bars, and the
/// mark draws each of its points in the slot for the point's x.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be drawn in a bar controller's slots for `{D}`",
    label = "not a bar mark for this dataset",
    note = "bar marks are `RangeMark` (`.range()`) and `ErrorBars` (`.error_bars()`) on datasets of `RangePoint`s, `BoxMark` (`.box_plot()`) on datasets of `BoxPoint`s, `CandleMark` (`.candles()`, `.ohlc()`) on datasets of `OhlcPoint`s, and types of your own implementing `BarMark`, attached with `.draw_as(..)` or `.overlay(..)` (see `Mark`)",
)]
pub trait BarMark<D: ?Sized>: Mark {
    /// Calls `f` with each point's x and the lowest and highest value the mark draws for it, for
    /// the scale bounds, before any stacking: the controller moves an overlay's values with its
    /// stacked bars. Points the mark doesn't draw are left out.
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

    /// Draws the mark for each of the dataset's points, in the slot for its x (`BarSlots::slot`).
    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas);

    /// Whether the mark draws around a polar chart's circle, through the canvas's frame (see
    /// `BarCanvas`). Marks that don't, the default, are skipped there, logged once: box plots
    /// and candlesticks, and marks of your own until they say they do.
    fn draws_around_a_circle(&self) -> bool {
        false
    }

    /// The colour and marker of the dataset's legend entry, when the mark draws it in colours of
    /// its own, or `None`, the default, for a square in the dataset's colour.
    fn legend(&self, _data: &D, _legend: &LegendContext) -> Option<(Color, LegendMarker)> {
        None
    }
}

/// What a mark can read to decide its dataset's legend entry (`BarMark::legend`,
/// `LineMark::legend`, `ScatterMark::legend`): the dataset's colours and the chart's
/// `SeriesStyle`.
pub struct LegendContext<'a> {
    pub(crate) colors: DatasetColors<'a>,
}

impl<'a> LegendContext<'a> {
    /// The dataset's colours (see `DatasetColors`), and through them the controller's.
    pub fn colors(&self) -> DatasetColors<'a> {
        self.colors
    }

    /// The colour `color` stands for in this dataset: a fixed colour, a palette colour, or, for a
    /// colour by dataset or by point, the dataset's colour.
    pub fn resolve(&self, color: DataColor) -> Color {
        self.colors.resolve(color)
    }

    /// The `SeriesStyle`'s colour for rising candlesticks and OHLC bars.
    pub fn rising_color(&self) -> Color {
        self.colors.series().rising()
    }

    /// The `SeriesStyle`'s colour for falling candlesticks and OHLC bars.
    pub fn falling_color(&self) -> Color {
        self.colors.series().falling()
    }
}

/// A bar mark together with the dataset it draws, as a bar controller reads it from a dataset's
/// `BarDataSet::bar_marks`.
pub trait DatasetBarMark {
    fn role(&self) -> MarkRole;

    fn bounds(&self, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

    fn draw(&self, slots: &BarSlots, canvas: &mut BarCanvas);

    fn legend(&self, legend: &LegendContext) -> Option<(Color, LegendMarker)>;

    fn draws_around_a_circle(&self) -> bool;
}

pub(crate) struct BoundBarMark<'a, M, D> {
    pub(crate) mark: &'a M,
    pub(crate) data: &'a D,
}

impl<M: BarMark<D>, D> DatasetBarMark for BoundBarMark<'_, M, D> {
    fn role(&self) -> MarkRole {
        <M::Role as Role>::ROLE
    }

    fn bounds(&self, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        self.mark.bounds(self.data, coordinates, f);
    }

    fn draw(&self, slots: &BarSlots, canvas: &mut BarCanvas) {
        self.mark.draw(self.data, slots, canvas);
    }

    fn legend(&self, legend: &LegendContext) -> Option<(Color, LegendMarker)> {
        self.mark.legend(self.data, legend)
    }

    fn draws_around_a_circle(&self) -> bool {
        self.mark.draws_around_a_circle()
    }
}

/// How a bar controller's datasets map onto the screen: along the sides of the chart area, in the
/// index frame of the controller's orientation, in pixels; or around a polar chart's circle, where
/// the index frame is the circle's own, the category's screen angle or distance from the centre,
/// whichever its scale is, and the value's the other (see `PolarFrame`).
#[derive(Copy, Clone)]
pub(crate) enum BarFrame<'a> {
    Sides { orientation: Orientation, area: Rect },
    Around(&'a PolarFrame),
}

impl BarFrame<'_> {
    /// Where `x` on the index scale falls along the index axis.
    pub(crate) fn index(&self, index_scale: &dyn Scale, x: Scalar) -> Scalar {
        match self {
            BarFrame::Sides { orientation, area } => pixel(orientation.index_axis(), index_scale, x, *area),
            BarFrame::Around(frame) if frame.x_is_angle => frame.polar.angle(index_scale.normalize(x)),
            BarFrame::Around(frame) => frame.polar.distance(index_scale.normalize(x)),
        }
    }

    /// Where `value` on the value scale falls along the value axis. Around the circle, a value
    /// past either end of the angle scale is kept at that end, so no bar goes round more than once.
    pub(crate) fn value(&self, value_scale: &dyn Scale, value: Scalar) -> Scalar {
        match self {
            BarFrame::Sides { orientation, area } => pixel(orientation.value_axis(), value_scale, value, *area),
            BarFrame::Around(frame) if frame.x_is_angle => frame.polar.distance(value_scale.normalize(value)),
            BarFrame::Around(frame) => frame.polar.angle(value_scale.normalize(value).clamp(0.0, 1.0)),
        }
    }

    /// The length of the index axis: the chart area's along a side, the sweep's angle around
    /// the circle, or the distance from the inner radius to the rim out along it.
    pub(crate) fn index_length(&self) -> Scalar {
        match self {
            BarFrame::Sides { orientation, area } => length(orientation.index_axis(), *area),
            BarFrame::Around(frame) if frame.x_is_angle => {
                let (start, end) = frame.polar.angle_range();
                (end - start).abs()
            }
            BarFrame::Around(frame) => frame.polar.circle().radius - frame.polar.inner_radius(),
        }
    }

    /// The shape from `side` to `side + thickness` along the index axis and from `from` to `to`
    /// along the value axis: a rectangle along the sides, and around the circle the part of a
    /// ring between two angles, its inner and outer edges in the grid's ring shape.
    pub(crate) fn shape(&self, side: Scalar, thickness: Scalar, from: Scalar, to: Scalar) -> ShapeKind {
        match self {
            BarFrame::Sides { orientation, .. } => ShapeKind::Rect { orientation: *orientation, side, thickness, from, to },
            BarFrame::Around(frame) => {
                let (angles, distances) = if frame.x_is_angle {
                    ((side, side + thickness), (from, to))
                } else {
                    ((from, to), (side, side + thickness))
                };

                ShapeKind::Wedge { polar: frame.polar, corners: frame.corners.clone(), angles, distances }
            }
        }
    }

    /// Where a category at `index`, a position along the index axis (see `index`), is on the
    /// screen, for picking the data at one category.
    pub(crate) fn along(&self, index: Scalar) -> Along {
        match self {
            BarFrame::Sides { orientation: Orientation::Vertical, .. } => Along::X(index),
            BarFrame::Sides { orientation: Orientation::Horizontal, .. } => Along::Y(index),
            BarFrame::Around(frame) if frame.x_is_angle => Along::Angle { center: frame.polar.circle().center, angle: index },
            BarFrame::Around(frame) => Along::Distance { center: frame.polar.circle().center, distance: index },
        }
    }

    /// Adds the shape from `side` to `side + thickness` along the index axis and from `from` to
    /// `to` along the value axis to the path, closed (see `shape`).
    pub(crate) fn trace_rect(&self, ctx: &mut CanvasContext, side: Scalar, thickness: Scalar, from: Scalar, to: Scalar) {
        self.shape(side, thickness, from, to).trace(ctx);
    }
}

/// Where a point's glyph goes along the index axis, in pixels: the centre of its category, and
/// the dataset's share of the category's slot, from `side`, `thickness` long. Share 0 is at the
/// smaller pixel: the left, or the top for horizontal bars. Around a polar chart's circle, in the
/// circle's frame (see `BarCanvas`): screen angles in radians for categories around the circle,
/// distances from the centre for categories out along the radius.
///
/// `stack_offset` is how far stacking moves the dataset's bar at this x along the value axis, in
/// values: where the bar ends, less the point's own value, or zero when it doesn't stack. A mark
/// drawn on a stacked bar adds it to the values it draws, so it moves with the bar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BarSlot {
    pub center: Scalar,
    pub side: Scalar,
    pub thickness: Scalar,
    pub stack_offset: Scalar,
}

impl BarSlot {
    /// The middle of the dataset's share, where a whisker or a line through the bars goes.
    pub fn middle(&self) -> Scalar {
        self.side + self.thickness / 2.0
    }
}

/// The slots of one dataset in a bar controller, by x value.
pub struct BarSlots<'a> {
    pub(crate) frame: BarFrame<'a>,
    pub(crate) index_scale: &'a dyn Scale,
    pub(crate) ruler: Scalar,
    pub(crate) group_index: usize,
    pub(crate) group_size: usize,
    pub(crate) category_percentage: Scalar,
    pub(crate) bar_percentage: Scalar,
    pub(crate) stack_offsets: Option<&'a HashMap<u64, Scalar>>,
}

impl BarSlots<'_> {
    /// The slot for a point at `x`, laid out as the controller lays out its bars.
    pub fn slot(&self, x: Scalar) -> BarSlot {
        let center = self.frame.index(self.index_scale, x);
        let (side, thickness) = crate::controller::bar_slot(center, self.ruler, self.group_index, self.group_size, self.category_percentage, self.bar_percentage);

        let stack_offset = crate::controller::stacking::stack_offset(self.stack_offsets, x);

        BarSlot { center, side, thickness, stack_offset }
    }
}

/// What a bar mark draws on. Positions are in the index frame: x along the categories and y along
/// the values, both in pixels, so a mark draws the same code for vertical and horizontal bars.
///
/// Around a polar chart's circle, the frame is the circle's own: for upright bars, x is the
/// screen angle, in radians clockwise from the top, and y the distance from the centre, in pixels;
/// for horizontal bars, whose categories run out along the radius, the other way round. A mark that
/// draws through `value_pixel`, `fill_rect` and `add_whisker` draws wedges and whiskers around the
/// circle unchanged, and says so with `BarMark::draws_around_a_circle`.
pub struct BarCanvas<'x, 'a, 'b, 'c: 'b> {
    pub(crate) ctx: &'x mut CanvasContext<'a, 'b, 'c>,
    pub(crate) coordinates: Coordinates<'x>,
    pub(crate) frame: BarFrame<'x>,
    pub(crate) value_scale: &'x dyn Scale,
    pub(crate) dataset_index: usize,
    pub(crate) colors: DatasetColors<'x>,
    /// The chart's shapes, which the marks add theirs to, for picking.
    pub(crate) shapes: &'x mut Shapes,
    /// The id of the dataset's index scale.
    pub(crate) index_id: &'x ScaleId,
}

impl<'x> BarCanvas<'x, '_, '_, '_> {
    /// Adds the rectangle across `slot` from `from` to `to` along the value axis, in the index
    /// frame's pixels as `value_pixel` gives them (around a circle, the wedge), as the shape of the
    /// point at `point_index`, whose x is `x`. That is how the chart finds the point under the
    /// pointer, for hovering, tooltips and clicks: a mark adds a shape for each point it draws, as
    /// the built-in marks do, or its points aren't found. Adding a shape draws nothing.
    pub fn add_rect(&mut self, point_index: usize, x: Scalar, slot: &BarSlot, from: Scalar, to: Scalar, color: Color) {
        let place = self.shapes.place(self.index_id, x, self.frame.along(slot.center));
        let shape = self.frame.shape(slot.side, slot.thickness, from, to);
        self.shapes.add_item(self.dataset_index, Some(Datum { index: point_index, place: Some(place) }), color, shape);
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

    /// Where `value` on the dataset's value scale falls along the value axis, in pixels, or,
    /// around a circle, in its frame.
    pub fn value_pixel(&self, value: Scalar) -> Scalar {
        self.frame.value(self.value_scale, value)
    }

    /// The dataset's colours (see `DatasetColors`), and through them the controller's.
    pub fn colors(&self) -> DatasetColors<'x> {
        self.colors
    }

    /// The colour of the point at `point_index`, whose own colour is `color` (see
    /// `DataPoint::color`), as a bar would be drawn in.
    pub fn point_color(&self, point_index: usize, color: DataColor) -> Color {
        self.colors.point(point_index, color)
    }

    /// The colour `color` stands for in this dataset: a fixed colour, a palette colour, or, for a
    /// colour by dataset or by point, the dataset's colour.
    pub fn resolve(&self, color: DataColor) -> Color {
        self.colors.resolve(color)
    }

    /// `color` at the `SeriesStyle`'s fill opacity, for shapes the grid should show through, such
    /// as a box plot's box.
    pub fn translucent(&self, color: Color) -> Color {
        self.colors.series().translucent(color)
    }

    /// The `SeriesStyle`'s colour for rising candlesticks and OHLC bars.
    pub fn rising_color(&self) -> Color {
        self.colors.series().rising()
    }

    /// The `SeriesStyle`'s colour for falling candlesticks and OHLC bars.
    pub fn falling_color(&self) -> Color {
        self.colors.series().falling()
    }

    /// The colour for marks drawn over the datasets' colours, such as error bars: the chart's
    /// label colour, so they stand out on any bar.
    pub fn ink(&self) -> Color {
        self.colors.series().ink()
    }

    /// Fills a rectangle from `side` to `side + thickness` along the index axis, and from `from`
    /// to `to` along the value axis: around a circle, the wedge between those angles and rings.
    pub fn fill_rect(&mut self, side: Scalar, thickness: Scalar, from: Scalar, to: Scalar, color: Color) {
        self.ctx.begin_path();
        self.frame.trace_rect(self.ctx, side, thickness, from, to);
        self.ctx.set_fill_style(color);
        self.ctx.fill();
    }

    /// Adds a whisker through the middle of `slot`, from the value `low` to the value `high`, with
    /// a cap at each end `cap_share` of the slot's thickness across, to the path `begin_lines`
    /// started. Along a side, it is straight along the value axis. Around a circle it follows the
    /// values: out along the wedge's middle spoke for upright bars, its caps across the spoke and as
    /// wide as that share of the wedge where they are; or around the ring through the bar's middle
    /// for horizontal bars, its caps across the ring.
    pub fn add_whisker(&mut self, slot: &BarSlot, low: Scalar, high: Scalar, cap_share: Scalar) {
        let middle = slot.middle();
        let (from, to) = (self.value_pixel(low), self.value_pixel(high));

        let BarFrame::Around(frame) = self.frame else {
            let cap = slot.thickness * cap_share / 2.0;
            self.add_line(Position::new(middle, from), Position::new(middle, to));
            self.add_line(Position::new(middle - cap, from), Position::new(middle + cap, from));
            self.add_line(Position::new(middle - cap, to), Position::new(middle + cap, to));
            return;
        };

        if frame.x_is_angle {
            // Out along the spoke: the caps are the share of the wedge's width at their distance.
            let cap = slot.thickness * cap_share * from.abs().max(to.abs());
            polar_whisker(self.ctx, frame, (middle, from), (middle, from), (middle, to), cap);
        } else {
            let cap = slot.thickness * cap_share;
            polar_whisker(self.ctx, frame, (from, middle), (from, middle), (to, middle), cap);
        }
    }

    /// Starts a path of straight lines, added one at a time with `add_line` and stroked together
    /// with `stroke_lines_added`, so a mark can draw many lines without collecting them first.
    pub fn begin_lines(&mut self) {
        self.ctx.begin_path();
    }

    /// Adds the straight line from `from` to `to`, in the index frame, to the path `begin_lines`
    /// started: around a circle, straight between the two points on the screen.
    pub fn add_line(&mut self, from: Position, to: Position) {
        match self.frame {
            BarFrame::Sides { orientation, .. } => {
                let mut path = Oriented { path: &mut *self.ctx, orientation };
                path.move_to(from);
                path.line_to(to);
            }
            BarFrame::Around(frame) => {
                let (from, to) = (frame_screen(frame, from), frame_screen(frame, to));
                PathSink::move_to(self.ctx, from);
                PathSink::line_to(self.ctx, to);
            }
        }
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
}

/// Where a position in a bar canvas's polar frame (see `BarCanvas`) is on the screen.
fn frame_screen(frame: &PolarFrame, position: Position) -> Position {
    if frame.x_is_angle {
        frame.polar.at(position.x, position.y)
    } else {
        frame.polar.at(position.y, position.x)
    }
}
