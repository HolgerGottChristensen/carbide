use crate::DataSet;
use crate::mark::AsDataSet;
use carbide::state::IntoReadState;
use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::DatasetController;
use crate::controller::options::{constant, get, option, Opt};
use crate::geometry::{DatumKey, Shapes};
use crate::controller::orientation::Orientation;
use crate::colors::SeriesColors;
use crate::controller::drawing::{polar_frame, register_values, collect_point_bounds, draw_fills, add_lines, draw_lines, draw_points, drawn_lines, first_mark_entry, legend_items, set_legend_visibility, describe_datum};
use crate::interaction::DatumInfo;
use crate::legend::{LegendItem, LegendMarker};
use crate::Fill;
use crate::mark::{LineCanvas, LineDataSet, LegendContext};
use crate::MarkRole;
use crate::controller::stacking::{normalized_totals, stack_offset, StackKey, StackOffsets, StackSums};
use crate::{IntoSequence, Sequence};
use crate::scale::{Coordinates, GridShape, Places, PolarRole, ScaleBounds, ScaleKind, Scales};

/// Draws each dataset as a line through its points, in order, with the points on top. The line
/// follows the dataset's `interpolation`: straight by default, or smooth, monotone or stepped.
///
/// Datasets in the same stack (`DatasetExt::stack`) are drawn at their running totals, with one
/// sum for positive and negative values alike, so a negative value lowers the lines above it, and
/// the bands between neighbouring lines never have gaps. With `Fill::Stack` they make a stacked
/// area chart.
///
/// Its default x scale is a `CategoryScale`, as its datasets are usually lists of values, each at
/// its index. For data with numeric x values, use a `ScatterController` with `show_line(true)`, or
/// set a `LinearScale` with `Chart::scale`.
///
/// A dataset with a mark (see `LineDataSet`) draws the mark in place of its line and points, such
/// as a band between each point's low and high value (`RangeDataSetExt::range`). Those datasets
/// don't stack, and are drawn before the other datasets' lines, as bands are areas.
///
/// `show_line(false)` leaves the lines out, and the areas with them, drawing only the points, at
/// the same places as with lines, stacked where they stack.
///
/// `horizontal` makes the line horizontal, with the same data, as `BarController::horizontal` does for
/// bars: the categories run down the dataset's y scale, the first at the top on a category scale,
/// and the values along its x scale. Everything that follows the categories follows them down:
/// monotone curves stay monotone along them, steps step along them, and areas fill sideways to
/// their baseline.
#[derive(Clone, Debug)]
pub struct LineController<D> {
    pub(crate) dataset_sequence: D,
    palette_offset: Opt<usize>,
    horizontal: Opt<bool>,
    show_line: Opt<bool>,
}

impl<D: Sequence<dyn LineDataSet>> LineController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn LineDataSet, M, Sequence=D>, M>(datasets: I) -> LineController<D> {
        LineController {
            dataset_sequence: datasets.into_sequence(),
            palette_offset: constant(0),
            horizontal: constant(false),
            show_line: constant(true),
        }
    }

    /// Makes the line horizontal with `true`, a value or a state of one: each point's category runs down
    /// the dataset's y scale, the first at the top on a category scale, and its value along the x
    /// scale. The data is read as for a vertical line, so the same datasets draw either way, and
    /// line up with horizontal bars. `false`, vertical, by default.
    pub fn horizontal(mut self, horizontal: impl IntoReadState<bool>) -> LineController<D> {
        self.horizontal = option(horizontal);
        self
    }

    /// Draws the line through each dataset's points with `true`, a value or a state of one, and
    /// with `false` only the points. A dataset's area (`DatasetExt::fill`) is drawn with its line,
    /// so it is left out too, and its baseline no longer counts towards the value scale's bounds.
    /// `true` by default.
    pub fn show_line(mut self, show_line: impl IntoReadState<bool>) -> LineController<D> {
        self.show_line = option(show_line);
        self
    }

    /// Which way the values grow, as `horizontal` was last synced.
    fn orientation(&self) -> Orientation {
        if get(&self.horizontal) { Orientation::Horizontal } else { Orientation::Vertical }
    }

    /// Starts the colours of this controller's datasets `offset` colours into the palette:
    /// dataset `i` takes palette colour `offset + i`, and so do the points of a dataset coloured
    /// by point. Defaults to 0. A chart with several controllers sets it so their datasets don't
    /// share colours, as each controller counts its datasets from 0. Datasets that pick a palette
    /// colour themselves (`DataColor::Palette`) aren't shifted.
    pub fn palette_offset(mut self, offset: usize) -> LineController<D> {
        self.palette_offset = constant(offset);
        self
    }
}

impl<D: Sequence<dyn LineDataSet>> LineController<D> {
    /// The indices of the datasets whose lines a mark replaces.
    fn replaced_lines(&self) -> Vec<usize> {
        let mut replaced = vec![];
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let mut primary = false;
            dataset.line_marks(&mut |mark| primary |= mark.role() == MarkRole::Primary);
            if primary {
                replaced.push(dataset_index);
            }
        });
        replaced
    }

    /// How far stacking moves the lines of the stacked datasets with overlay marks (see
    /// `StackOffsets`), with one running sum per stack, as the lines are stacked. Empty when no
    /// dataset has an overlay, and for datasets that don't stack, including those whose line a mark
    /// replaces (`replaced`).
    fn stack_offsets(&self, places: &dyn Places, replaced: &[usize]) -> StackOffsets {
        let mut offsets = StackOffsets::default();

        let mut overlays = false;
        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| overlays |= has_mark(dataset, MarkRole::Overlay));
        if !overlays {
            return offsets;
        }

        let totals = normalized_totals(&self.dataset_sequence, places, self.orientation(), &|dataset_index| !replaced.contains(&dataset_index));
        let mut sums = StackSums::single().normalized(totals);

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            if replaced.contains(&dataset_index) {
                return;
            }

            let options = dataset.options();
            // A hidden dataset adds nothing to the bounds, its stack or what is drawn.
            if !options.visible {
                return;
            }
            let key = StackKey::new(&options, dataset_index);
            if !matches!(key, StackKey::Stacked { .. }) {
                return;
            }

            let (index_id, value_id) = self.orientation().for_dataset(&options, places).index_and_value_scales(&options);
            let coordinates = Coordinates::new(places, index_id, value_id);
            let recorded = has_mark(dataset, MarkRole::Overlay);
            dataset.points(&mut |_index, point| {
                if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
                    let (_, total) = sums.add(&key, x, y);
                    if recorded {
                        offsets.record(dataset_index, x, total - y);
                    }
                }
            });
        });

        offsets
    }

    /// Draws every dataset's marks with `role`, in dataset order, each on its dataset's scales.
    /// Datasets without such marks are passed over without reading their options.
    fn draw_marks(&self, role: MarkRole, offsets: &StackOffsets, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect, colors: &SeriesColors) {
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            if !has_mark(dataset, role) {
                return;
            }

            let options = dataset.options();
            // A hidden dataset adds nothing to the bounds, its stack or what is drawn.
            if !options.visible {
                return;
            }
            let wants_x_angle = self.orientation().polar_roles().0 == PolarRole::Angle;
            let orientation = self.orientation().for_dataset(&options, scales);
            let (index_id, value_id) = orientation.index_and_value_scales(&options);
            let (Some(index_scale), Some(value_scale)) = (scales.get(index_id), scales.get(value_id)) else {
                return;
            };

            // Around a circle, the canvas draws in the circle's frame (see `LineCanvas`).
            let Some(frame) = polar_frame(scales, index_id, value_id, wants_x_angle) else {
                return;
            };

            let mut canvas = LineCanvas {
                ctx: &mut *ctx,
                coordinates: Coordinates::new(scales, index_id, value_id),
                orientation,
                index_scale,
                value_scale,
                area: chart_area,
                options: &options,
                colors: colors.dataset(options.color, dataset_index),
                stack_offsets: offsets.dataset(dataset_index),
                polar: frame.as_ref(),
            };

            dataset.line_marks(&mut |mark| {
                if mark.role() == role {
                    mark.draw(&mut canvas);
                }
            });
        });
    }
}

/// Whether one of the dataset's marks has `role`.
fn has_mark(dataset: &dyn LineDataSet, role: MarkRole) -> bool {
    let mut found = false;
    dataset.line_marks(&mut |mark| found |= mark.role() == role);
    found
}

impl<D: Sequence<dyn LineDataSet>> DatasetController for LineController<D> {
    /// Horizontal when `horizontal` was last synced `true`.
    fn orientation(&self) -> Orientation {
        Self::orientation(self)
    }

    /// The offset set with `palette_offset`, as last synced.
    fn palette_offset(&self) -> usize {
        get(&self.palette_offset)
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = self.dataset_sequence.sync(env);
        changed |= self.palette_offset.sync(env);
        changed |= self.horizontal.sync(env);
        changed |= self.show_line.sync(env);
        changed
    }

    /// Asks for a category index scale (x, or y when `horizontal`), without the bar chart's offset
    /// or grid offset, then reports each dataset's range: its points, or, when a mark replaces its
    /// line, what the mark draws.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        register_values(&self.dataset_sequence, bounds, self.orientation(), true);
        let (placement, requests) = bounds.split();

        let replaced = self.replaced_lines();
        let offsets = self.stack_offsets(placement, &replaced);

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let (index_scale, value_scale) = self.orientation().for_dataset(&options, placement).index_and_value_scales(&options);
            requests.request_kind(index_scale, ScaleKind::Category);

            // On polar scales, a radar's rings are polygons through the spokes.
            requests.request_grid_shape(index_scale, GridShape::Polygon);
            requests.request_grid_shape(value_scale, GridShape::Polygon);

            // A hidden dataset still asks for its scales, so they keep their kind, but adds no data.
            if !options.visible {
                return;
            }

            // A mark's values move with the dataset's line where it is stacked.
            let coordinates = placement.coordinates(index_scale, value_scale);
            dataset.line_marks(&mut |mark| {
                mark.bounds(&coordinates, &mut |x, low, high| {
                    let offset = stack_offset(offsets.dataset(dataset_index), x);
                    requests.include(index_scale, x, x);
                    if !requests.is_hidden(index_scale, value_scale, x) {
                        requests.include(value_scale, low + offset, high + offset);
                    }
                });
            });
        });

        collect_point_bounds(&self.dataset_sequence, bounds, get(&self.show_line), true, self.orientation(), |dataset_index| !replaced.contains(&dataset_index));
    }

    /// Each named dataset as a line in its colour, a point when lines aren't shown, or a square
    /// when it is filled or a mark, such as a band, replaces its line, unless one of its marks gives an entry of its own
    /// (`LineMark::legend`).
    /// The datum's value, or a mark's values, with the marker its dataset has in the legend.
    fn describe(&self, key: DatumKey, scales: &Scales, env: &mut Environment) -> Option<DatumInfo> {
        let colors = SeriesColors::new(env, get(&self.palette_offset));
        let replaced = self.replaced_lines();
        let show_line = get(&self.show_line);
        describe_datum(&self.dataset_sequence, key, scales, self.orientation(), &colors, env, |dataset, options, dataset_index| {
            let context = LegendContext { colors: colors.dataset(options.color, dataset_index) };
            if let Some((color, marker)) = first_mark_entry(|f| dataset.line_marks(&mut |mark| f(mark.legend(&context)))) {
                return (Some(color), marker);
            }
            let marker = match (replaced.contains(&dataset_index), show_line, options.fill) {
                (true, _, _) => LegendMarker::Square,
                (false, false, _) => LegendMarker::Point,
                (false, true, Fill::None) => LegendMarker::Line,
                (false, true, _) => LegendMarker::Square,
            };
            (None, marker)
        })
    }

    /// Shows or hides the named dataset behind the entry.
    fn legend_visibility(&self, entry: usize, visible: bool) -> bool {
        set_legend_visibility(&self.dataset_sequence, entry, visible)
    }

    fn visit_datasets(&self, f: &mut dyn FnMut(usize, &dyn DataSet)) {
        self.dataset_sequence.foreach(&mut |index, dataset| f(index, dataset.as_dataset()));
    }

    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        let replaced = self.replaced_lines();
        legend_items(
            &self.dataset_sequence,
            &SeriesColors::new(env, get(&self.palette_offset)),
            items,
            |dataset_index, options| match (replaced.contains(&dataset_index), get(&self.show_line), options.fill) {
                (true, _, _) => LegendMarker::Square,
                (false, false, _) => LegendMarker::Point,
                (false, true, Fill::None) => LegendMarker::Line,
                (false, true, _) => LegendMarker::Square,
            },
            |dataset, context| first_mark_entry(|f| dataset.line_marks(&mut |mark| f(mark.legend(context)))),
        );
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect, shapes: &mut Shapes) {
        ctx.save();

        let replaced = self.replaced_lines();
        let lines = drawn_lines(&self.dataset_sequence, scales, chart_area, true, true, self.orientation(), |dataset_index| !replaced.contains(&dataset_index));
        let colors = SeriesColors::new(ctx.env(), get(&self.palette_offset));

        // Marks replacing lines are usually areas, so they go first, under every line; overlays
        // go on top of everything.
        let offsets = self.stack_offsets(scales, &replaced);
        self.draw_marks(MarkRole::Primary, &offsets, ctx, scales, chart_area, &colors);
        if get(&self.show_line) {
            draw_fills(&lines, ctx, scales, chart_area, &colors);
            draw_lines(&lines, ctx, chart_area, &colors);
            add_lines(&lines, shapes, scales, chart_area, &colors);
        }
        draw_points(&lines, ctx, &colors, shapes, |_| true);
        self.draw_marks(MarkRole::Overlay, &offsets, ctx, scales, chart_area, &colors);

        ctx.restore();
    }
}

#[cfg(test)]
mod tests {
    use carbide::environment::Environment;
    use crate::{DatasetExt, Fill};
    use crate::DatasetController;
    use crate::scale::{ScaleBounds, ScaleId, ScaleKind};
    use super::LineController;

    /// The stack offsets are only worked out for stacked datasets with an overlay mark, as for bar
    /// controllers. Plain stacked lines, and error bars on lines that don't stack, get none.
    #[test]
    fn stack_offsets_are_only_kept_for_stacked_overlays() {
        use crate::{Estimate, RangeDataSetExt, StackId};
        use crate::controller::stacking::stack_offset;

        const ALL: StackId = StackId::new("all");
        let measured = vec![Estimate { x: 0.0, y: 5.0, low: 4.0, high: 7.0 }];

        let plain = LineController::new((vec![10.0].stack(ALL), vec![5.0].stack(ALL)));
        assert!(plain.stack_offsets(ScaleBounds::default().placement(), &plain.replaced_lines()).is_empty());

        let unstacked = LineController::new((vec![10.0], measured.clone().error_bars()));
        assert!(unstacked.stack_offsets(ScaleBounds::default().placement(), &unstacked.replaced_lines()).is_empty());

        let stacked = LineController::new((vec![10.0].stack(ALL), measured.error_bars().stack(ALL)));
        let offsets = stacked.stack_offsets(ScaleBounds::default().placement(), &stacked.replaced_lines());
        assert!(offsets.dataset(0).is_none());
        assert_eq!(stack_offset(offsets.dataset(1), 0.0), 10.0, "the error bars sit on the line at the total of 15");
    }

    /// A horizontal line reads the same data as a vertical one, with the scales swapped: the
    /// categories go on the dataset's y scale, which is asked to be a category scale, without the
    /// bars' offset, and the values, with the fill's baseline, on its x scale.
    #[test]
    fn horizontal_lines_put_the_categories_on_the_y_scale() {
        let line = LineController::new(vec![12.0, 19.0, 8.0].fill(Fill::Value(-5.0))).horizontal(true);

        let mut bounds = ScaleBounds::default();
        line.collect_bounds(&mut bounds, &mut Environment::new());

        let y = bounds.requests().get(&ScaleId::Y);
        assert_eq!(y.range(), Some((0.0, 2.0)), "the categories are the points' x values");
        assert_eq!(y.kind(), Some(ScaleKind::Category));
        assert!(!y.offset());

        let x = bounds.requests().get(&ScaleId::X);
        assert_eq!(x.range(), Some((-5.0, 19.0)), "the values, and the baseline they fill to");
        assert_eq!(x.kind(), None);
    }

    /// Without lines, a filled dataset has no area, so its baseline is left out of the value
    /// scale's bounds, and the legend shows a point for it.
    #[test]
    fn hidden_lines_leave_out_their_areas_and_show_points_in_the_legend() {
        use crate::legend::LegendMarker;

        let line = LineController::new(vec![12.0, 19.0, 8.0].fill(Fill::Value(-5.0)).label("Sales")).show_line(false);

        let mut bounds = ScaleBounds::default();
        line.collect_bounds(&mut bounds, &mut Environment::new());
        assert_eq!(bounds.requests().get(&ScaleId::Y).range(), Some((8.0, 19.0)), "only the points, not the baseline");

        let mut items = vec![];
        line.legend_items(&Environment::new(), &mut items);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].marker, LegendMarker::Point);
    }
}
