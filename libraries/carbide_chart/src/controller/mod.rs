use std::fmt::Debug;
use dyn_clone::DynClone;
use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
pub use bar_controller::BarController;
pub use controller_sequence::ControllerSequence;
pub use line_controller::LineController;
pub use scatter_controller::ScatterController;
use crate::scale::{Scale, ScaleBounds, ScaleId, Scales};

mod bar_controller;
mod controller_sequence;
mod drawing;
mod line_controller;
mod scatter_controller;

/// Draws a group of datasets on the scales the chart owns.
///
/// Controllers of different types can be mixed in a tuple, or, boxed with `.boxed()`, in a `Vec`
/// or a state of a `Vec`: `vec![bars.boxed(), lines.boxed()]`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a chart controller",
    label = "not a controller",
    note = "controllers are `BarController`, `LineController`, `ScatterController`, boxed controllers (`.boxed()`), and types implementing `DatasetController`",
)]
pub trait DatasetController: DynClone + Debug + 'static {
    /// Reports the data range of every dataset to the bounds of the scales it is bound to.
    /// `env` is for reading the datasets' options, which can be states.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, env: &mut Environment);

    /// Draws the datasets inside `area`, each on the chart's fitted scales it is bound to.
    /// A dataset bound to a scale the chart doesn't have is skipped.
    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect);

    /// The scale this kind of chart wants for the default scale `id` (`ScaleId::X` or
    /// `ScaleId::Y`), like the per-type `overrides.scales` in Chart.js: a bar chart wants a
    /// category scale along x. `Chart::new` asks its controllers in order and uses the first
    /// answer; scales set with `Chart::scale` replace it. `None` keeps the default linear scale.
    fn default_scale(&self, _id: &ScaleId) -> Option<Box<dyn Scale>> {
        None
    }

    /// Boxes the controller, so controllers of different types can share a `Vec` or a state of
    /// a `Vec`, as the controllers of a chart.
    fn boxed(self) -> Box<dyn DatasetController> where Self: Sized {
        Box::new(self)
    }
}

dyn_clone::clone_trait_object!(DatasetController);

/// A boxed controller is a controller, so a `Vec<Box<dyn DatasetController>>` is a sequence of
/// controllers of mixed types.
impl DatasetController for Box<dyn DatasetController> {
    fn collect_bounds(&self, bounds: &mut ScaleBounds, env: &mut Environment) {
        (**self).collect_bounds(bounds, env);
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect) {
        (**self).draw(ctx, scales, area);
    }

    fn default_scale(&self, id: &ScaleId) -> Option<Box<dyn Scale>> {
        (**self).default_scale(id)
    }

    /// Already boxed: returned as it is, not boxed again.
    fn boxed(self) -> Box<dyn DatasetController> {
        self
    }
}
