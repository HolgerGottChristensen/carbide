use carbide::state::IntoReadState;
use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::DatasetController;
use crate::controller::options::{constant, get, option, Opt};
use crate::geometry::{DatumKey, Shapes};
use crate::controller::orientation::Orientation;
use crate::colors::SeriesColors;
use crate::controller::drawing::{polar_frame, register_values, collect_point_bounds, dataset_scales, draw_fills, add_lines, draw_lines, draw_points, drawn_lines, first_mark_entry, legend_items, set_legend_visibility, describe_datum};
use crate::interaction::DatumInfo;
use crate::legend::{LegendItem, LegendMarker};
use crate::Fill;
use crate::mark::{MarkRole, ScatterCanvas, ScatterDataSet, SizeScale, LegendContext};
use crate::{IntoSequence, Sequence};
use crate::scale::{Coordinates, GridShape, ScaleBounds, Scales};

/// Draws each dataset's points: points only by default. With `show_line(true)` it also draws a line
/// through them in order, following the dataset's `interpolation`, as a line chart does.
///
/// A dataset with a mark (see `ScatterDataSet`) draws the mark in place of its points, such as
/// bubbles (`SizedDataSetExt::bubbles`). Marks that draw sizes share one size scale across the
/// controller's datasets, set with `radius_range`.
///
/// Its x and y scales stay linear, so it suits data with numeric x values. For lists of values at
/// their indices, use a `LineController`.
#[derive(Clone, Debug)]
pub struct ScatterController<D> {
    dataset_sequence: D,
    show_line: Opt<bool>,
    palette_offset: Opt<usize>,
    min_radius: Opt<Scalar>,
    max_radius: Opt<Scalar>,
}

impl<D: Sequence<dyn ScatterDataSet>> ScatterController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn ScatterDataSet, M, Sequence=D>, M>(datasets: I) -> ScatterController<D> {
        ScatterController {
            dataset_sequence: datasets.into_sequence(),
            show_line: constant(false),
            palette_offset: constant(0),
            min_radius: constant(4.0),
            max_radius: constant(30.0),
        }
    }

    /// Starts the colours of this controller's datasets `offset` colours into the palette:
    /// dataset `i` takes palette colour `offset + i`, and so do the points of a dataset coloured
    /// by point. Defaults to 0. A chart with several controllers sets it so their datasets don't
    /// share colours, as each controller counts its datasets from 0. Datasets that pick a palette
    /// colour themselves (`DataColor::Palette`) aren't shifted.
    pub fn palette_offset(mut self, offset: usize) -> ScatterController<D> {
        self.palette_offset = constant(offset);
        self
    }

    /// The radii sizes are drawn at, in pixels, such as bubbles' (see `SizedDataSetExt::bubbles`):
    /// the largest size among the controller's datasets gets `max`, the others an area in
    /// proportion to it, and none is smaller than `min`. Defaults to 4 and 30. Each a value or a
    /// state of one.
    pub fn radius_range(mut self, min: impl IntoReadState<Scalar>, max: impl IntoReadState<Scalar>) -> ScatterController<D> {
        self.min_radius = option(min);
        self.max_radius = option(max);
        self
    }

    /// Also draws a line through each dataset's points, under the points, a value or a state of
    /// one. Defaults to `false`.
    pub fn show_line(mut self, show_line: impl IntoReadState<bool>) -> ScatterController<D> {
        self.show_line = option(show_line);
        self
    }
}

impl<D: Sequence<dyn ScatterDataSet>> ScatterController<D> {
    /// The indices of the datasets whose points a mark replaces.
    fn replaced_points(&self) -> Vec<usize> {
        let mut replaced = vec![];
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let mut primary = false;
            dataset.scatter_marks(&mut |mark| primary |= mark.role() == MarkRole::Primary);
            if primary {
                replaced.push(dataset_index);
            }
        });
        replaced
    }
}

/// Whether one of the dataset's marks has `role`, or, with `None`, whether it has any mark.
fn has_mark(dataset: &dyn ScatterDataSet, role: Option<MarkRole>) -> bool {
    let mut found = false;
    dataset.scatter_marks(&mut |mark| found |= role.is_none_or(|role| mark.role() == role));
    found
}

impl<D: Sequence<dyn ScatterDataSet>> DatasetController for ScatterController<D> {
    /// The offset set with `palette_offset`, as last synced.
    fn palette_offset(&self) -> usize {
        get(&self.palette_offset)
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = self.dataset_sequence.sync(env);
        changed |= self.palette_offset.sync(env);
        changed |= self.min_radius.sync(env);
        changed |= self.max_radius.sync(env);
        changed |= self.show_line.sync(env);
        changed
    }

    /// Reports each dataset's points, and what its marks draw. A dataset whose points a mark
    /// replaces reports only what the mark draws, as its other points aren't drawn, unless a line
    /// runs through all its points (`show_line`).
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        register_values(&self.dataset_sequence, bounds, Orientation::Vertical, true);

        let replaced = self.replaced_points();
        collect_point_bounds(&self.dataset_sequence, bounds, get(&self.show_line), false, Orientation::Vertical, |dataset_index| {
            get(&self.show_line) || !replaced.contains(&dataset_index)
        });

        let (placement, requests) = bounds.split();

        // On polar scales, the rings around scattered points are circles.
        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| {
            let options = dataset.options();
            requests.request_grid_shape(&options.x_scale, GridShape::Circle);
            requests.request_grid_shape(&options.y_scale, GridShape::Circle);
        });

        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| {
            if !has_mark(dataset, None) {
                return;
            }

            let options = dataset.options();
            // A hidden dataset adds nothing to the bounds, its stack or what is drawn.
            if !options.visible {
                return;
            }
            let coordinates = placement.coordinates(&options.x_scale, &options.y_scale);
            dataset.scatter_marks(&mut |mark| {
                mark.bounds(&coordinates, &mut |x, y| {
                    requests.include(&options.x_scale, x, x);
                    if !requests.is_hidden(&options.x_scale, &options.y_scale, x) {
                        requests.include(&options.y_scale, y, y);
                    }
                });
            });
        });
    }

    /// Each named dataset as a point in its colour, or, when lines are shown, a line, or a square
    /// when it is filled, unless one of its marks gives an entry of its own (`ScatterMark::legend`),
    /// such as a bubble.
    /// The datum's values, as a point, or as its mark draws it, such as a bubble.
    fn describe(&self, key: DatumKey, scales: &Scales, env: &mut Environment) -> Option<DatumInfo> {
        let colors = SeriesColors::new(env, get(&self.palette_offset));
        describe_datum(&self.dataset_sequence, key, scales, Orientation::Vertical, &colors, env, |dataset, options, dataset_index| {
            let context = LegendContext { colors: colors.dataset(options.color, dataset_index) };
            match first_mark_entry(|f| dataset.scatter_marks(&mut |mark| f(mark.legend(&context)))) {
                // A mark's marker, but each point keeps its own colour.
                Some((_, marker)) => (None, marker),
                None => (None, LegendMarker::Point),
            }
        })
    }

    /// Shows or hides the named dataset behind the entry.
    fn legend_visibility(&self, entry: usize, visible: bool) -> bool {
        set_legend_visibility(&self.dataset_sequence, entry, visible)
    }

    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        legend_items(
            &self.dataset_sequence,
            &SeriesColors::new(env, get(&self.palette_offset)),
            items,
            |_, options| match (get(&self.show_line), options.fill) {
                (false, _) => LegendMarker::Point,
                (true, Fill::None) => LegendMarker::Line,
                (true, _) => LegendMarker::Square,
            },
            |dataset, context| first_mark_entry(|f| dataset.scatter_marks(&mut |mark| f(mark.legend(context)))),
        );
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect, shapes: &mut Shapes) {
        ctx.save();

        // Scatter charts don't stack: each dataset's points are drawn at their own values.
        let lines = drawn_lines(&self.dataset_sequence, scales, chart_area, false, false, Orientation::Vertical, |_| true);

        let colors = SeriesColors::new(ctx.env(), get(&self.palette_offset));

        if get(&self.show_line) {
            draw_fills(&lines, ctx, scales, chart_area, &colors);
            draw_lines(&lines, ctx, chart_area, &colors);
            add_lines(&lines, shapes, scales, chart_area, &colors);
        }

        // The points of datasets whose points a mark replaces are drawn by the mark instead.
        let replaced = self.replaced_points();
        draw_points(&lines, ctx, &colors, shapes, |dataset_index| !replaced.contains(&dataset_index));

        // One size scale for every dataset's marks, so their sizes compare, hidden ones included,
        // so the bubbles shown keep their sizes when another dataset is hidden or shown.
        let mut max_size: Option<Scalar> = None;
        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| {
            dataset.scatter_marks(&mut |mark| {
                mark.sizes(&mut |size| {
                    if size > 0.0 {
                        max_size = Some(max_size.map_or(size, |max| max.max(size)));
                    }
                });
            });
        });
        let sizes = SizeScale { min_radius: get(&self.min_radius), max_radius: get(&self.max_radius), max_size };

        // The marks replacing points, then the overlays on top of everything.
        for role in [MarkRole::Primary, MarkRole::Overlay] {
            self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
                // Datasets without such marks are passed over without reading their options.
                if !has_mark(dataset, Some(role)) {
                    return;
                }

                let options = dataset.options();
                if !options.visible {
                    return;
                }
                let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
                    return;
                };

                // Around a circle, marks are drawn where the points are. Scatter datasets ask for
                // x to be the angle; drawn with the other roles, a mark would leave its points.
                let Some(frame) = polar_frame(scales, &options.x_scale, &options.y_scale, true) else {
                    return;
                };

                let mut canvas = ScatterCanvas {
                    ctx: &mut *ctx,
                    coordinates: Coordinates::new(scales, &options.x_scale, &options.y_scale),
                    x_scale,
                    y_scale,
                    area: chart_area,
                    dataset_index,
                    colors: colors.dataset(options.color, dataset_index),
                    sizes,
                    polar: frame.as_ref(),
                    shapes: &mut *shapes,
                    x_id: &options.x_scale,
                };

                dataset.scatter_marks(&mut |mark| {
                    if mark.role() == role {
                        mark.draw(&mut canvas);
                    }
                });
            });
        }

        ctx.restore();
    }
}
