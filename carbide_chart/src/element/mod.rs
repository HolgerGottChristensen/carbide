mod line;
mod cubic_interpolation_mode;
mod span_gaps;
mod stepped;

use carbide::environment::Environment;
use carbide_core::draw::Scalar;
use carbide_core::widget::canvas::CanvasContext;
pub use stepped::Stepped;

pub trait Element {
    fn x(&self) -> Scalar;
    fn y(&self) -> Scalar;

    fn draw(&self, ctx: &mut CanvasContext, env: &mut Environment);
}