use std::fmt::Debug;
use dyn_clone::DynClone;
use carbide::draw::{Dimension, Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
pub use bar_controller::BarController;
pub(crate) use bar_controller::bar_slot;
pub use controller_sequence::ControllerSequence;
pub use line_controller::LineController;
pub use pie_controller::{PieController, PiePart, DEFAULT_OUTLINE_WIDTH, DEFAULT_RING_SPACING};
pub use scatter_controller::ScatterController;
pub use orientation::Orientation;
use crate::{DataSet, SeriesColors};
use crate::legend::{LegendItem, LegendMarker};
use drawing::describe_point;
use crate::scale::{ScaleBounds, Scales};
use crate::geometry::{DatumKey, Shapes};
use crate::interaction::DatumInfo;

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
pub(crate) mod options;
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
    /// dataset's data range, and anything its kind of chart asks of a scale, such as an offset,
    /// grid lines between ticks, or a category scale for the default x scale (see `ScaleBounds`).
    /// It runs on every fit, so the requests follow the controllers and datasets as they change.
    /// The datasets have been synced by then; `env` is for anything else the controller reads from
    /// the environment.
    ///
    /// A controller first registers each dataset's values (`ScaleBounds::register_values`), so
    /// names become categories and dates pick a time scale, then splits the bounds
    /// (`ScaleBounds::split`), reads each dataset's points through the placement
    /// (`Placement::coordinates`), and includes them in the requests. Including a point with
    /// `ScaleRequests::include_point` leaves its value out of a scale fitting to the visible data
    /// when it is outside the zoom window, as the built-in controllers do. A hidden dataset
    /// (`DataSetOptions::visible`) is registered but adds no bounds.
    ///
    /// ```
    /// use carbide_chart::{DataSet, ScaleBounds};
    ///
    /// /// Collects `dataset`'s points, each on its x and y scale.
    /// fn collect(dataset: &dyn DataSet, bounds: &mut ScaleBounds) {
    ///     let options = dataset.options();
    ///     bounds.register_values(dataset, &options.x_scale, &options.y_scale);
    ///     if !options.visible {
    ///         return;
    ///     }
    ///
    ///     let (placement, requests) = bounds.split();
    ///     let coordinates = placement.coordinates(&options.x_scale, &options.y_scale);
    ///     dataset.points(&mut |_, point| {
    ///         if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
    ///             requests.include_point(&options.x_scale, &options.y_scale, x, y);
    ///         }
    ///     });
    /// }
    /// ```
    fn collect_bounds(&self, bounds: &mut ScaleBounds, env: &mut Environment);

    /// Draws the datasets inside `area`, each on the chart's fitted scales it is bound to.
    /// A dataset bound to a scale the chart doesn't have is skipped.
    ///
    /// A controller reads each dataset's points through `Scales::coordinates`, which places names
    /// on their categories and leaves out values a scale has no place for, and maps them to the
    /// screen with `Scales::pixel`, or, in the index frame of a controller that can be drawn
    /// horizontally, with `Orientation`. A dataset on two radial scales is drawn around the
    /// chart's circle: `Scales::polar` gives the circle for its angle and radius scales, and
    /// `Polar::at` the position at an angle (`Polar::angle` of the angle scale's normalized value)
    /// and a distance from the centre (`Polar::distance` of the radius scale's).
    ///
    /// For each point it draws, a controller adds its shape to `shapes` (`Shapes::add`, or
    /// `Shapes::add_line` for a line through a dataset's points), so the chart can find the point
    /// under or near the pointer, for hovering, tooltips and clicks; a point without a shape isn't
    /// found. A hidden dataset (`DataSetOptions::visible`) is neither drawn nor added.
    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect, shapes: &mut Shapes);

    /// Calls `f` with each of the controller's datasets and its index in the controller, shown or
    /// hidden, so the provided `legend_items`, `legend_visibility` and `describe` can read them.
    /// The default visits none, and those then add no entries, toggle nothing and describe
    /// nothing. A controller of your own whose datasets are a `Sequence<dyn DataSet>` visits them
    /// with `self.datasets.foreach(f)`.
    fn visit_datasets(&self, _f: &mut dyn FnMut(usize, &dyn DataSet)) {}

    /// The marker of the dataset at `dataset_index`, in its legend entry and in tooltips, for the
    /// provided `legend_items` and `describe`. The default is a square.
    fn legend_marker(&self, _dataset_index: usize) -> LegendMarker {
        LegendMarker::Square
    }

    /// Which way the controller draws: upright, its datasets' values up their y scale, or
    /// horizontal, along x, for the provided `describe`, which reads a datum's index and value off
    /// its index and value scales (see `Orientation::index_and_value_scales`). The default is
    /// upright.
    fn orientation(&self) -> Orientation {
        Orientation::Vertical
    }

    /// How many colours into the palette the controller's datasets start (see
    /// `SeriesColors::new`), for the provided `legend_items` and `describe`. The default is 0.
    fn palette_offset(&self) -> usize {
        0
    }

    /// Adds an entry to `items` for each of the controller's datasets that has a name (see
    /// `DatasetExt::label`), in the colour it is drawn in, for the chart's legend (see
    /// `Chart::legend`). `env` holds the chart's `SeriesStyle`.
    ///
    /// The default adds an entry for each named dataset `visit_datasets` visits, in its colour
    /// (see `SeriesColors`, with `palette_offset`), with its `legend_marker`, dimmed when the
    /// dataset is hidden.
    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        let colors = SeriesColors::new(env, self.palette_offset());
        self.visit_datasets(&mut |dataset_index, dataset| {
            let options = dataset.options();
            if let Some(label) = options.label.clone() {
                let color = colors.dataset(options.color, dataset_index).color();
                items.push(LegendItem { label, color, marker: self.legend_marker(dataset_index), hidden: !options.visible });
            }
        });
    }

    /// Shows or hides what legend entry `entry` stands for, counting the controller's entries in
    /// the order `legend_items` adds them, as a click on the entry does (see
    /// `Chart::legend_toggles`), and returns whether it could. The built-in controllers show or
    /// hide the entry's dataset (`DataSet::set_visible`), or for a pie its part.
    ///
    /// The default shows or hides the named dataset behind the entry, among those
    /// `visit_datasets` visits, counted as the provided `legend_items` adds them; with none, it
    /// can't, and returns `false`, so the entry doesn't toggle.
    fn legend_visibility(&self, entry: usize, visible: bool) -> bool {
        let (mut named, mut set) = (0, false);
        self.visit_datasets(&mut |_, dataset| {
            if dataset.options().label.is_none() {
                return;
            }
            if named == entry {
                set = dataset.set_visible(visible);
            }
            named += 1;
        });
        set
    }

    /// Describes the datum `key` (its dataset and index; its controller is this one) for a
    /// tooltip and for `Pick::items`: its dataset's name, colour and legend marker, its index value
    /// and its values as text, labelled by the fitted `scales`, or `None` to leave it out.
    ///
    /// The default describes the point at `key.index` of the dataset at `key.dataset` that
    /// `visit_datasets` visits, as the built-in controllers do: its index value and its values
    /// (`DataPoint::describe`) labelled by its index and value scales as `orientation` reads them,
    /// in its colour (with `palette_offset`) and with its dataset's `legend_marker`. `None` for a
    /// hidden dataset, a datum it doesn't have, or a controller that visits no datasets.
    fn describe(&self, key: DatumKey, scales: &Scales, env: &mut Environment) -> Option<DatumInfo> {
        let colors = SeriesColors::new(env, self.palette_offset());
        let mut found = None;
        self.visit_datasets(&mut |dataset_index, dataset| {
            if dataset_index == key.dataset {
                found = describe_point(dataset, key, scales, self.orientation(), &colors, env, None, self.legend_marker(dataset_index));
            }
        });
        found
    }

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

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect, shapes: &mut Shapes) {
        (**self).draw(ctx, scales, area, shapes);
    }

    fn visit_datasets(&self, f: &mut dyn FnMut(usize, &dyn DataSet)) {
        (**self).visit_datasets(f);
    }

    fn legend_marker(&self, dataset_index: usize) -> LegendMarker {
        (**self).legend_marker(dataset_index)
    }

    fn orientation(&self) -> Orientation {
        (**self).orientation()
    }

    fn palette_offset(&self) -> usize {
        (**self).palette_offset()
    }

    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        (**self).legend_items(env, items);
    }

    fn legend_visibility(&self, entry: usize, visible: bool) -> bool {
        (**self).legend_visibility(entry, visible)
    }

    fn describe(&self, key: DatumKey, scales: &Scales, env: &mut Environment) -> Option<DatumInfo> {
        (**self).describe(key, scales, env)
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
