use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use indexmap::IndexMap;
use crate::controller::DatasetController;
use crate::controller::drawing::{dataset_color, dataset_scales};
use crate::{DataColor, DataSet, DataSetSequence};
use crate::sequence::IntoSequence;
use crate::scale::{Axis, CategoryScale, Scale, ScaleBounds, ScaleId, Scales};

/// Draws each dataset as vertical bars, one per point, growing from zero to the point's y. Based
/// on Chart.js's bar controller (`controller.bar.js`):
///
/// - The x scale gets an offset (half a tick step at each end), so the first and last bars aren't
///   cut in half at the chart area's edges.
/// - Zero is always included in the y bounds, like Chart.js's `beginAtZero: true` for bars.
/// - The datasets in this controller that share an x scale are grouped side by side. Each point
///   gets a slot `category_percentage` of the smallest gap between neighbouring x values or ticks;
///   the slot is split evenly between the datasets, and each bar is `bar_percentage` of its share.
///
/// Bars in two different bar controllers on the same x scale are not grouped with each other.
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
    /// up. Defaults to 0.8, as in Chart.js.
    pub fn category_percentage(mut self, category_percentage: Scalar) -> BarController<D> {
        self.category_percentage = category_percentage;
        self
    }

    /// How much of its share of the group's slot each bar takes up. Defaults to 0.9, as in
    /// Chart.js.
    pub fn bar_percentage(mut self, bar_percentage: Scalar) -> BarController<D> {
        self.bar_percentage = bar_percentage;
        self
    }
}

impl<D: DataSetSequence<Scalar, Scalar, Scalar>> DatasetController for BarController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| {
            // An empty dataset has no bounds and contributes nothing.
            let (Some((x_min, y_min, _)), Some((x_max, y_max, _))) = (dataset.min(), dataset.max()) else {
                return;
            };

            let options = dataset.options();

            bounds.include(&options.x_scale, x_min, x_max);
            bounds.include(&options.y_scale, y_min.min(0.0), y_max.max(0.0));
            bounds.request_offset(&options.x_scale);
        });
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        // First pass: which datasets are grouped together (per x scale, in sequence order), and
        // every bar's x position, for the smallest gap between neighbours.
        let mut group_sizes: IndexMap<ScaleId, usize> = IndexMap::new();
        let mut positions: IndexMap<ScaleId, Vec<Scalar>> = IndexMap::new();
        let mut places: Vec<Option<(ScaleId, usize)>> = vec![];

        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| {
            let options = dataset.options();

            let Some((x_scale, _)) = dataset_scales(scales, &options) else {
                places.push(None);
                return;
            };

            let group_size = group_sizes.entry(options.x_scale.clone()).or_insert(0);
            places.push(Some((options.x_scale.clone(), *group_size)));
            *group_size += 1;

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

        // Second pass: the bars.
        ctx.save();

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let Some(&Some((ref x_id, group_index))) = places.get(dataset_index) else {
                return;
            };

            let options = dataset.options();

            let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
                return;
            };

            let group_size = group_sizes[x_id];
            let ruler = rulers.get(x_id).copied().unwrap_or(chart_area.width());

            let y_pixel = |value: Scalar| chart_area.bottom() + (1.0 - y_scale.normalize(value)) * chart_area.height();
            let base = y_pixel(base_value(y_scale));

            dataset.points(&mut |_index, point| {
                let center = chart_area.left() + x_scale.normalize(point.x()) * chart_area.width();
                let (left, width) = bar_slot(center, ruler, group_index, group_size, self.category_percentage, self.bar_percentage);
                let head = y_pixel(point.y());

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

    /// Bar charts sit on a category scale along x, with grid lines between the bar groups, as in
    /// Chart.js's bar `overrides` (`_index_: {type: 'category', offset: true, grid: {offset:
    /// true}}`). The offset itself is requested in `collect_bounds`, so it also applies to an x
    /// scale set with `Chart::scale`.
    fn default_scale(&self, id: &ScaleId) -> Option<Box<dyn Scale>> {
        (*id == ScaleId::X).then(|| Box::new(CategoryScale::new(Axis::Horizontal).grid_offset(true)) as Box<dyn Scale>)
    }
}

/// The value bars grow from: zero when it is inside the y scale's range, otherwise the end of the
/// range nearest to zero (Chart.js's `getBaseValue`).
fn base_value(y_scale: &dyn Scale) -> Scalar {
    0.0_f64.clamp(y_scale.min(), y_scale.max())
}

/// The smallest gap, in pixels, between neighbouring bar positions or neighbouring ticks of the x
/// scale, capped at the chart area's width (Chart.js's `computeMinSampleSize`). It sets how wide
/// a point's group of bars can be without overlapping the next point's.
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
/// `bar_percentage` of share number `group_index`, centred in it. Chart.js's
/// `computeFitCategoryTraits` and `_calculateBarIndexPixels`.
fn bar_slot(center: Scalar, ruler: Scalar, group_index: usize, group_size: usize, category_percentage: Scalar, bar_percentage: Scalar) -> (Scalar, Scalar) {
    let slot = ruler * category_percentage;
    let share = slot / group_size.max(1) as Scalar;
    let share_center = center - slot / 2.0 + share * group_index as Scalar + share / 2.0;
    let width = share * bar_percentage;

    (share_center - width / 2.0, width)
}

#[cfg(test)]
mod tests {
    use super::bar_slot;

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
