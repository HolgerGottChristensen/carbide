use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use indexmap::{IndexMap, IndexSet};
use crate::controller::DatasetController;
use crate::controller::drawing::{polar_frame, register_values, dataset_scales, first_mark_entry, legend_items, point_color, Palette, PolarFrame};
use crate::legend::{LegendItem, LegendMarker};
use crate::controller::stacking::{normalized_totals, stack_offset, NormalizedTotals, StackKey, StackOffsets, StackSums};
use crate::DataSetOptions;
use crate::mark::{BarCanvas, BarDataSet, BarFrame, BarSlots, MarkRole};
use crate::sequence::{IntoSequence, Sequence};
use crate::controller::orientation::{is_polar, Orientation};
use crate::scale::{Coordinates, GridShape, Places, PolarRole, Scale, ScaleBounds, ScaleId, ScaleKind, Scales};

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

impl<D: Sequence<dyn BarDataSet>> BarController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn BarDataSet, M, Sequence=D>, M>(datasets: I) -> BarController<D> {
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

impl<D: Sequence<dyn BarDataSet>> DatasetController for BarController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    /// Bars sit on a category index scale, offset so the first and last bars aren't cut in half,
    /// with grid lines between the bar groups. The offset and grid offset are asked of any index
    /// scale, also one set with `Chart::scale`; the kind only decides a default scale. They are
    /// asked for even while a dataset is empty, so the scale doesn't change when data arrives.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        register_values(&self.dataset_sequence, bounds, self.orientation, true);

        let mut stacks = StackSums::split_by_sign().normalized(self.normalized_totals(&*bounds));
        let offsets = self.stack_offsets(&*bounds, self.has_overlays());

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();

            // Around a circle, wedges and radial bars ask for circle rings.
            if is_polar(&options, &*bounds) {
                bounds.request_grid_shape(&options.x_scale, GridShape::Circle);
                bounds.request_grid_shape(&options.y_scale, GridShape::Circle);
            }

            let (index_scale, value_scale) = self.orientation.for_dataset(&options, &*bounds).index_and_value_scales(&options);

            bounds.request_kind(index_scale, ScaleKind::Category);
            bounds.request_offset(index_scale);
            bounds.request_grid_offset(index_scale);

            // A dataset whose bars a mark replaces reports the mark's values instead of its bars.
            if !has_primary_mark(dataset) {
                // Each bar spans from where its stack has reached to where it takes it, and every
                // bar includes zero, its base. A missing value has no bar and adds nothing to its
                // stack.
                let key = StackKey::new(&options, dataset_index);
                let coordinates = Coordinates::new(&*bounds, index_scale, value_scale);
                dataset.points(&mut |_index, point| {
                    let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) else {
                        return;
                    };

                    let (start, end) = stacks.add(&key, x, y);

                    bounds.include(index_scale, x, x);
                    bounds.include(value_scale, start.min(end).min(0.0), start.max(end).max(0.0));
                });
            }

            // A mark's values move with the dataset's bars where they are stacked.
            let coordinates = Coordinates::new(&*bounds, index_scale, value_scale);
            dataset.bar_marks(&mut |mark| {
                mark.bounds(&coordinates, &mut |x, low, high| {
                    let offset = stack_offset(offsets.dataset(dataset_index), x);
                    bounds.include(index_scale, x, x);
                    bounds.include(value_scale, low + offset, high + offset);
                });
            });
        });
    }

    /// Each named dataset as a square in its colour, or as its mark asks, such as a square split
    /// into the rising and falling colours for candlesticks.
    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        legend_items(
            &self.dataset_sequence,
            env,
            &Palette::from_env(env, self.palette_offset),
            items,
            |_, _| LegendMarker::Square,
            |dataset, context| first_mark_entry(|f| dataset.bar_marks(&mut |mark| f(mark.legend(context)))),
        );
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        let wants_x_angle = self.orientation.polar_roles().0 == PolarRole::Angle;

        // First pass: how each dataset maps onto the screen (along the sides, or around the
        // circle), which bars sit side by side (per index scale, a share for each stack and each
        // dataset without one, in sequence order), and every bar's position along the index axis,
        // for the smallest gap between neighbours.
        let mut groups: IndexMap<ScaleId, IndexSet<StackKey>> = IndexMap::new();
        let mut positions: IndexMap<ScaleId, (Vec<Scalar>, Vec<Scalar>, Scalar)> = IndexMap::new();
        let mut places: Vec<Option<(ScaleId, usize)>> = vec![];
        let mut polar_frames: Vec<Option<PolarFrame>> = vec![];

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let orientation = self.orientation.for_dataset(&options, scales);
            let (index_id, value_id) = orientation.index_and_value_scales(&options);

            let (Some(index_scale), Some(_)) = (scales.get(index_id), dataset_scales(scales, &options)) else {
                places.push(None);
                polar_frames.push(None);
                return;
            };

            let Some(polar) = polar_frame(scales, index_id, value_id, wants_x_angle) else {
                places.push(None);
                polar_frames.push(None);
                return;
            };

            let frame = match &polar {
                Some(polar) => BarFrame::Around(polar),
                None => BarFrame::Sides { orientation, area: chart_area },
            };

            let (group_index, _) = groups.entry(index_id.clone()).or_default().insert_full(slot_key(dataset, &options, dataset_index));
            places.push(Some((index_id.clone(), group_index)));

            let (index_positions, _, _) = positions.entry(index_id.clone()).or_insert_with(|| {
                let ticks = index_scale.ticks().iter().map(|tick| frame.index(index_scale, *tick)).collect();
                (vec![], ticks, frame.index_length())
            });
            let coordinates = Coordinates::new(scales, index_id, value_id);
            dataset.points(&mut |_index, point| {
                let (Some(x), Some(_)) = (coordinates.x(point), coordinates.y(point)) else {
                    return;
                };

                index_positions.push(frame.index(index_scale, x));
            });

            polar_frames.push(polar);
        });

        let rulers: IndexMap<ScaleId, Scalar> = positions.into_iter()
            .map(|(id, (index_positions, tick_positions, length))| (id, min_sample_size(index_positions, tick_positions, length)))
            .collect();

        // Second pass: the bars, stacked in the same order as their bounds were collected, in the
        // colours of the chart's `SeriesStyle`, or the marks that replace them. Third pass: the
        // overlays, on top of every dataset's bars.
        let mut stacks = StackSums::split_by_sign().normalized(self.normalized_totals(scales));
        let palette = Palette::new(ctx, self.palette_offset);
        ctx.save();

        // Overlays are drawn in a pass of their own, and only read the stack offsets, so a chart
        // without them needs neither.
        let has_overlays = self.has_overlays();
        let offsets = self.stack_offsets(scales, has_overlays);
        let roles: &[MarkRole] = if has_overlays { &[MarkRole::Primary, MarkRole::Overlay] } else { &[MarkRole::Primary] };

        for &role in roles {
            self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
                let Some(&Some((ref index_id, group_index))) = places.get(dataset_index) else {
                    return;
                };

                let options = dataset.options();
                let orientation = self.orientation.for_dataset(&options, scales);
                let (_, value_id) = orientation.index_and_value_scales(&options);

                let (Some(index_scale), Some(value_scale)) = (scales.get(index_id), scales.get(value_id)) else {
                    return;
                };

                let frame = match &polar_frames[dataset_index] {
                    Some(polar) => BarFrame::Around(polar),
                    None => BarFrame::Sides { orientation, area: chart_area },
                };

                let group_size = groups[index_id].len();
                let ruler = rulers.get(index_id).copied().unwrap_or_else(|| frame.index_length());

                let slots = BarSlots {
                    frame,
                    index_scale,
                    ruler,
                    group_index,
                    group_size,
                    category_percentage: self.category_percentage,
                    bar_percentage: self.bar_percentage,
                    stack_offsets: offsets.dataset(dataset_index),
                };

                let coordinates = Coordinates::new(scales, index_id, value_id);
                let mut canvas = BarCanvas {
                    ctx: &mut *ctx,
                    coordinates,
                    frame,
                    value_scale,
                    options: &options,
                    dataset_index,
                    palette: &palette,
                };

                let around = matches!(frame, BarFrame::Around(_));
                dataset.bar_marks(&mut |mark| {
                    if mark.role() != role {
                        return;
                    }

                    // Marks that don't draw through the circle's frame are left out around it.
                    if around && !mark.draws_around_a_circle() {
                        scales.warn_once("polar bar marks", index_id, value_id, || format!("A bar mark that doesn't draw around a circle, such as a box plot or candlesticks, on the polar scales {:?} and {:?} is not drawn", index_id.name(), value_id.name()));
                        return;
                    }

                    mark.draw(&slots, &mut canvas);
                });

                if role == MarkRole::Overlay || has_primary_mark(dataset) {
                    return;
                }

                let key = StackKey::new(&options, dataset_index);
                let zero = frame.value(value_scale, base_value(value_scale));

                dataset.points(&mut |point_index, point| {
                    // A missing value leaves its place in the slot empty.
                    let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) else {
                        return;
                    };

                    let center = frame.index(index_scale, x);
                    let (side, thickness) = bar_slot(center, ruler, group_index, group_size, self.category_percentage, self.bar_percentage);

                    // The first bar of a stack grows from zero, or the end of the range nearest it.
                    let (start, end) = stacks.add(&key, x, y);
                    let base = if start == 0.0 { zero } else { frame.value(value_scale, start) };
                    let head = frame.value(value_scale, end);

                    // In the index frame, along the index axis and along the value axis: a
                    // rectangle along the sides, a wedge around the circle.
                    ctx.begin_path();
                    frame.trace_rect(ctx, side, thickness, base, head);

                    ctx.set_fill_style(point_color(point.color(), &options, dataset_index, point_index, &palette));

                    ctx.fill();
                });
            });
        }

        ctx.restore();
    }
}

impl<D: Sequence<dyn BarDataSet>> BarController<D> {
    /// The totals of the normalised stacks at each x (see `StackId::normalized`), over the
    /// datasets drawn as bars: those whose bars a mark replaces don't stack.
    fn normalized_totals(&self, places: &dyn Places) -> NormalizedTotals {
        let mut replaced = vec![];
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            if has_primary_mark(dataset) {
                replaced.push(dataset_index);
            }
        });

        normalized_totals(&self.dataset_sequence, places, self.orientation, &|dataset_index| !replaced.contains(&dataset_index))
    }

    /// Whether any dataset has an overlay mark, drawn on top of the bars.
    fn has_overlays(&self) -> bool {
        let mut overlays = false;
        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| overlays |= has_overlay_mark(dataset));
        overlays
    }

    /// How far stacking moves the bars of the stacked datasets with overlay marks (see
    /// `StackOffsets`), with the sums in the same order as the bars are stacked. Empty without
    /// overlays (`has_overlays`), and for datasets that don't stack.
    fn stack_offsets(&self, places: &dyn Places, has_overlays: bool) -> StackOffsets {
        let mut offsets = StackOffsets::default();

        if !has_overlays {
            return offsets;
        }

        let mut stacks = StackSums::split_by_sign().normalized(self.normalized_totals(places));

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            // Bars a mark replaces don't stack, and a dataset alone adds to no one else's sums.
            if has_primary_mark(dataset) {
                return;
            }

            let options = dataset.options();
            let key = StackKey::new(&options, dataset_index);
            if !matches!(key, StackKey::Stacked { .. }) {
                return;
            }

            let (index_id, value_id) = self.orientation.for_dataset(&options, places).index_and_value_scales(&options);
            let coordinates = Coordinates::new(places, index_id, value_id);
            let recorded = has_overlay_mark(dataset);
            dataset.points(&mut |_index, point| {
                if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
                    let (_, end) = stacks.add(&key, x, y);
                    if recorded {
                        offsets.record(dataset_index, x, end - y);
                    }
                }
            });
        });

        offsets
    }
}

/// Whether one of the dataset's marks is drawn on top of its bars.
fn has_overlay_mark(dataset: &dyn BarDataSet) -> bool {
    let mut overlay = false;
    dataset.bar_marks(&mut |mark| overlay |= mark.role() == MarkRole::Overlay);
    overlay
}

/// Whether one of the dataset's marks replaces its bars.
fn has_primary_mark(dataset: &dyn BarDataSet) -> bool {
    let mut primary = false;
    dataset.bar_marks(&mut |mark| primary |= mark.role() == MarkRole::Primary);
    primary
}

/// Which share of the slot a dataset takes: its stack's, or its own when it has no stack, or when
/// a mark replaces its bars, as those don't stack.
fn slot_key(dataset: &dyn BarDataSet, options: &DataSetOptions, dataset_index: usize) -> StackKey {
    if has_primary_mark(dataset) {
        StackKey::Alone(dataset_index)
    } else {
        StackKey::new(options, dataset_index)
    }
}

/// The value bars grow from: zero when it is inside the value scale's range, otherwise the end of
/// the range nearest to zero.
fn base_value(value_scale: &dyn Scale) -> Scalar {
    0.0_f64.clamp(value_scale.min(), value_scale.max())
}

/// The smallest gap between neighbouring bar positions or neighbouring ticks of the index scale,
/// both along the index axis in the dataset's frame (pixels along a side, angles or distances
/// around a circle), capped at the axis's `length`. It sets how thick a point's group of bars can
/// be without overlapping the next point's.
pub(crate) fn min_sample_size(mut index_positions: Vec<Scalar>, mut tick_positions: Vec<Scalar>, length: Scalar) -> Scalar {
    let mut min = length;

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
pub(crate) fn bar_slot(center: Scalar, ruler: Scalar, group_index: usize, group_size: usize, category_percentage: Scalar, bar_percentage: Scalar) -> (Scalar, Scalar) {
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

    /// The stack offsets are only worked out for what reads them: stacked datasets with an overlay
    /// mark. A chart without overlays, or with overlays only on datasets that don't stack, gets
    /// none, so it allocates nothing for them on its updates and renders.
    #[test]
    fn stack_offsets_are_only_kept_for_stacked_overlays() {
        use crate::{Estimate, RangeDataSetExt};

        const COSTS: StackId = StackId::new("costs");
        let measured = vec![Estimate { x: 0.0, y: 5.0, low: 4.0, high: 7.0 }];

        let plain = BarController::new((vec![10.0].stack(COSTS), vec![5.0].stack(COSTS)));
        assert!(!plain.has_overlays() && plain.stack_offsets(&ScaleBounds::default(), plain.has_overlays()).is_empty(), "stacked bars without overlays need no offsets");

        let unstacked = BarController::new((vec![10.0], measured.clone().error_bars()));
        assert!(unstacked.stack_offsets(&ScaleBounds::default(), unstacked.has_overlays()).is_empty(), "error bars that don't stack are moved by zero");

        let stacked = BarController::new((vec![10.0].stack(COSTS), measured.error_bars().stack(COSTS)));
        let offsets = stacked.stack_offsets(&ScaleBounds::default(), stacked.has_overlays());
        assert!(offsets.dataset(0).is_none(), "the bottom dataset has no overlay, so nothing reads its offsets");
        assert_eq!(super::stack_offset(offsets.dataset(1), 0.0), 10.0, "the error bars sit on top of the 10 below them");
    }

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
