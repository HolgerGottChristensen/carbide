use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::scale::{ScaleBounds, Scales};

/// Placeholder for a line controller. It has no datasets yet, so it reports no bounds and draws
/// nothing.
#[derive(Clone, Debug)]
pub struct LineController;

impl LineController {
    pub fn new() -> LineController {
        LineController
    }
}

impl Default for LineController {
    fn default() -> Self {
        Self::new()
    }
}

impl DatasetController for LineController {
    fn collect_bounds(&self, _bounds: &mut ScaleBounds, _env: &mut Environment) {}

    fn draw(&self, _ctx: &mut CanvasContext, _scales: &Scales, _area: Rect) {}
}
