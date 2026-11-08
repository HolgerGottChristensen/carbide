use std::collections::HashSet;
use carbide::environment::Environment;
use carbide::lifecycle::{Update, UpdateContext};
use carbide::widget::EdgeInsets;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::render::{Render, RenderContext};
use carbide_core::widget::{CommonWidget, WidgetId};
use carbide_core::widget::canvas::CanvasContext;
use carbide_derive::Widget;
use crate::controller::ControllerSequence;
use crate::scale::{Axis, ScaleBounds, ScaleId, Scales};

/// A container that owns the scales and draws a sequence of controllers on them. Each dataset
/// is drawn on the scales it is bound to (`ScaleId::X` and `ScaleId::Y` by default), so datasets
/// in different controllers share a scale when they are bound to the same id.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Update)]
pub struct Chart<C> where C: ControllerSequence {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    controllers: C,
    scales: Scales,
    padding: EdgeInsets,
    /// Unknown scale ids already warned about, so each is logged once, not every update.
    warned_scale_ids: HashSet<ScaleId>,
}

impl<C: ControllerSequence> Chart<C> {
    pub fn new(controllers: C) -> Chart<C> {
        Chart {
            id: Default::default(),
            position: Default::default(),
            dimension: Default::default(),
            controllers,
            scales: Scales::new(),
            padding: EdgeInsets::all(30.0),
            warned_scale_ids: HashSet::new(),
        }
    }

    /// Collects the bounds every controller reports for each scale id, then fits each scale to
    /// the bounds collected for it. Data bound to an id the chart has no scale for is not drawn;
    /// it is logged once per id.
    fn fit_scales(&mut self, env: &mut Environment) {
        let mut bounds = ScaleBounds::default();

        self.controllers.controllers(&mut |controller| {
            controller.collect_bounds(&mut bounds, env);
        });

        for id in bounds.ids() {
            if !self.scales.contains(id) && self.warned_scale_ids.insert(id) {
                log::warn!("Chart has no scale with id {:?}; datasets bound to it are not drawn", id.name());
            }
        }

        self.scales.fit(&bounds);
    }

    /// The area the datasets are drawn in: the chart's dimension minus the padding and the space
    /// for the tick labels. Vertical scales' labels sit left of the area, horizontal scales'
    /// below it (see `Scale::draw_grid`).
    fn chart_area(&self) -> Rect {
        let has_ticks = |axis: Axis| self.scales.iter()
            .any(|(_, scale)| scale.axis() == axis && scale.display_ticks());

        let left_tick_gutter = if has_ticks(Axis::Vertical) { 10.0 } else { 0.0 };
        let bottom_tick_gutter = if has_ticks(Axis::Horizontal) { 10.0 } else { 0.0 };

        Rect::new(
            Position::new(self.padding.left + left_tick_gutter, self.padding.top),
            Dimension::new(
                self.dimension.width - self.padding.left - self.padding.right - left_tick_gutter,
                self.dimension.height - self.padding.top - self.padding.bottom - bottom_tick_gutter,
            )
        )
    }
}

impl<C: ControllerSequence> CommonWidget for Chart<C> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

impl<C: ControllerSequence> Update for Chart<C> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.fit_scales(ctx.env);
    }
}

impl<C: ControllerSequence> Render for Chart<C> {
    fn render(&mut self, context: &mut RenderContext) {
        let chart_area = self.chart_area();
        let mut canvas = CanvasContext::new(self.position, self.dimension, context);

        for (_, scale) in self.scales.iter() {
            scale.draw_grid(&mut canvas, chart_area);
        }

        for (_, scale) in self.scales.iter() {
            scale.draw_border(&mut canvas, chart_area);
        }

        let scales = &self.scales;
        self.controllers.controllers(&mut |controller| {
            controller.draw(&mut canvas, scales, chart_area);
        });
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::Scalar;
    use carbide::environment::Environment;
    use carbide::state::{LocalState, State};
    use crate::controller::{ControllerSequence, ScatterController};
    use crate::dataset::DatasetExt;
    use crate::scale::ScaleId;
    use super::Chart;

    /// Datasets spread over several controllers share the chart's scales: they fit exactly as
    /// they would with every dataset in one controller.
    #[test]
    fn scales_fit_the_bounds_of_every_controller() {
        let first = vec![75.0, 70.0, 80.0, -60.0];
        let second = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        let split = fitted(Chart::new((
            ScatterController::new(first.clone()),
            ScatterController::new(second.clone()),
        )));
        let combined = fitted(Chart::new(ScatterController::new((first, second))));

        assert_eq!(scale_range(&split, ScaleId::X), scale_range(&combined, ScaleId::X));
        assert_eq!(scale_range(&split, ScaleId::Y), scale_range(&combined, ScaleId::Y));
        assert_eq!(split.scales.get(ScaleId::Y).unwrap().ticks(), combined.scales.get(ScaleId::Y).unwrap().ticks());

        // Sanity check that the shared range really spans both controllers' data: the y range
        // must cover the first controller's -60 and the second controller's 92.
        let (y_min, y_max) = scale_range(&split, ScaleId::Y);
        assert!(y_min <= -60.0 && y_max >= 92.0, "y range {y_min}..{y_max} should span both controllers' data");
    }

    /// An empty dataset contributes nothing to the bounds, and a chart with no data at all
    /// doesn't panic: its scales fall back to `0..1`.
    #[test]
    fn empty_datasets_do_not_change_the_scales() {
        let empty: Vec<Scalar> = vec![];
        let data = vec![100.0, 150.0, 200.0];

        let with_empty = fitted(Chart::new(ScatterController::new((empty.clone(), data.clone()))));
        let without = fitted(Chart::new(ScatterController::new(data)));

        assert_eq!(scale_range(&with_empty, ScaleId::X), scale_range(&without, ScaleId::X));
        assert_eq!(scale_range(&with_empty, ScaleId::Y), scale_range(&without, ScaleId::Y));

        let only_empty = fitted(Chart::new(ScatterController::new(empty)));
        assert_eq!(scale_range(&only_empty, ScaleId::Y), (0.0, 1.0));
    }

    /// A dataset's scale binding is a state: changing it moves the dataset's bounds to the newly
    /// bound scale on the next fit.
    #[test]
    fn scale_binding_is_reactive() {
        let binding = LocalState::new(ScaleId::new("elsewhere"));
        let small = vec![1.0, 2.0];
        let large = vec![100.0, 200.0].y_scale(binding.clone());

        let mut chart = Chart::new(ScatterController::new((small, large)));
        let mut env = Environment::new();

        // Bound to a scale the chart doesn't have: the large values don't reach Y.
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 100.0, "Y should only cover the small dataset, but reaches {y_max}");

        // Rebound to Y: the next fit includes the large values.
        binding.clone().set_value(ScaleId::Y);
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 200.0, "Y should cover the large dataset after rebinding, but ends at {y_max}");
    }

    fn fitted<C: ControllerSequence>(mut chart: Chart<C>) -> Chart<C> {
        chart.fit_scales(&mut Environment::new());
        chart
    }

    fn scale_range<C: ControllerSequence>(chart: &Chart<C>, id: ScaleId) -> (Scalar, Scalar) {
        let range = chart.scales.get(id).expect("the chart should have the scale").range();
        (range.start, range.end)
    }
}
