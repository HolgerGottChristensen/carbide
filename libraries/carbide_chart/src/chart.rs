use std::collections::HashSet;
use carbide::environment::Environment;
use carbide::lifecycle::{Update, UpdateContext};
use carbide::widget::EdgeInsets;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position, Rect, Scalar};
use indexmap::IndexMap;
use carbide_core::render::{Render, RenderContext};
use carbide_core::widget::{CommonWidget, WidgetId};
use carbide_core::widget::canvas::CanvasContext;
use carbide_derive::Widget;
use crate::controller::ControllerSequence;
use crate::scale::{AxisPosition, Scale, ScaleBounds, ScaleId, Scales};

/// The gap between the bands of two scales stacked on the same side of the chart area, like
/// ag-charts' axis `spacing`.
const SCALE_SPACING: Scalar = 8.0;

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

    /// Adds `scale` to the chart under `id`, for datasets bound to it with `.x_scale(id)` or
    /// `.y_scale(id)`. Passing `ScaleId::X` or `ScaleId::Y` replaces that default scale.
    ///
    /// Scales on the same side of the chart area are stacked outwards, each in its own band:
    /// by `Scale::weight`, then in the order they were added, the first closest to the area.
    pub fn scale(mut self, id: ScaleId, scale: impl Scale) -> Chart<C> {
        self.scales.insert(id, scale);
        self
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

    /// Lays the scales out around the chart area the way Chart.js lays out its boxes
    /// (`core.layouts.js`): on each side, the scales are sorted by weight, ties kept in the order
    /// they were added, and stacked outwards from the chart area, each taking a band as wide as
    /// its measured `sizes` entry (see `Scale::size`), with `SCALE_SPACING` between neighbouring
    /// bands. The chart area is what the padding and every side's bands leave over.
    fn layout(&self, sizes: &IndexMap<ScaleId, Scalar>) -> ScaleLayout {
        let mut offsets = IndexMap::new();

        let mut stack_side = |side: AxisPosition| -> Scalar {
            let mut on_side: Vec<(i32, usize, ScaleId)> = self.scales.iter()
                .enumerate()
                .filter(|(_, (_, scale))| scale.axis_position() == side)
                .map(|(index, (id, scale))| (scale.weight(), index, id))
                .collect();

            on_side.sort_by_key(|(weight, index, _)| (*weight, *index));

            let mut depth = 0.0;
            for (_, _, id) in on_side {
                let size = sizes.get(&id).copied().unwrap_or(0.0);

                // Spacing goes only between two bands: never against the chart area, and not
                // next to a scale that takes no space.
                if depth > 0.0 && size > 0.0 {
                    depth += SCALE_SPACING;
                }

                offsets.insert(id, depth);
                depth += size;
            }

            depth
        };

        let left = stack_side(AxisPosition::Left);
        let right = stack_side(AxisPosition::Right);
        let top = stack_side(AxisPosition::Top);
        let bottom = stack_side(AxisPosition::Bottom);

        let chart_area = Rect::new(
            Position::new(self.padding.left + left, self.padding.top + top),
            Dimension::new(
                self.dimension.width - self.padding.left - self.padding.right - left - right,
                self.dimension.height - self.padding.top - self.padding.bottom - top - bottom,
            )
        );

        ScaleLayout { chart_area, offsets }
    }
}

/// Where everything goes for one render: the area the datasets are drawn in, and for each scale
/// the distance from that area to the inner edge of its band.
#[derive(Debug)]
struct ScaleLayout {
    chart_area: Rect,
    offsets: IndexMap<ScaleId, Scalar>,
}

impl ScaleLayout {
    fn offset(&self, id: ScaleId) -> Scalar {
        self.offsets.get(&id).copied().unwrap_or(0.0)
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
        let mut canvas = CanvasContext::new(self.position, self.dimension, context);

        let sizes: IndexMap<ScaleId, Scalar> = self.scales.iter()
            .map(|(id, scale)| (id, scale.size(&mut canvas)))
            .collect();

        let layout = self.layout(&sizes);
        let chart_area = layout.chart_area;

        for (id, scale) in self.scales.iter() {
            scale.draw_grid(&mut canvas, chart_area, layout.offset(id));
        }

        for (id, scale) in self.scales.iter() {
            scale.draw_border(&mut canvas, chart_area, layout.offset(id));
        }

        let scales = &self.scales;
        self.controllers.controllers(&mut |controller| {
            controller.draw(&mut canvas, scales, chart_area);
        });
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Dimension, Scalar};
    use carbide::environment::Environment;
    use carbide::state::{LocalState, State};
    use indexmap::IndexMap;
    use crate::controller::{BarController, ControllerSequence, ScatterController};
    use crate::dataset::DatasetExt;
    use crate::scale::{Axis, AxisPosition, LinearScale, ScaleId};
    use super::{Chart, SCALE_SPACING};

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

    /// A dataset bound to a custom scale fits that scale, not `ScaleId::Y`, and a scale on the
    /// right gets a band on the right of the chart area.
    #[test]
    fn custom_scales_get_their_own_bounds_and_side() {
        const PERCENT: ScaleId = ScaleId::new("percent");

        let revenue = vec![120.0, 150.0, 190.0];
        let margin = vec![0.12, 0.25, 0.31].y_scale(PERCENT);

        let mut chart = fitted(
            Chart::new(ScatterController::new((revenue, margin)))
                .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
        );

        let (percent_min, percent_max) = scale_range(&chart, PERCENT);
        assert!(percent_min <= 0.12 && percent_max >= 0.31 && percent_max < 1.0, "percent should fit only the margin, got {percent_min}..{percent_max}");

        let (y_min, _) = scale_range(&chart, ScaleId::Y);
        assert!(y_min >= 100.0, "Y should fit only the revenue, but starts at {y_min}");

        chart.dimension = Dimension::new(400.0, 300.0);
        let area = chart.layout(&sizes(&[(ScaleId::Y, 35.0), (PERCENT, 45.0)])).chart_area;
        assert_eq!(area.left(), 30.0 + 35.0, "left: padding + Y's band");
        assert_eq!(area.right(), 400.0 - 30.0 - 45.0, "right: padding + percent's band");
    }

    /// Without a scale on the right there is no band there.
    #[test]
    fn default_scales_have_no_right_band() {
        let mut chart = fitted(Chart::new(ScatterController::new(vec![1.0, 2.0])));

        chart.dimension = Dimension::new(400.0, 300.0);
        let layout = chart.layout(&sizes(&[(ScaleId::X, 20.0), (ScaleId::Y, 35.0)]));
        assert_eq!(layout.chart_area.right(), 400.0 - 30.0);
        assert_eq!(layout.chart_area.left(), 30.0 + 35.0);
        assert_eq!(layout.chart_area.height(), 300.0 - 30.0 - 30.0 - 20.0, "X's band comes off the height");
    }

    /// Scales on the same side stack outwards in the order they were added, each starting
    /// `SCALE_SPACING` past where the band inside it ends; the chart area gives up the bands and
    /// the spacing between them.
    #[test]
    fn scales_on_one_side_stack_outwards_in_order() {
        const INNER: ScaleId = ScaleId::new("inner");
        const OUTER: ScaleId = ScaleId::new("outer");

        let mut chart = Chart::new(ScatterController::new(vec![1.0, 2.0]))
            .scale(INNER, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
            .scale(OUTER, LinearScale::new(Axis::Vertical).position(AxisPosition::Right));

        chart.dimension = Dimension::new(400.0, 300.0);
        let layout = chart.layout(&sizes(&[(INNER, 40.0), (OUTER, 25.0)]));

        assert_eq!(layout.offset(INNER), 0.0);
        assert_eq!(layout.offset(OUTER), 40.0 + SCALE_SPACING);
        assert_eq!(layout.chart_area.right(), 400.0 - 30.0 - 40.0 - SCALE_SPACING - 25.0);
    }

    /// No spacing is added next to a scale that takes no space, so hiding a scale's band doesn't
    /// leave a gap behind.
    #[test]
    fn empty_bands_get_no_spacing() {
        const EMPTY: ScaleId = ScaleId::new("empty");
        const OUTER: ScaleId = ScaleId::new("outer");

        let mut chart = Chart::new(ScatterController::new(vec![1.0, 2.0]))
            .scale(EMPTY, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
            .scale(OUTER, LinearScale::new(Axis::Vertical).position(AxisPosition::Right));

        chart.dimension = Dimension::new(400.0, 300.0);
        let layout = chart.layout(&sizes(&[(EMPTY, 0.0), (OUTER, 25.0)]));

        assert_eq!(layout.offset(OUTER), 0.0);
        assert_eq!(layout.chart_area.right(), 400.0 - 30.0 - 25.0);
    }

    /// A higher weight moves a scale further out, whatever order it was added in; Y keeps its
    /// default weight 0 and stays innermost on the left.
    #[test]
    fn higher_weights_are_further_from_the_chart_area() {
        const HEAVY: ScaleId = ScaleId::new("heavy");
        const LIGHT: ScaleId = ScaleId::new("light");

        let mut chart = Chart::new(ScatterController::new(vec![1.0, 2.0]))
            .scale(HEAVY, LinearScale::new(Axis::Vertical).weight(2))
            .scale(LIGHT, LinearScale::new(Axis::Vertical).weight(1));

        chart.dimension = Dimension::new(400.0, 300.0);
        let layout = chart.layout(&sizes(&[(ScaleId::Y, 30.0), (HEAVY, 20.0), (LIGHT, 10.0)]));

        assert_eq!(layout.offset(ScaleId::Y), 0.0);
        assert_eq!(layout.offset(LIGHT), 30.0 + SCALE_SPACING);
        assert_eq!(layout.offset(HEAVY), 30.0 + SCALE_SPACING + 10.0 + SCALE_SPACING);
        assert_eq!(layout.chart_area.left(), 30.0 + 30.0 + 10.0 + 20.0 + 2.0 * SCALE_SPACING);
    }

    /// A bar controller makes its y scale include zero, so bars grow from it, and offsets its x
    /// scale so the first and last bars sit inside the chart area. A scatter controller on its own
    /// does neither.
    #[test]
    fn bars_include_zero_and_offset_their_x_scale() {
        let revenue = vec![120.0, 150.0, 190.0];

        let bars = fitted(Chart::new(BarController::new(revenue.clone())));
        let (y_min, _) = scale_range(&bars, ScaleId::Y);
        assert!(y_min <= 0.0, "bars should start the y scale at zero, but it starts at {y_min}");

        let x = bars.scales.get(ScaleId::X).unwrap();
        assert!(x.normalize(0.0) > 0.0 && x.normalize(2.0) < 1.0, "the first and last bar should sit inside the chart area");

        let points = fitted(Chart::new(ScatterController::new(revenue)));
        let (y_min, _) = scale_range(&points, ScaleId::Y);
        assert!(y_min > 0.0, "without bars the y scale needn't include zero, but starts at {y_min}");
        assert_eq!(points.scales.get(ScaleId::X).unwrap().normalize(0.0), 0.0);
    }

    fn sizes(entries: &[(ScaleId, Scalar)]) -> IndexMap<ScaleId, Scalar> {
        entries.iter().copied().collect()
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
