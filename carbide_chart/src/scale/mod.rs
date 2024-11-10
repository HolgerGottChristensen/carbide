mod linear_scale;
mod percent_or_value;
mod bounds;

use std::fmt::Debug;
use std::ops::Range;
use carbide::color::{GREEN, RED, WHITE, YELLOW};
use carbide::draw::{Alignment, Rect};
use carbide::environment::EnvironmentStack;
use carbide::state::{ReadState, StateSync};
use carbide::widget::EdgeInsets;
use carbide_core::draw::{Dimension, Position, Scalar};
use carbide_core::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
pub use linear_scale::*;

pub trait Scale: Clone + Debug + 'static {
    fn axis(&self) -> Axis;

    fn draw(&self, ctx: &mut CanvasContext, env: &mut Environment, area: Rect) {
        self.draw_grid(ctx, area);
        self.draw_border(ctx, area);
    }

    fn draw_grid(&self, ctx: &mut CanvasContext, area: Rect) {
        if self.display_grid() {
            ctx.save();
            ctx.set_line_width(1.0);
            ctx.set_stroke_style(YELLOW);
            ctx.set_fill_style(YELLOW);


            let tick_offset = if self.display_ticks() { 10.0 } else { 0.0 };

            let ticks = self.ticks();
            let range = self.range();

            let difference = range.end - range.start;


            ctx.begin_path();


            match self.axis() {
                Axis::Horizontal => {
                    ctx.set_text_align(Alignment::Top);
                    for tick in ticks {
                        let x = (tick - range.start) / difference;
                        ctx.move_to(area.left() + x * area.width(), area.top() + tick_offset);
                        ctx.line_to(area.left() + x * area.width(), area.bottom());

                        let value = &format_tick(*tick, ctx.env_stack());

                        ctx.fill_text(value, area.left() + x * area.width(), area.top() + tick_offset);
                    }
                }
                Axis::Vertical => {
                    ctx.set_text_align(Alignment::Trailing);
                    for tick in ticks {
                        let y = 1.0 - ((tick - range.start) / difference);
                        ctx.move_to(area.left() - tick_offset, area.bottom() + y * area.height());
                        ctx.line_to(area.right(), area.bottom() + y * area.height());

                        let value = &format_tick(*tick, ctx.env_stack());

                        ctx.fill_text(value, area.left() - tick_offset, area.bottom() + y * area.height());
                    }
                }
                Axis::Radial => {}
            }

            ctx.stroke();

            ctx.restore();
        }
    }

    fn draw_border(&self, ctx: &mut CanvasContext, area: Rect) {

        match self.axis() {
            Axis::Horizontal => {
                ctx.save();

                ctx.begin_path();
                ctx.set_line_width(3.0);
                ctx.set_stroke_style(RED);
                ctx.move_to(area.left(), area.top());
                ctx.line_to(area.right(), area.top());
                ctx.stroke();

                ctx.restore();
            }
            Axis::Vertical => {
                ctx.save();

                ctx.begin_path();
                ctx.set_line_width(3.0);
                ctx.set_stroke_style(GREEN);
                ctx.move_to(area.left(), area.bottom());
                ctx.line_to(area.left(), area.top());
                ctx.stroke();

                ctx.restore();
            }
            Axis::Radial => {}
        }
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

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
    Radial,
}

#[cfg(feature = "carbide_fluent")]
fn format_tick(x: Scalar, env: &mut EnvironmentStack) -> String {
    use carbide_fluent::LocalizedNumber;

    let mut number = LocalizedNumber::new(x);
    number.sync(env);

    let x = number.value().to_string(); x
}

#[cfg(not(feature = "carbide_fluent"))]
fn format_tick(x: Scalar, env: &mut Environment) -> String {
    format!("{}", x)
}