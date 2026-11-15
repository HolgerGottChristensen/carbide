use std::collections::HashMap;
use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use indexmap::{IndexMap, IndexSet};
use crate::controller::DatasetController;
use crate::controller::drawing::{dataset_color, dataset_scales};
use crate::{DataColor, DataSet, DataSetOptions, DataSetSequence, StackId};
use crate::sequence::IntoSequence;
use crate::scale::{Scale, ScaleBounds, ScaleId, ScaleKind, Scales};

/// Draws each dataset as vertical bars, one per point, growing from zero to the point's y:
///
/// - The x scale gets an offset (half a tick step at each end), so the first and last bars aren't
///   cut in half at the chart area's edges.
/// - Zero is always included in the y bounds, so every bar has a base to grow from.
/// - The datasets in this controller that share an x scale are grouped side by side. Each point
///   gets a slot `category_percentage` of the smallest gap between neighbouring x values or ticks;
///   the slot is split evenly between the bars beside each other, and each bar is
///   `bar_percentage` of its share.
/// - Datasets with the same stack (`DatasetExt::stack`) and the same x and y scales are stacked
///   into one bar, in dataset order, the first at the bottom. Positive values stack upwards from
///   zero and negative values downwards, each on their own. A stack takes one share of the slot,
///   like a dataset without a stack, and the y bounds cover the stacked sums.
///
/// Bars in two different bar controllers on the same x scale are neither grouped nor stacked with
/// each other.
#[derive(Clone, Debug)]
pub struct BarController<D> {
    dataset_sequence: D,
    category_percentage: Scalar,
    bar_percentage: Scalar,
}

impl<D: DataSetSequence<Scalar, Scalar, Scalar>> BarController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn DataSet<Scalar, Scalar, Scalar>, M, Sequence=D>, M>(datasets: I) -> BarController<D> {
        BarController {
            dataset_sequence: datasets.into_sequence(),
            category_percentage: 0.8,
            bar_percentage: 0.9,
        }
    }

    /// How much of the smallest gap between neighbouring x values a point's group of bars takes
    /// up. Defaults to 0.8.
    pub fn category_percentage(mut self, category_percentage: Scalar) -> BarController<D> {
        self.category_percentage = category_percentage;
        self
    }

    /// How much of its share of the group's slot each bar takes up. Defaults to 0.9.
    pub fn bar_percentage(mut self, bar_percentage: Scalar) -> BarController<D> {
        self.bar_percentage = bar_percentage;
        self
    }
}

impl<D: DataSetSequence<Scalar, Scalar, Scalar>> DatasetController for BarController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    /// Bars sit on a category scale along x, offset so the first and last bars aren't cut in
    /// half, with grid lines between the bar groups. The offset and grid offset are asked of any
    /// x scale, also one set with `Chart::scale`; the kind only decides a default scale. They are
    /// asked for even while a dataset is empty, so the scale doesn't change when data arrives.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        let mut stacks = StackSums::default();

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();

            bounds.request_kind(&options.x_scale, ScaleKind::Category);
            bounds.request_offset(&options.x_scale);
            bounds.request_grid_offset(&options.x_scale);

            // Each bar spans from where its stack has reached to where it takes it, and every bar
            // includes zero, its base.
            let key = StackKey::new(&options, dataset_index);
            dataset.points(&mut |_index, point| {
                let (start, end) = stacks.add(&key, point.x(), point.y());

                bounds.include(&options.x_scale, point.x(), point.x());
                bounds.include(&options.y_scale, start.min(end).min(0.0), start.max(end).max(0.0));
            });
        });
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        // First pass: which bars sit side by side (per x scale, a share for each stack and each
        // dataset without one, in sequence order), and every bar's x position, for the smallest
        // gap between neighbours.
        let mut groups: IndexMap<ScaleId, IndexSet<StackKey>> = IndexMap::new();
        let mut positions: IndexMap<ScaleId, Vec<Scalar>> = IndexMap::new();
        let mut places: Vec<Option<(ScaleId, usize)>> = vec![];

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();

            let Some((x_scale, _)) = dataset_scales(scales, &options) else {
                places.push(None);
                return;
            };

            let (group_index, _) = groups.entry(options.x_scale.clone()).or_default().insert_full(StackKey::new(&options, dataset_index));
            places.push(Some((options.x_scale.clone(), group_index)));

            let x_positions = positions.entry(options.x_scale).or_default();
            dataset.points(&mut |_index, point| {
                x_positions.push(chart_area.left() + x_scale.normalize(point.x()) * chart_area.width());
            });
        });

        let rulers: IndexMap<ScaleId, Scalar> = positions.into_iter()
            .filter_map(|(id, x_positions)| {
                let scale = scales.get(&id)?;
                Some((id, min_sample_size(x_positions, scale, chart_area)))
            })
            .collect();

        // Second pass: the bars, stacked in the same order as their bounds were collected.
        let mut stacks = StackSums::default();
        ctx.save();

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let Some(&Some((ref x_id, group_index))) = places.get(dataset_index) else {
                return;
            };

            let options = dataset.options();

            let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
                return;
            };

            let group_size = groups[x_id].len();
            let key = StackKey::new(&options, dataset_index);
            let ruler = rulers.get(x_id).copied().unwrap_or(chart_area.width());

            let y_pixel = |value: Scalar| chart_area.bottom() + (1.0 - y_scale.normalize(value)) * chart_area.height();
            let zero = y_pixel(base_value(y_scale));

            dataset.points(&mut |_index, point| {
                let center = chart_area.left() + x_scale.normalize(point.x()) * chart_area.width();
                let (left, width) = bar_slot(center, ruler, group_index, group_size, self.category_percentage, self.bar_percentage);

                // The first bar of a stack grows from zero, or the end of the range nearest it.
                let (start, end) = stacks.add(&key, point.x(), point.y());
                let base = if start == 0.0 { zero } else { y_pixel(start) };
                let head = y_pixel(end);

                ctx.begin_path();
                ctx.move_to(left, base);
                ctx.line_to(left + width, base);
                ctx.line_to(left + width, head);
                ctx.line_to(left, head);
                ctx.close_path();

                match point.color() {
                    DataColor::Inherit => ctx.set_fill_style(dataset_color(&options, dataset_index)),
                    DataColor::Color(color) => ctx.set_fill_style(color),
                }

                ctx.fill();
            });
        });

        ctx.restore();
    }
}

/// Which bars share one place in a slot: the datasets with the same stack on the same x and y
/// scales, stacked into one bar, or a single dataset without a stack.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum StackKey {
    Stacked { x_scale: ScaleId, y_scale: ScaleId, stack: StackId },
    Alone(usize),
}

impl StackKey {
    fn new(options: &DataSetOptions, dataset_index: usize) -> StackKey {
        match &options.stack {
            Some(stack) => StackKey::Stacked {
                x_scale: options.x_scale.clone(),
                y_scale: options.y_scale.clone(),
                stack: stack.clone(),
            },
            None => StackKey::Alone(dataset_index),
        }
    }
}

/// How far each stack has reached at each x value, with positive and negative values summed
/// apart, so each grows away from zero on its own side.
#[derive(Default)]
struct StackSums {
    sums: HashMap<(StackKey, u64, bool), Scalar>,
}

impl StackSums {
    /// Adds a bar of `y` to the stack `key` at `x`, and returns where it starts and ends: from the
    /// stack's sum on `y`'s side so far, to that sum plus `y`. A bar on its own starts at zero.
    fn add(&mut self, key: &StackKey, x: Scalar, y: Scalar) -> (Scalar, Scalar) {
        // Adding 0.0 turns -0.0 into 0.0, so both find the same stack.
        let sum = self.sums.entry((key.clone(), (x + 0.0).to_bits(), y >= 0.0)).or_insert(0.0);
        let start = *sum;
        *sum += y;
        (start, *sum)
    }
}

/// The value bars grow from: zero when it is inside the y scale's range, otherwise the end of the
/// range nearest to zero.
fn base_value(y_scale: &dyn Scale) -> Scalar {
    0.0_f64.clamp(y_scale.min(), y_scale.max())
}

/// The smallest gap, in pixels, between neighbouring bar positions or neighbouring ticks of the x
/// scale, capped at the chart area's width. It sets how wide a point's group of bars can be without
/// overlapping the next point's.
fn min_sample_size(mut x_positions: Vec<Scalar>, x_scale: &dyn Scale, chart_area: Rect) -> Scalar {
    let mut min = chart_area.width();

    let mut tick_positions: Vec<Scalar> = x_scale.ticks().iter()
        .map(|tick| chart_area.left() + x_scale.normalize(*tick) * chart_area.width())
        .collect();

    for positions in [&mut x_positions, &mut tick_positions] {
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

/// The left edge and width of one bar: the group's slot is `category_percentage` of `ruler`,
/// centred on the point's x; it is split into `group_size` equal shares, and the bar is
/// `bar_percentage` of share number `group_index`, centred in it.
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
    use crate::scale::{ScaleBounds, ScaleId};
    use super::{bar_slot, BarController};

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
