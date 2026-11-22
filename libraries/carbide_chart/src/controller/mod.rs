use std::fmt::Debug;
use dyn_clone::DynClone;
use carbide::draw::{Dimension, Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
pub use bar_controller::BarController;
pub(crate) use bar_controller::bar_slot;
pub use controller_sequence::ControllerSequence;
pub use line_controller::LineController;
pub use pie_controller::{PieController, PiePart, DEFAULT_RING_SPACING};
pub use scatter_controller::ScatterController;
use crate::legend::LegendItem;
use crate::scale::{ScaleBounds, Scales};

mod bar_controller;
mod controller_sequence;
pub(crate) mod band;
pub(crate) mod drawing;
pub(crate) mod line_path;
mod line_controller;
mod pie_controller;
pub(crate) mod orientation;
pub(crate) mod polar_path;
mod scatter_controller;
pub(crate) mod stacking;

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

    /// Whether the controller draws around a circle unless told otherwise, so a chart whose first
    /// controller it is makes its default x and y scales radial, as a pie controller does. The
    /// default is `false`: default scales run along the sides.
    fn prefers_polar(&self) -> bool {
        false
    }

    /// Calls `f` with each label the controller draws outside a polar chart's circle, its screen
    /// angle (see `Polar`), its size, and how far out from the rim it starts, so the chart can make
    /// room for it when it fits the circle, as it does for the angle scale's labels. `measure` gives
    /// a text's size. The scales are fitted; the circle isn't yet. The default has none.
    fn polar_labels(&self, _scales: &Scales, _measure: &mut dyn FnMut(&str) -> Dimension, _f: &mut dyn FnMut(Scalar, Dimension, Scalar)) {}

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

    fn prefers_polar(&self) -> bool {
        (**self).prefers_polar()
    }

    fn polar_labels(&self, scales: &Scales, measure: &mut dyn FnMut(&str) -> Dimension, f: &mut dyn FnMut(Scalar, Dimension, Scalar)) {
        (**self).polar_labels(scales, measure, f);
    }

    /// Already boxed: returned as it is, not boxed again.
    fn boxed(self) -> Box<dyn DatasetController> {
        self
    }
}
