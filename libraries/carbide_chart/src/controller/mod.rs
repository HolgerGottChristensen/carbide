use std::fmt::Debug;
use dyn_clone::DynClone;
use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
pub use bar_controller::BarController;
pub use controller_sequence::ControllerSequence;
pub use line_controller::LineController;
pub use scatter_controller::ScatterController;
use crate::legend::LegendItem;
use crate::scale::{ScaleBounds, Scales};

mod bar_controller;
mod controller_sequence;
mod drawing;
mod line_path;
mod line_controller;
mod scatter_controller;
mod stacking;

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
    /// Syncs the states in the controller's datasets with the environment, and returns whether
    /// anything may have changed. The chart calls it before every update, render and event, as
    /// carbide syncs a widget's states, so `collect_bounds` and `draw` read synced values. The
    /// default syncs nothing: a controller of your own should sync its datasets here.
    fn sync(&mut self, _env: &mut Environment) -> bool {
        false
    }

    /// Reports what the controller needs from the scales its datasets are bound to: each
    /// dataset's data range (`ScaleBounds::include`), and anything its kind of chart asks of a
    /// scale, such as an offset, grid lines between ticks, or a category scale for the default x
    /// scale (see `ScaleBounds`). It runs on every fit, so the requests follow the controllers
    /// and datasets as they change. The datasets have been synced by then; `env` is for anything
    /// else the controller reads from the environment.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, env: &mut Environment);

    /// Draws the datasets inside `area`, each on the chart's fitted scales it is bound to.
    /// A dataset bound to a scale the chart doesn't have is skipped.
    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect);

    /// Adds an entry to `items` for each of the controller's datasets that has a name (see
    /// `DatasetExt::label`), in the colour it is drawn in, for the chart's legend (see
    /// `Chart::legend`). `env` holds the chart's `SeriesStyle`. The default adds nothing.
    fn legend_items(&self, _env: &Environment, _items: &mut Vec<LegendItem>) {}

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
    fn sync(&mut self, env: &mut Environment) -> bool {
        (**self).sync(env)
    }

    fn collect_bounds(&self, bounds: &mut ScaleBounds, env: &mut Environment) {
        (**self).collect_bounds(bounds, env);
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect) {
        (**self).draw(ctx, scales, area);
    }

    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        (**self).legend_items(env, items);
    }

    /// Already boxed: returned as it is, not boxed again.
    fn boxed(self) -> Box<dyn DatasetController> {
        self
    }
}
