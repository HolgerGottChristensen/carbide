use std::fmt::Debug;
use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
pub use controller_sequence::ControllerSequence;
pub use line_controller::LineController;
pub use scatter_controller::ScatterController;
use crate::scale::{ScaleBounds, Scales};

mod controller_sequence;
mod line_controller;
mod scatter_controller;

/// Draws a group of datasets on the scales the chart owns.
pub trait DatasetController: Debug + 'static {
    /// Reports the data range of every dataset to the bounds of the scales it is bound to.
    /// `env` is for reading the datasets' options, which can be states.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, env: &mut Environment);

    /// Draws the datasets inside `area`, each on the chart's fitted scales it is bound to.
    /// A dataset bound to a scale the chart doesn't have is skipped.
    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect);
}
