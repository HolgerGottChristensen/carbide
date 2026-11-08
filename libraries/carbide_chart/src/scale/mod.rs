mod linear_scale;
mod percent_or_value;
mod bounds;
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

    /// Orders scales on the same side of the chart area, as Chart.js's `weight` does: a higher
    /// weight puts the scale further away from the chart area. Scales with the same weight keep
    /// the order they were added to the chart in, the first one closest to the area.
    fn weight(&self) -> i32 {
        0
    }

    fn draw(&self, ctx: &mut CanvasContext, _env: &mut Environment, area: Rect) {
        self.draw_grid(ctx, area, 0.0);
        self.draw_border(ctx, area, 0.0);
    }

    /// How far the scale's band reaches out from the chart area: its tick marks plus its widest
    /// tick label for a vertical scale, or its tallest for a horizontal one. Scales on the same
    /// side are stacked outwards, each in its own band.
    fn size(&self, ctx: &mut CanvasContext) -> Scalar {
        if !self.display_grid() {
            return 0.0;
        }

        let tick_length = if self.display_ticks() { TICK_LENGTH } else { 0.0 };

        let mut label_extent: Scalar = 0.0;
        for tick in self.ticks() {
            let label = format_tick(*tick, ctx.env());
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

    fn draw_grid(&self, ctx: &mut CanvasContext, area: Rect, offset: Scalar) {
        if self.display_grid() {
            ctx.save();
            ctx.set_line_width(1.0);
            ctx.set_stroke_style(YELLOW);
            ctx.set_fill_style(YELLOW);


            let tick_offset = if self.display_ticks() { TICK_LENGTH } else { 0.0 };

            let ticks = self.ticks();
            let range = self.range();

            let difference = range.end - range.start;


            ctx.begin_path();


            match (self.axis(), self.axis_position()) {
                (Axis::Horizontal, AxisPosition::Top) => {
                    let band_edge = area.bottom() - offset;
                    ctx.set_text_align(Alignment::Bottom);
                    for tick in ticks {
                        let x = area.left() + (tick - range.start) / difference * area.width();
                        ctx.move_to(x, band_edge - tick_offset);
                        if offset > 0.0 {
                            ctx.line_to(x, band_edge);
                            ctx.move_to(x, area.bottom());
                        }
                        ctx.line_to(x, area.top());

                        let value = &format_tick(*tick, ctx.env());

                        ctx.fill_text(value, x, band_edge - tick_offset);
                    }
                }
                (Axis::Horizontal, _) => {
                    let band_edge = area.top() + offset;
                    ctx.set_text_align(Alignment::Top);
                    for tick in ticks {
                        let x = area.left() + (tick - range.start) / difference * area.width();
                        ctx.move_to(x, band_edge + tick_offset);
                        if offset > 0.0 {
                            ctx.line_to(x, band_edge);
                            ctx.move_to(x, area.top());
                        }
                        ctx.line_to(x, area.bottom());

                        let value = &format_tick(*tick, ctx.env());

                        ctx.fill_text(value, x, band_edge + tick_offset);
                    }
                }
                (Axis::Vertical, AxisPosition::Right) => {
                    let band_edge = area.right() + offset;
                    ctx.set_text_align(Alignment::Leading);
                    for tick in ticks {
                        let y = area.bottom() + (1.0 - ((tick - range.start) / difference)) * area.height();
                        ctx.move_to(band_edge + tick_offset, y);
                        if offset > 0.0 {
                            ctx.line_to(band_edge, y);
                            ctx.move_to(area.right(), y);
                        }
                        ctx.line_to(area.left(), y);

                        let value = &format_tick(*tick, ctx.env());

                        ctx.fill_text(value, band_edge + tick_offset, y);
                    }
                }
                (Axis::Vertical, _) => {
                    let band_edge = area.left() - offset;
                    ctx.set_text_align(Alignment::Trailing);
                    for tick in ticks {
                        let y = area.bottom() + (1.0 - ((tick - range.start) / difference)) * area.height();
                        ctx.move_to(band_edge - tick_offset, y);
                        if offset > 0.0 {
                            ctx.line_to(band_edge, y);
                            ctx.move_to(area.left(), y);
                        }
                        ctx.line_to(area.right(), y);

                        let value = &format_tick(*tick, ctx.env());

                        ctx.fill_text(value, band_edge - tick_offset, y);
                    }
                }
                (Axis::Radial, _) => {}
            }

            ctx.stroke();

            ctx.restore();
        }
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