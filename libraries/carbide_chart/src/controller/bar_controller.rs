use carbide::draw::{Position, Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use indexmap::{IndexMap, IndexSet};
use crate::controller::DatasetController;
use crate::controller::drawing::{dataset_scales, legend_items, point_color, Palette};
use crate::legend::{LegendItem, LegendMarker};
use crate::controller::stacking::{StackKey, StackSums};
use crate::{DataSet, DataSetSequence};
use crate::sequence::IntoSequence;
use crate::controller::line_path::PathSink;
use crate::controller::orientation::{length, pixel, Oriented, Orientation};
use crate::scale::{Axis, Scale, ScaleBounds, ScaleId, ScaleKind, Scales};

/// Draws each dataset as bars, one per point. A point is read as its category (x) and its value
/// (y), and the bar grows from zero to the value. Bars stand up by default: the categories run
/// along the dataset's x scale and the values up its y scale. `horizontal` lays them down, with
/// the same data: the categories run down the dataset's y scale and the values along its x scale.
///
/// The index scale is the one the categories run along, and the value scale the one the bars grow
/// along. A missing value (see `DataPoint::is_missing`) has no bar, and leaves its place in the
/// slot empty:
///
/// - The index scale gets an offset (half a tick step at each end), so the first and last bars
///   aren't cut in half at the chart area's edges.
/// - Zero is always included in the value scale's bounds, so every bar has a base to grow from.
/// - The datasets in this controller that share an index scale are grouped side by side. Each
///   point gets a slot `category_percentage` of the smallest gap between neighbouring categories
///   or ticks; the slot is split evenly between the bars beside each other, and each bar is
///   `bar_percentage` of its share.
/// - Datasets with the same stack (`DatasetExt::stack`) and the same x and y scales are stacked
///   into one bar, in dataset order, the first nearest zero. Positive values stack away from zero
///   one way and negative values the other, each on their own. A stack takes one share of the
///   slot, like a dataset without a stack, and the value scale's bounds cover the stacked sums.
///
/// Bars in two different bar controllers on the same index scale are neither grouped nor stacked
/// with each other.
#[derive(Clone, Debug)]
pub struct BarController<D> {
    dataset_sequence: D,
    category_percentage: Scalar,
    bar_percentage: Scalar,
    palette_offset: usize,
    orientation: Orientation,
}

impl<D: DataSetSequence> BarController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn DataSet, M, Sequence=D>, M>(datasets: I) -> BarController<D> {
        BarController {
            dataset_sequence: datasets.into_sequence(),
            category_percentage: 0.8,
            bar_percentage: 0.9,
            palette_offset: 0,
            orientation: Orientation::Vertical,
        }
    }

    /// Lays the bars down: each point's category runs down the dataset's y scale, the first at
    /// the top on a category scale, and its bar grows along the x scale, from zero to the value.
    /// The data is read as for standing bars, so the same datasets draw either way.
    pub fn horizontal(mut self) -> BarController<D> {
        self.orientation = Orientation::Horizontal;
        self
    }

    /// How much of the smallest gap between neighbouring categories a point's group of bars takes
    /// up. Defaults to 0.8.
    pub fn category_percentage(mut self, category_percentage: Scalar) -> BarController<D> {
        self.category_percentage = category_percentage;
        self
    }

    /// Starts the colours of this controller's datasets `offset` colours into the palette:
    /// dataset `i` takes palette colour `offset + i`, and so do the points of a dataset coloured
    /// by point. Defaults to 0. A chart with several controllers sets it so their datasets don't
    /// share colours, as each controller counts its datasets from 0. Datasets that pick a palette
    /// colour themselves (`DataColor::Palette`) aren't shifted.
    pub fn palette_offset(mut self, offset: usize) -> BarController<D> {
        self.palette_offset = offset;
        self
    }

    /// How much of its share of the group's slot each bar takes up. Defaults to 0.9.
    pub fn bar_percentage(mut self, bar_percentage: Scalar) -> BarController<D> {
        self.bar_percentage = bar_percentage;
        self
    }
}

impl<D: DataSetSequence> DatasetController for BarController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    /// Bars sit on a category index scale, offset so the first and last bars aren't cut in half,
    /// with grid lines between the bar groups. The offset and grid offset are asked of any index
    /// scale, also one set with `Chart::scale`; the kind only decides a default scale. They are
    /// asked for even while a dataset is empty, so the scale doesn't change when data arrives.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        let mut stacks = StackSums::split_by_sign();

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let (index_scale, value_scale) = self.orientation.index_and_value_scales(&options);

            bounds.request_kind(index_scale, ScaleKind::Category);
            bounds.request_offset(index_scale);
            bounds.request_grid_offset(index_scale);

            // Each bar spans from where its stack has reached to where it takes it, and every bar
            // includes zero, its base. A missing value has no bar and adds nothing to its stack.
            let key = StackKey::new(&options, dataset_index);
            dataset.points(&mut |_index, point| {
                let (Some(x), Some(y)) = (point.x(), point.y()) else {
                    return;
                };

                let (start, end) = stacks.add(&key, x, y);

                bounds.include(index_scale, x, x);
                bounds.include(value_scale, start.min(end).min(0.0), start.max(end).max(0.0));
            });
        });
    }

    /// Each named dataset as a square in its colour.
    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        legend_items(&self.dataset_sequence, &Palette::from_env(env, self.palette_offset), items, |_| LegendMarker::Square);
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        let index_axis = self.orientation.index_axis();
        let value_axis = self.orientation.value_axis();

        // First pass: which bars sit side by side (per index scale, a share for each stack and
        // each dataset without one, in sequence order), and every bar's position along the index
        // axis, for the smallest gap between neighbours.
        let mut groups: IndexMap<ScaleId, IndexSet<StackKey>> = IndexMap::new();
        let mut positions: IndexMap<ScaleId, Vec<Scalar>> = IndexMap::new();
        let mut places: Vec<Option<(ScaleId, usize)>> = vec![];

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let (index_id, _) = self.orientation.index_and_value_scales(&options);

            let (Some(index_scale), Some(_)) = (scales.get(index_id), dataset_scales(scales, &options)) else {
                places.push(None);
                return;
            };

            let (group_index, _) = groups.entry(index_id.clone()).or_default().insert_full(StackKey::new(&options, dataset_index));
            places.push(Some((index_id.clone(), group_index)));

            let index_positions = positions.entry(index_id.clone()).or_default();
            dataset.points(&mut |_index, point| {
                let (Some(x), Some(_)) = (point.x(), point.y()) else {
                    return;
                };

                index_positions.push(pixel(index_axis, index_scale, x, chart_area));
            });
        });

        let rulers: IndexMap<ScaleId, Scalar> = positions.into_iter()
            .filter_map(|(id, index_positions)| {
                let scale = scales.get(&id)?;
                Some((id, min_sample_size(index_positions, scale, index_axis, chart_area)))
            })
            .collect();

        // Second pass: the bars, stacked in the same order as their bounds were collected, in the
        // colours of the chart's `SeriesStyle`.
        let mut stacks = StackSums::split_by_sign();
        let palette = Palette::new(ctx, self.palette_offset);
        ctx.save();

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let Some(&Some((ref index_id, group_index))) = places.get(dataset_index) else {
                return;
            };

            let options = dataset.options();
            let (_, value_id) = self.orientation.index_and_value_scales(&options);

            let (Some(index_scale), Some(value_scale)) = (scales.get(index_id), scales.get(value_id)) else {
                return;
            };

            let group_size = groups[index_id].len();
            let key = StackKey::new(&options, dataset_index);
            let ruler = rulers.get(index_id).copied().unwrap_or(length(index_axis, chart_area));
            let zero = pixel(value_axis, value_scale, base_value(value_scale), chart_area);

            dataset.points(&mut |point_index, point| {
                // A missing value leaves its place in the slot empty.
                let (Some(x), Some(y)) = (point.x(), point.y()) else {
                    return;
                };

                let center = pixel(index_axis, index_scale, x, chart_area);
                let (side, thickness) = bar_slot(center, ruler, group_index, group_size, self.category_percentage, self.bar_percentage);

                // The first bar of a stack grows from zero, or the end of the range nearest it.
                let (start, end) = stacks.add(&key, x, y);
                let base = if start == 0.0 { zero } else { pixel(value_axis, value_scale, start, chart_area) };
                let head = pixel(value_axis, value_scale, end, chart_area);

                // The corners in the index frame, along the index axis and along the value axis.
                ctx.begin_path();
                let mut path = Oriented { path: &mut *ctx, orientation: self.orientation };
                path.move_to(Position::new(side, base));
                path.line_to(Position::new(side + thickness, base));
                path.line_to(Position::new(side + thickness, head));
                path.line_to(Position::new(side, head));
                ctx.close_path();

                ctx.set_fill_style(point_color(point.color(), &options, dataset_index, point_index, &palette));

                ctx.fill();
            });
        });

        ctx.restore();
    }
}

/// The value bars grow from: zero when it is inside the value scale's range, otherwise the end of
/// the range nearest to zero.
fn base_value(value_scale: &dyn Scale) -> Scalar {
    0.0_f64.clamp(value_scale.min(), value_scale.max())
}

/// The smallest gap, in pixels, between neighbouring bar positions or neighbouring ticks of the
/// index scale, along `index_axis`, capped at the chart area's length along it. It sets how thick a
/// point's group of bars can be without overlapping the next point's.
fn min_sample_size(mut index_positions: Vec<Scalar>, index_scale: &dyn Scale, index_axis: Axis, chart_area: Rect) -> Scalar {
    let mut min = length(index_axis, chart_area);

    let mut tick_positions: Vec<Scalar> = index_scale.ticks().iter()
        .map(|tick| pixel(index_axis, index_scale, *tick, chart_area))
        .collect();

    for positions in [&mut index_positions, &mut tick_positions] {
        positions.sort_by(Scalar::total_cmp);

        for pair in positions.windows(2) {
            let gap = (pair[1] - pair[0]).abs();

            // Two bars at the same x don't make a gap.
            if gap > 0.0 {
                min = min.min(gap);
            }
        }
    }

    min
}

/// Where one bar starts along the index axis, and how thick it is: the group's slot is
/// `category_percentage` of `ruler`, centred on the point's category; it is split into
/// `group_size` equal shares, and the bar is `bar_percentage` of share number `group_index`,
/// centred in it. Share 0 is at the smaller pixel: the left, or the top for lying bars.
fn bar_slot(center: Scalar, ruler: Scalar, group_index: usize, group_size: usize, category_percentage: Scalar, bar_percentage: Scalar) -> (Scalar, Scalar) {
    let slot = ruler * category_percentage;
    let share = slot / group_size.max(1) as Scalar;
    let share_center = center - slot / 2.0 + share * group_index as Scalar + share / 2.0;
    let width = share * bar_percentage;

    (share_center - width / 2.0, width)
}

#[cfg(test)]
mod tests {
    use carbide::environment::Environment;
    use crate::{DatasetExt, StackId};
    use crate::controller::DatasetController;
    use crate::scale::{ScaleBounds, ScaleId, ScaleKind};
    use super::{bar_slot, BarController};

    /// A missing value has no bar: it adds nothing to the bounds, or to its stack.
    #[test]
    fn missing_values_have_no_bar() {
        const COSTS: StackId = StackId::new("costs");
        let values: Vec<Option<f64>> = vec![Some(4.0), None, Some(-2.0)];

        let mut bounds = ScaleBounds::default();
        BarController::new((values.clone().stack(COSTS), vec![1.0, 30.0, 1.0].stack(COSTS))).collect_bounds(&mut bounds, &mut Environment::new());

        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((-2.0, 30.0)), "at x = 1 the stack is just the 30");
        assert_eq!(bounds.get(&ScaleId::X).range(), Some((0.0, 2.0)));
    }

    /// The same stack id on two y scales makes two stacks: a stack is the datasets with the same
    /// id on the same x and y scales, so each y scale only reaches its own dataset's values. The
    /// same holds for stacked lines.
    #[test]
    fn the_same_stack_on_two_y_scales_makes_two_stacks() {
        const COSTS: StackId = StackId::new("costs");
        const RIGHT: ScaleId = ScaleId::new("right");

        let bars = BarController::new((vec![10.0, 20.0].stack(COSTS), vec![5.0, 8.0].stack(COSTS).y_scale(RIGHT)));
        let mut bounds = ScaleBounds::default();
        bars.collect_bounds(&mut bounds, &mut Environment::new());

        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((0.0, 20.0)), "Y should only reach the first dataset");
        assert_eq!(bounds.get(&RIGHT).range(), Some((0.0, 8.0)), "the right scale should only reach the second dataset, not the sum");

        let lines = crate::LineController::new((vec![10.0, 20.0].stack(COSTS), vec![5.0, 8.0].stack(COSTS).y_scale(RIGHT)));
        let mut bounds = ScaleBounds::default();
        lines.collect_bounds(&mut bounds, &mut Environment::new());

        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((10.0, 20.0)));
        assert_eq!(bounds.get(&RIGHT).range(), Some((5.0, 8.0)), "lines on another y scale don't stack either");
    }

    /// Stacked datasets report the sums they reach, positive and negative values each on their own
    /// side of zero; a dataset without a stack reports its own values.
    #[test]
    fn stacked_bars_report_their_sums() {
        const COSTS: StackId = StackId::new("costs");

        let materials = vec![10.0, 20.0].stack(COSTS);
        let labour = vec![5.0, -30.0].stack(COSTS);
        let refunds = vec![-8.0, -4.0].stack(COSTS);
        let revenue = vec![12.0, 6.0];

        let mut bounds = ScaleBounds::default();
        BarController::new((materials, labour, refunds, revenue)).collect_bounds(&mut bounds, &mut Environment::new());

        // At x = 0 the stack reaches 10 + 5 = 15 upwards and -8 downwards; at x = 1, 20 upwards
        // and -30 - 4 = -34 downwards. The revenue bar on its own stays within that.
        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((-34.0, 20.0)));
        assert_eq!(bounds.get(&ScaleId::X).range(), Some((0.0, 1.0)));

        let mut unstacked = ScaleBounds::default();
        BarController::new((vec![10.0, 20.0], vec![5.0, 30.0])).collect_bounds(&mut unstacked, &mut Environment::new());
        assert_eq!(unstacked.get(&ScaleId::Y).range(), Some((0.0, 30.0)), "without a stack, bars don't add up");
    }

    /// Lying bars read the same data as standing ones, with the scales swapped: the categories
    /// run along the dataset's y scale, which is asked to be an offset category scale with grid
    /// lines between the groups, and the stacked values along its x scale.
    #[test]
    fn horizontal_bars_put_the_categories_on_the_y_scale() {
        const COSTS: StackId = StackId::new("costs");
        let bars = BarController::new((vec![10.0, -20.0].stack(COSTS), vec![5.0, -4.0].stack(COSTS))).horizontal();

        let mut bounds = ScaleBounds::default();
        bars.collect_bounds(&mut bounds, &mut Environment::new());

        let y = bounds.get(&ScaleId::Y);
        assert_eq!(y.range(), Some((0.0, 1.0)), "the categories are the points' x values");
        assert_eq!(y.kind(), Some(ScaleKind::Category));
        assert!(y.offset() && y.grid_offset());

        let x = bounds.get(&ScaleId::X);
        assert_eq!(x.range(), Some((-24.0, 15.0)), "the values, stacked, with zero");
        assert_eq!(x.kind(), None, "the value scale is left to its default");
        assert!(!x.offset() && !x.grid_offset());
    }

    /// One dataset: the bar is `bar_percentage` of `category_percentage` of the ruler, centred on
    /// the point. With a 50px ruler: slot 40px, bar 36px, from 82 to 118 around x = 100.
    #[test]
    fn a_single_bar_is_centred_on_its_point() {
        let (left, width) = bar_slot(100.0, 50.0, 0, 1, 0.8, 0.9);

        assert!((width - 36.0).abs() < 1e-9, "width {width}");
        assert!((left - 82.0).abs() < 1e-9, "left {left}");
    }

    /// Two datasets: the 40px slot is split into two 20px shares centred at 90 and 110, each
    /// holding an 18px bar, so the group stays centred on the point.
    #[test]
    fn grouped_bars_split_the_slot_side_by_side() {
        let (first_left, first_width) = bar_slot(100.0, 50.0, 0, 2, 0.8, 0.9);
        let (second_left, second_width) = bar_slot(100.0, 50.0, 1, 2, 0.8, 0.9);

        assert!((first_width - 18.0).abs() < 1e-9 && (second_width - 18.0).abs() < 1e-9);
        assert!((first_left - 81.0).abs() < 1e-9, "first left {first_left}");
        assert!((second_left - 101.0).abs() < 1e-9, "second left {second_left}");
    }
}
