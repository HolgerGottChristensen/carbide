mod category_scale;
mod linear_scale;
mod scales;
mod scale_id;

use std::fmt::Debug;
use dyn_clone::DynClone;
use std::ops::Range;
use carbide::color::{GREEN, RED, YELLOW};
use carbide::draw::{Alignment, Position, Rect};
use carbide::environment::Environment;
use carbide::state::{ReadState, StateSync};
use carbide_core::draw::Scalar;
use carbide_core::widget::canvas::CanvasContext;
pub use category_scale::CategoryScale;
pub use linear_scale::*;
pub use scales::*;
pub use scale_id::ScaleId;

/// How far tick marks reach out from the inner edge of their scale's band.
const TICK_LENGTH: Scalar = 10.0;

/// A scale maps data values to positions along an axis. Charts own their scales as
/// `Box<dyn Scale>`, so different kinds of scales can sit side by side.
pub trait Scale: DynClone + Debug + 'static {
    fn axis(&self) -> Axis;

    /// The side of the chart area the scale's ticks, labels and border are drawn on, and where
    /// the chart reserves space for them.
    fn axis_position(&self) -> AxisPosition {
        self.axis().default_position()
    }

    /// Orders scales on the same side of the chart area: a higher weight puts the scale further
    /// away from the chart area. Scales with the same weight keep the order they were added to the
    /// chart in, the first one closest to the area.
    fn weight(&self) -> i32 {
        0
    }

    /// Where `value` falls along the scale, from 0 at the start of the chart area to 1 at its
    /// end. Controllers and the grid map values to positions through this, so a scale's offset
    /// (see `set_offset`) applies to everything drawn on it.
    fn normalize(&self, value: Scalar) -> Scalar {
        let range = self.range();
        (value - range.start) / (range.end - range.start)
    }

    /// Pads the mapping by half a tick step at both ends, so values on the first and last tick sit
    /// inside the chart area instead of on its edges. The chart sets it on every fit, for the
    /// scales a controller requested it on (see `ScaleBounds::request_offset`); bar charts use it
    /// so their first and last bars aren't cut in half. Scales that don't support it ignore it.
    fn set_offset(&mut self, _offset: bool) {}

    /// The text shown for the tick at `value`: the number, formatted for the environment's
    /// locale. A category scale shows its labels instead.
    fn tick_label(&self, value: Scalar, env: &mut Environment) -> String {
        format_tick(value, env)
    }

    /// Where grid lines and tick marks are drawn: on the ticks themselves, unless the scale puts
    /// them between ticks instead (see `CategoryScale::grid_offset`). Labels are always drawn on
    /// the ticks.
    fn grid_line_values(&self) -> Vec<Scalar> {
        self.ticks().to_vec()
    }

    fn draw(&self, ctx: &mut CanvasContext, _env: &mut Environment, area: Rect) {
        self.draw_grid(ctx, area, 0.0);
        self.draw_border(ctx, area, 0.0);
    }

    /// How far the scale's band reaches out from the chart area: its tick marks plus its widest
    /// tick label for a vertical scale, or its tallest for a horizontal one. Scales on the same
    /// side are stacked outwards, each in its own band.
    fn size(&self, ctx: &mut CanvasContext) -> Scalar {
        if self.axis() == Axis::Radial {
            return 0.0;
        }

        let tick_length = if self.display_ticks() { TICK_LENGTH } else { 0.0 };

        let mut label_extent: Scalar = 0.0;
        for tick in self.ticks() {
            let label = self.tick_label(*tick, ctx.env());
            let size = ctx.measure_text(&label);

            label_extent = label_extent.max(match self.axis() {
                Axis::Horizontal => size.height,
                Axis::Vertical | Axis::Radial => size.width,
            });
        }

        tick_length + label_extent
    }

    // In the methods below, `offset` is the distance from the chart area to the inner edge of the
    // scale's band (0 for the scale closest to the area). Grid lines span the chart area; tick
    // marks, labels and the border sit in the band.
    //
    // `Rect`'s y grows downwards on screen, so `area.bottom()` is the screen's top edge of the
    // area and `area.top()` its bottom edge.

    /// Draws the scale's grid lines across the chart area (if `display_grid`), its tick marks in
    /// its band (if `display_ticks`), and its tick labels. Turning the grid lines off keeps the
    /// labels, which is what secondary scales usually want.
    fn draw_grid(&self, ctx: &mut CanvasContext, area: Rect, offset: Scalar) {
        // Per side: the chart area's edge the band starts at, which way is outwards across the
        // axis (+1 or -1), the opposite edge the grid lines run to, and how labels hang off the
        // end of the tick marks.
        let (horizontal, area_edge, outward, far_edge, alignment) = match (self.axis(), self.axis_position()) {
            (Axis::Horizontal, AxisPosition::Top) => (true, area.bottom(), -1.0, area.top(), Alignment::Bottom),
            (Axis::Horizontal, _) => (true, area.top(), 1.0, area.bottom(), Alignment::Top),
            (Axis::Vertical, AxisPosition::Right) => (false, area.right(), 1.0, area.left(), Alignment::Leading),
            (Axis::Vertical, _) => (false, area.left(), -1.0, area.right(), Alignment::Trailing),
            (Axis::Radial, _) => return,
        };

        let grid_lines = self.display_grid();
        let tick_marks = self.display_ticks();
        let tick_length = if tick_marks { TICK_LENGTH } else { 0.0 };

        let band_edge = area_edge + outward * offset;
        let tick_end = band_edge + outward * tick_length;

        ctx.save();
        ctx.set_line_width(1.0);
        ctx.set_stroke_style(YELLOW);
        ctx.set_fill_style(YELLOW);
        ctx.set_text_align(alignment);

        // `along` is a value's coordinate along the axis; `across` a coordinate across it.
        let along = |value: Scalar| {
            let fraction = self.normalize(value);

            if horizontal {
                area.left() + fraction * area.width()
            } else {
                area.bottom() + (1.0 - fraction) * area.height()
            }
        };
        let point = |along: Scalar, across: Scalar| if horizontal { (along, across) } else { (across, along) };

        ctx.begin_path();

        for value in self.grid_line_values() {
            let fraction = self.normalize(value);

            // Lines that would fall outside the chart area are skipped.
            if !(-1e-9..=1.0 + 1e-9).contains(&fraction) {
                continue;
            }

            let along = along(value);
            let point = |across: Scalar| point(along, across);

            // Drawn as one segment from the tick mark's end through the chart area when the band
            // touches the area, and as two separate segments when it is stacked further out.
            let mut pen_down = false;

            if tick_marks {
                let (x, y) = point(tick_end);
                ctx.move_to(x, y);
                pen_down = true;

                if offset > 0.0 || !grid_lines {
                    let (x, y) = point(band_edge);
                    ctx.line_to(x, y);
                    pen_down = false;
                }
            }

            if grid_lines {
                if !pen_down {
                    let (x, y) = point(area_edge);
                    ctx.move_to(x, y);
                }

                let (x, y) = point(far_edge);
                ctx.line_to(x, y);
            }
        }

        for tick in self.ticks() {
            let label = &self.tick_label(*tick, ctx.env());
            let (x, y) = point(along(*tick), tick_end);
            ctx.fill_text(label, x, y);
        }

        ctx.stroke();

        ctx.restore();
    }

    fn draw_border(&self, ctx: &mut CanvasContext, area: Rect, offset: Scalar) {
        let (color, from, to) = match (self.axis(), self.axis_position()) {
            (Axis::Horizontal, AxisPosition::Top) => {
                (RED, Position::new(area.left(), area.bottom() - offset), Position::new(area.right(), area.bottom() - offset))
            }
            (Axis::Horizontal, _) => {
                (RED, Position::new(area.left(), area.top() + offset), Position::new(area.right(), area.top() + offset))
            }
            (Axis::Vertical, AxisPosition::Right) => {
                (GREEN, Position::new(area.right() + offset, area.bottom()), Position::new(area.right() + offset, area.top()))
            }
            (Axis::Vertical, _) => {
                (GREEN, Position::new(area.left() - offset, area.bottom()), Position::new(area.left() - offset, area.top()))
            }
            (Axis::Radial, _) => return,
        };

        ctx.save();

        ctx.begin_path();
        ctx.set_line_width(3.0);
        ctx.set_stroke_style(color);
        ctx.move_to(from.x, from.y);
        ctx.line_to(to.x, to.y);
        ctx.stroke();

        ctx.restore();
    }

    fn min(&self) -> Scalar;
    fn max(&self) -> Scalar;
    fn set_range(&mut self, min: Scalar, max: Scalar);
    fn display_ticks(&self) -> bool;
    fn display_grid(&self) -> bool;
    fn recalculate_ticks(&mut self);
    fn ticks(&self) -> &[Scalar];
    fn range(&self) -> Range<Scalar>;
}

dyn_clone::clone_trait_object!(Scale);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
    Radial,
}

impl Axis {
    /// Where a scale on this axis sits unless told otherwise: below the chart area for
    /// horizontal scales, left of it for the others.
    pub fn default_position(&self) -> AxisPosition {
        match self {
            Axis::Horizontal => AxisPosition::Bottom,
            Axis::Vertical | Axis::Radial => AxisPosition::Left,
        }
    }

    /// Whether a scale on this axis can sit at `position`: horizontal scales above or below the
    /// chart area, vertical scales left or right of it.
    pub fn allows(&self, position: AxisPosition) -> bool {
        match self {
            Axis::Horizontal => matches!(position, AxisPosition::Top | AxisPosition::Bottom),
            Axis::Vertical => matches!(position, AxisPosition::Left | AxisPosition::Right),
            Axis::Radial => position == self.default_position(),
        }
    }
}

/// The side of the chart area a scale is drawn on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AxisPosition {
    Left,
    Right,
    Top,
    Bottom,
}

#[cfg(feature = "carbide_fluent")]
fn format_tick(x: Scalar, env: &mut Environment) -> String {
    use carbide_fluent::LocalizedNumber;

    let mut number = LocalizedNumber::new(x);
    number.sync(env);

     number.value().to_string()
}

#[cfg(not(feature = "carbide_fluent"))]
fn format_tick(x: Scalar, env: &mut Environment) -> String {
    format!("{}", x)
}