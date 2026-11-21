use std::fmt::Debug;
use carbide::color::Color;
use carbide::draw::{Position, Rect, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataSetOptions};
use crate::controller::drawing::{point_color, Palette};
use crate::controller::line_path::PathSink;
use crate::controller::orientation::{pixel, Oriented, Orientation};
use crate::mark::MarkRole;
use crate::scale::Scale;
use crate::style::axis_style;

/// A mark that can be drawn in a bar controller's slots, for datasets of type `D`. Implementing
/// it is what lets a dataset with this mark be drawn by a `BarController` (see `BarDataSet`).
///
/// The controller lays the slots out, grouping and orienting them as it does for bars, and the
/// mark draws each of its points in the slot for the point's x.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be drawn in a bar controller's slots for `{D}`",
    label = "not a bar mark for this dataset",
    note = "bar marks are `RangeMark` (`.range()`) and `ErrorBars` (`.error_bars()`) on datasets of `RangePoint`s, and types of your own implementing `BarMark`",
)]
pub trait BarMark<D: ?Sized>: Clone + Debug + 'static {
    /// Whether the mark replaces the bar or is drawn on top of it.
    fn role(&self) -> MarkRole;

    /// Calls `f` with each point's x and the lowest and highest value the mark draws for it, for
    /// the scale bounds. Points the mark doesn't draw are left out.
    fn bounds(&self, data: &D, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

    /// Draws the mark for each of the dataset's points, in the slot for its x (`BarSlots::slot`).
    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas);
}

/// A bar mark together with the dataset it draws, as a bar controller reads it from a dataset's
/// `BarDataSet::bar_marks`.
pub trait DatasetBarMark {
    fn role(&self) -> MarkRole;

    fn bounds(&self, f: &mut dyn FnMut(Scalar, Scalar, Scalar));

    fn draw(&self, slots: &BarSlots, canvas: &mut BarCanvas);
}

pub(crate) struct BoundBarMark<'a, M, D> {
    pub(crate) mark: &'a M,
    pub(crate) data: &'a D,
}

impl<M: BarMark<D>, D> DatasetBarMark for BoundBarMark<'_, M, D> {
    fn role(&self) -> MarkRole {
        self.mark.role()
    }

    fn bounds(&self, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        self.mark.bounds(self.data, f);
    }

    fn draw(&self, slots: &BarSlots, canvas: &mut BarCanvas) {
        self.mark.draw(self.data, slots, canvas);
    }
}

/// Where a point's glyph goes along the index axis, in pixels: the centre of its category, and
/// the dataset's share of the category's slot, from `side`, `thickness` long. Share 0 is at the
/// smaller pixel: the left, or the top for lying bars.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BarSlot {
    pub center: Scalar,
    pub side: Scalar,
    pub thickness: Scalar,
}

impl BarSlot {
    /// The middle of the dataset's share, where a whisker or a line through the bars goes.
    pub fn middle(&self) -> Scalar {
        self.side + self.thickness / 2.0
    }
}

/// The slots of one dataset in a bar controller, by x value.
pub struct BarSlots<'a> {
    pub(crate) orientation: Orientation,
    pub(crate) index_scale: &'a dyn Scale,
    pub(crate) area: Rect,
    pub(crate) ruler: Scalar,
    pub(crate) group_index: usize,
    pub(crate) group_size: usize,
    pub(crate) category_percentage: Scalar,
    pub(crate) bar_percentage: Scalar,
}

impl BarSlots<'_> {
    /// The slot for a point at `x`, laid out as the controller lays out its bars.
    pub fn slot(&self, x: Scalar) -> BarSlot {
        let center = pixel(self.orientation.index_axis(), self.index_scale, x, self.area);
        let (side, thickness) = crate::controller::bar_slot(center, self.ruler, self.group_index, self.group_size, self.category_percentage, self.bar_percentage);

        BarSlot { center, side, thickness }
    }
}

/// What a bar mark draws on. Positions are in the index frame: x along the categories and y along
/// the values, both in pixels, so a mark draws the same code for standing and lying bars.
pub struct BarCanvas<'x, 'a, 'b, 'c: 'b> {
    pub(crate) ctx: &'x mut CanvasContext<'a, 'b, 'c>,
    pub(crate) orientation: Orientation,
    pub(crate) value_scale: &'x dyn Scale,
    pub(crate) area: Rect,
    pub(crate) options: &'x DataSetOptions,
    pub(crate) dataset_index: usize,
    pub(crate) palette: &'x Palette,
}

impl BarCanvas<'_, '_, '_, '_> {
    /// Where `value` on the dataset's value scale falls along the value axis, in pixels.
    pub fn value_pixel(&self, value: Scalar) -> Scalar {
        pixel(self.orientation.value_axis(), self.value_scale, value, self.area)
    }

    /// The colour of the point at `point_index`, whose own colour is `color` (see
    /// `DataPoint::color`), as a bar would be drawn in.
    pub fn point_color(&self, point_index: usize, color: DataColor) -> Color {
        point_color(color, self.options, self.dataset_index, point_index, self.palette)
    }

    /// The colour for marks drawn over the datasets' colours, such as error bars: the chart's
    /// label colour, so they stand out on any bar.
    pub fn ink(&mut self) -> Color {
        let env = self.ctx.env();
        axis_style(env).label_color(env)
    }

    /// Fills a rectangle from `side` to `side + thickness` along the index axis, and from `from`
    /// to `to` along the value axis.
    pub fn fill_rect(&mut self, side: Scalar, thickness: Scalar, from: Scalar, to: Scalar, color: Color) {
        self.ctx.begin_path();
        let mut path = Oriented { path: &mut *self.ctx, orientation: self.orientation };
        path.move_to(Position::new(side, from));
        path.line_to(Position::new(side + thickness, from));
        path.line_to(Position::new(side + thickness, to));
        path.line_to(Position::new(side, to));
        self.ctx.close_path();
        self.ctx.set_fill_style(color);
        self.ctx.fill();
    }

    /// Strokes the straight lines between each pair of `segments`, `width` wide.
    pub fn stroke_lines(&mut self, segments: &[(Position, Position)], width: Scalar, color: Color) {
        self.ctx.save();
        self.ctx.begin_path();
        let mut path = Oriented { path: &mut *self.ctx, orientation: self.orientation };
        for (from, to) in segments {
            path.move_to(*from);
            path.line_to(*to);
        }
        self.ctx.set_line_width(width);
        self.ctx.set_stroke_style(color);
        self.ctx.stroke();
        self.ctx.restore();
    }
}
