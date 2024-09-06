use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::widget::EdgeInsets;
use carbide_core::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
pub use line_controller::LineController;
pub use scatter_controller::ScatterController;

mod line_controller;
mod scatter_controller;

pub trait DatasetController: Debug + Clone + 'static {
    fn draw(&self, ctx: &mut CanvasContext, padding: EdgeInsets);

    fn update_scales_min_max(&mut self);
}