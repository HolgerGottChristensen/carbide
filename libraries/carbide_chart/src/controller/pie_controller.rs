use carbide::state::IntoReadState;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use carbide::color::Color;
use carbide::draw::{Alignment, Dimension, Position, Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataSetOptions, PieDataSet, Value};
use crate::mark::MarkRole;
use crate::controller::{bar_slot, DatasetController};
use crate::controller::options::{constant, get, option, Opt};
use crate::geometry::{Datum, Shapes};
use crate::controller::bar_controller::min_sample_size;
use crate::controller::drawing::{point_color, polar_frame, Palette, PolarFrame};
use crate::controller::line_path::PathSink;
use crate::controller::orientation::{is_polar, Orientation};
use crate::legend::{LegendItem, LegendMarker};
use crate::mark::BarFrame;
use crate::scale::{GridShape, ScaleBounds, ScaleKind, Scales, System};
use crate::scale::polar::{angle_range, direction};
use crate::sequence::{IntoSequence, Sequence};
use crate::style::{axis_style, series_style};

/// How far a leader line runs out from its slice before it turns level.
const RADIAL_RUN: Scalar = 12.0;
/// How far a leader line runs level, from the turn to the label's column.
const LEVEL_RUN: Scalar = 10.0;
/// The gap between the end of a leader line and its label.
const LABEL_GAP: Scalar = 3.0;
/// The smallest gap between two labels one above the other.
const LABEL_SPACING: Scalar = 4.0;
/// How far a ring reaches into the touching ring outside it, in pixels.
const SEAM_OVERLAP: Scalar = 0.75;
/// The gap between neighbouring rings, in pixels, unless set (see `PieController::ring_spacing`).
pub const DEFAULT_RING_SPACING: Scalar = 4.0;
/// The width of each part's outline, in pixels, unless set (see `PieController::outline_width`).
pub const DEFAULT_OUTLINE_WIDTH: Scalar = 2.0;

/// One part of a whole, a slice of a pie: its name, its value, and its share of its dataset's
/// total, from 0 to 100, as a pie controller's label text reads it (see `PieController::label_text`).
#[derive(Clone, Debug, PartialEq)]
pub struct PiePart<'a> {
    pub name: &'a str,
    pub value: Scalar,
    pub share: Scalar,
}

/// Draws each dataset as a whole and its points as its parts: around a circle, a pie, a doughnut,
/// or nested rings; along the sides, a bar of 100% per dataset.
///
/// Each point is a part: its y is its size, and its x, when it is text (`("Red", 12.0)`), its name;
/// otherwise its index names it. A part is its share of its dataset's total, the absolute values
/// counting, so a negative value's part is the size of its absolute value; a missing value has no
/// part. Parts are coloured by point by default, each the next palette colour, so a dataset left
/// coloured `ByDataSet` is drawn `ByPoint`. The legend has an entry for each part, by name.
///
/// Each dataset is placed on its x scale, at its name (its label, or its index), and its parts'
/// running shares, 0 to 100, on its y scale, as horizontal bars place categories and values. With no
/// scales set, a chart whose first controller this is makes its default scales radial
/// (`prefers_polar`), so `Chart::new(PieController::new(values))` draws a pie, and asks them not to
/// draw their grid, ticks and labels, so only the slices are drawn. Around a circle, x is the
/// radius: each dataset a ring in an equal slot out along it, the first innermost, and an inner
/// radius on the radius scale makes a doughnut; the outermost ring's parts are labelled outside the
/// circle (`labels`). With cartesian scales set (`Chart::scale`), each dataset is an upright bar of
/// 100% instead, its parts stacked from zero, with its axes.
#[derive(Clone)]
pub struct PieController<D> {
    pub(crate) dataset_sequence: D,
    palette_offset: Opt<usize>,
    category_percentage: Opt<Scalar>,
    bar_percentage: Opt<Scalar>,
    spacing: Opt<Scalar>,
    ring_spacing: Opt<Scalar>,
    outline_width: Opt<Scalar>,
    label_text: Option<Arc<dyn Fn(&PiePart) -> String>>,
    labels: Opt<bool>,
}

impl<D: Debug> Debug for PieController<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PieController")
            .field("dataset_sequence", &self.dataset_sequence)
            .field("palette_offset", &get(&self.palette_offset))
            .field("category_percentage", &get(&self.category_percentage))
            .field("bar_percentage", &get(&self.bar_percentage))
            .field("spacing", &get(&self.spacing).max(0.0))
            .field("ring_spacing", &get(&self.ring_spacing).max(0.0))
            .field("outline_width", &get(&self.outline_width).max(0.0))
            .field("labels", &get(&self.labels))
            .finish_non_exhaustive()
    }
}

impl<D: Sequence<dyn PieDataSet>> PieController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset: one pie, or one ring each.
    pub fn new<I: IntoSequence<dyn PieDataSet, M, Sequence=D>, M>(datasets: I) -> PieController<D> {
        PieController {
            dataset_sequence: datasets.into_sequence(),
            palette_offset: constant(0),
            category_percentage: constant(1.0),
            bar_percentage: constant(1.0),
            spacing: constant(0.0),
            ring_spacing: constant(DEFAULT_RING_SPACING),
            outline_width: constant(DEFAULT_OUTLINE_WIDTH),
            label_text: None,
            labels: constant(true),
        }
    }

    /// Starts the parts' colours `offset` colours into the palette (see
    /// `BarController::palette_offset`). Defaults to 0.
    pub fn palette_offset(mut self, offset: usize) -> PieController<D> {
        self.palette_offset = constant(offset);
        self
    }

    /// How much of its slot each dataset's ring (or bar) takes, as `BarController` splits its
    /// slots: 1 by default, so a pie reaches its centre and rings touch; less leaves gaps between
    /// rings. A value or a state of one.
    pub fn category_percentage(mut self, category_percentage: impl IntoReadState<Scalar>) -> PieController<D> {
        self.category_percentage = option(category_percentage);
        self
    }

    /// How much of its share of the slot each ring takes, after `category_percentage`: 1 by
    /// default. A value or a state of one.
    pub fn bar_percentage(mut self, bar_percentage: impl IntoReadState<Scalar>) -> PieController<D> {
        self.bar_percentage = option(bar_percentage);
        self
    }

    /// A gap of `spacing` pixels between neighbouring parts, kept out of the parts' sizes:
    /// measured across the middle of the ring around a circle, and along the bar along a side.
    /// 0 by default; a value or a state of one, below 0 taken as 0.
    pub fn spacing(mut self, spacing: impl IntoReadState<Scalar>) -> PieController<D> {
        self.spacing = option(spacing);
        self
    }

    /// A gap of `spacing` pixels between neighbouring rings around a circle, so nested rings can be
    /// told apart: `DEFAULT_RING_SPACING` (4) by default. Only between rings: a pie still reaches
    /// its centre, and the outer ring the rim. 0 lets rings touch. A value or a state of one, below
    /// 0 taken as 0.
    pub fn ring_spacing(mut self, spacing: impl IntoReadState<Scalar>) -> PieController<D> {
        self.ring_spacing = option(spacing);
        self
    }

    /// How wide the outline around each part is, in pixels, centred on its edges, in the
    /// `SeriesStyle`'s outline colour (the background by default), so neighbouring slices stay
    /// apart even when they share a colour: `DEFAULT_OUTLINE_WIDTH` (2) by default. A part's
    /// outline is at most a quarter of its thickness and its length, so thin parts keep their
    /// colour. 0 draws none. A value or a state of one, below 0 taken as 0.
    pub fn outline_width(mut self, width: impl IntoReadState<Scalar>) -> PieController<D> {
        self.outline_width = option(width);
        self
    }

    /// Whether the outermost ring's parts are labelled outside the circle, joined to their
    /// slices by leader lines. `true` by default; a value or a state of one.
    pub fn labels(mut self, labels: impl IntoReadState<bool>) -> PieController<D> {
        self.labels = option(labels);
        self
    }

    /// The text of each part's label outside the circle, from the part: its name by default.
    /// `|part| format!("{} {:.0}%", part.name, part.share)` shows its share too.
    pub fn label_text(mut self, text: impl Fn(&PiePart) -> String + 'static) -> PieController<D> {
        self.label_text = Some(Arc::new(text));
        self
    }

    /// The text of the label for `part` of a whole of `total`.
    fn text(&self, part: &Part, total: Scalar) -> String {
        match &self.label_text {
            Some(text) => text(&PiePart { name: &part.name, value: part.value, share: part.size / total * 100.0 }),
            None => part.name.clone(),
        }
    }
}

/// A dataset's parts: each drawn point's index, name, colour, value and size, and their total.
pub(crate) struct Parts {
    pub(crate) parts: Vec<Part>,
    pub(crate) total: Scalar,
}

pub(crate) struct Part {
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) color: DataColor,
    pub(crate) value: Scalar,
    /// The part's share of the whole comes from this: its value's absolute value.
    pub(crate) size: Scalar,
    /// How far out its slice reaches, from a mark that replaces the slices (`RadiiMark`), or
    /// `None` for a slice reaching the rim.
    pub(crate) radius: Option<Scalar>,
}

impl Parts {
    /// The parts of `dataset`: its points with a y, their sizes the absolute values, and their
    /// radii from the dataset's mark that sets them, if it has one.
    pub(crate) fn of(dataset: &dyn PieDataSet) -> Parts {
        let mut radii: Vec<(usize, Scalar)> = vec![];
        dataset.pie_marks(&mut |mark| {
            if mark.role() == MarkRole::Primary {
                mark.sizes(&mut |index, size| radii.push((index, size)));
            }
        });

        let mut parts = vec![];
        dataset.points(&mut |index, point| {
            let Some(value) = point.y().and_then(Value::number) else {
                return;
            };

            let name = match point.x() {
                Some(Value::Text(text)) => text.to_string(),
                _ => index.to_string(),
            };

            let radius = radii.iter().find(|(at, _)| *at == index).map(|(_, size)| *size);
            parts.push(Part { index, name, color: point.color(), value, size: value.abs(), radius });
        });

        let total = parts.iter().map(|part| part.size).sum();
        Parts { parts, total }
    }

    /// How far out `part`'s slice reaches, as a share of its ring, against `largest`, the largest
    /// radius in the controller: 1 without a radius, in proportion to it otherwise, and 0, no
    /// slice, for a radius of zero or less.
    pub(crate) fn reach(part: &Part, largest: Scalar) -> Scalar {
        match part.radius {
            None => 1.0,
            Some(radius) if radius > 0.0 && largest > 0.0 => (radius / largest).min(1.0),
            Some(_) => 0.0,
        }
    }

    /// Each part with the shares it runs between, from 0 to 100; none when the total is zero.
    pub(crate) fn shares(&self) -> Vec<(&Part, Scalar, Scalar)> {
        if self.total <= 0.0 {
            return vec![];
        }

        let mut reached = 0.0;
        self.parts.iter().map(|part| {
            let start = reached;
            reached += part.size / self.total * 100.0;
            (part, start, reached)
        }).collect()
    }
}

/// The colour a part is drawn in: its point's own colour if it sets one, its dataset's if that
/// picks one, and otherwise the palette colour for its index, as a dataset coloured `ByPoint`.
fn part_color(part: &Part, options: &DataSetOptions, dataset_index: usize, palette: &Palette) -> Color {
    let color = match (part.color, options.color) {
        (DataColor::ByDataSet, DataColor::ByDataSet) => DataColor::ByPoint,
        (color, _) => color,
    };
    point_color(color, options, dataset_index, part.index, palette)
}

/// The name of a dataset, its category on the x scale: its label, or its index.
fn dataset_name(options: &DataSetOptions, dataset_index: usize) -> String {
    options.label.clone().unwrap_or_else(|| dataset_index.to_string())
}

/// Where a dataset is drawn: its options, its place on its x scale, and, around a circle, how it
/// lies around it.
struct Placed {
    options: DataSetOptions,
    x: Scalar,
    polar: Option<PolarFrame>,
}

impl Placed {
    fn frame(&self, chart_area: Rect) -> BarFrame<'_> {
        match &self.polar {
            Some(polar) => BarFrame::Around(polar),
            None => BarFrame::Sides { orientation: Orientation::Vertical, area: chart_area },
        }
    }
}

impl<D: Sequence<dyn PieDataSet>> PieController<D> {
    /// Where each dataset is drawn, `None` for those that aren't, and the ruler their slots are
    /// split by: the smallest gap between the datasets' places, or the whole axis for one.
    fn layout(&self, scales: &Scales, chart_area: Rect) -> (Vec<Option<Placed>>, Scalar) {
        let mut placed = vec![];
        let mut positions = vec![];
        let mut ticks = vec![];
        let mut length = None;

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let (Some(x_scale), Some(_)) = (scales.get(&options.x_scale), scales.get(&options.y_scale)) else {
                placed.push(None);
                return;
            };

            // A pie's x is its radius around a circle: the roles of a horizontal controller.
            let Some(polar) = polar_frame(scales, &options.x_scale, &options.y_scale, false) else {
                placed.push(None);
                return;
            };

            let x = x_scale.category_index(&dataset_name(&options, dataset_index)).unwrap_or(dataset_index as Scalar);
            let place = Placed { options, x, polar };
            let frame = place.frame(chart_area);

            positions.push(frame.index(x_scale, x));
            if length.is_none() {
                ticks = x_scale.ticks().iter().map(|tick| frame.index(x_scale, *tick)).collect();
                length = Some(frame.index_length());
            }

            placed.push(Some(place));
        });

        let ruler = min_sample_size(positions, ticks, length.unwrap_or(0.0));
        (placed, ruler)
    }

    /// The largest radius of any part in the controller, which every slice's reach is measured
    /// against (see `RadiiMark`), so slices compare across rings; 0 when no part has one.
    fn largest_radius(&self) -> Scalar {
        let mut largest: Scalar = 0.0;
        self.dataset_sequence.foreach(&mut |_index, dataset| {
            for part in Parts::of(dataset).parts {
                if let Some(radius) = part.radius {
                    largest = largest.max(radius);
                }
            }
        });
        largest
    }

    /// The outermost ring around a circle, whose parts are labelled outside it: its dataset's
    /// index, and each part's point index, its middle as a screen angle, its share of the whole
    /// (0 to 1), and its text, leaving out parts of no size; `None` when labels are off or no
    /// dataset is drawn around a circle. It reads only the
    /// fitted scales, not the circle, so the chart can ask for it before fitting the circle.
    fn outer_ring_labels(&self, scales: &Scales) -> Option<(usize, Vec<(usize, Scalar, Scalar, String)>)> {
        if !get(&self.labels) {
            return None;
        }

        // The ring furthest out: the polar dataset furthest along its radius scale.
        let mut outer: Option<(usize, Scalar)> = None;
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            if scales.system(&options.x_scale, &options.y_scale) != Some(System::Polar { x_is_angle: false }) {
                return;
            }
            let Some(radius_scale) = scales.get(&options.x_scale) else {
                return;
            };

            let x = radius_scale.category_index(&dataset_name(&options, dataset_index)).unwrap_or(dataset_index as Scalar);
            let reach = radius_scale.normalize(x);
            if outer.is_none_or(|(_, furthest)| reach > furthest) {
                outer = Some((dataset_index, reach));
            }
        });
        let (outer_index, _) = outer?;

        let mut labels = vec![];
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            if dataset_index != outer_index {
                return;
            }
            let Some(angle_scale) = scales.get(&dataset.options().y_scale) else {
                return;
            };
            let (start, sweep) = angle_range(angle_scale);

            let parts = Parts::of(dataset);
            for (part, from, to) in parts.shares() {
                if part.size <= 0.0 {
                    continue;
                }
                let middle = angle_scale.normalize((from + to) / 2.0).clamp(0.0, 1.0);
                labels.push((part.index, start + middle * sweep, part.size / parts.total, self.text(part, parts.total)));
            }
        });

        Some((outer_index, labels))
    }
}

impl<D: Sequence<dyn PieDataSet>> DatasetController for PieController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = self.dataset_sequence.sync(env);
        changed |= self.palette_offset.sync(env);
        changed |= self.category_percentage.sync(env);
        changed |= self.bar_percentage.sync(env);
        changed |= self.spacing.sync(env);
        changed |= self.ring_spacing.sync(env);
        changed |= self.outline_width.sync(env);
        changed |= self.labels.sync(env);
        changed
    }

    /// Places each dataset at its name on its x scale, asking for a category scale with each
    /// whole in the middle of its slot, and its parts' shares from 0 to 100 on its y scale. Around
    /// a circle, asks for the x scale to be the radius and the y scale the angle, for circle rings,
    /// and for the default scales to leave out their grid, ticks and labels.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let (x_id, y_id) = (&options.x_scale, &options.y_scale);

            if is_polar(&options, &*bounds) {
                Orientation::Horizontal.request_polar_roles(&options, bounds);
                for id in [x_id, y_id] {
                    bounds.request_grid_shape(id, GridShape::Circle);
                    bounds.request_hidden_decoration(id);
                }
            }

            let x = bounds.register_category(x_id, &dataset_name(&options, dataset_index)) as Scalar;
            bounds.request_kind(x_id, ScaleKind::Category);
            bounds.request_offset(x_id);
            bounds.include(x_id, x, x);
            bounds.include(y_id, 0.0, 100.0);
        });
    }

    /// An entry for each part, by name, in its colour, each name once.
    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        let palette = Palette::from_env(env, get(&self.palette_offset));
        let mut named: Vec<String> = vec![];

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            for part in Parts::of(dataset).parts {
                if named.contains(&part.name) {
                    continue;
                }

                let color = part_color(&part, &options, dataset_index, &palette);
                items.push(LegendItem { label: part.name.clone(), color, marker: LegendMarker::Square });
                named.push(part.name);
            }
        });
    }

    fn prefers_polar(&self) -> bool {
        true
    }

    /// The labels outside the circle, each in its side's column: reported straight to the left or
    /// right of the circle, as far out as the leader lines reach, so the circle leaves room for
    /// the columns.
    fn polar_labels(&self, scales: &Scales, measure: &mut dyn FnMut(&str) -> Dimension, f: &mut dyn FnMut(Scalar, Dimension, Scalar)) {
        let Some((_, labels)) = self.outer_ring_labels(scales) else {
            return;
        };

        for (_, angle, _, text) in labels {
            let size = measure(&text);
            let side = if direction(angle).x >= 0.0 { std::f64::consts::FRAC_PI_2 } else { -std::f64::consts::FRAC_PI_2 };
            f(side, size, RADIAL_RUN + LEVEL_RUN + LABEL_GAP);
        }
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect, shapes: &mut Shapes) {
        let (placed, ruler) = self.layout(scales, chart_area);
        let palette = Palette::new(ctx, get(&self.palette_offset));
        let outline = series_style(ctx.env()).outline_color(ctx.env());
        ctx.save();

        // Rings that touch the next one out reach a little into it, so the seam between two rings,
        // whose curved edges are each flattened into their own straight pieces, leaves no gaps.
        // With ring spacing, each ring gives up half the gap on each side it has a neighbour.
        let touching = get(&self.category_percentage) * get(&self.bar_percentage) >= 1.0;
        let rings = || placed.iter().flatten().filter(|place| place.polar.is_some()).map(|place| place.x);
        let outermost = rings().fold(Scalar::NEG_INFINITY, Scalar::max);
        let innermost = rings().fold(Scalar::INFINITY, Scalar::min);
        let half_gap = get(&self.ring_spacing).max(0.0) / 2.0;
        // The parts drawn, by dataset and point index, with how far out each reaches in the
        // dataset's frame, so only those are labelled, and their lines start where they end.
        let mut drawn: Vec<(usize, usize, Scalar)> = vec![];
        let largest = self.largest_radius();

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let Some(Some(place)) = placed.get(dataset_index) else {
                return;
            };
            let (Some(x_scale), Some(y_scale)) = (scales.get(&place.options.x_scale), scales.get(&place.options.y_scale)) else {
                return;
            };

            let frame = place.frame(chart_area);
            let center = frame.index(x_scale, place.x);
            let (mut side, mut thickness) = bar_slot(center, ruler, 0, 1, get(&self.category_percentage), get(&self.bar_percentage));
            if place.polar.is_some() && half_gap > 0.0 {
                if place.x < outermost {
                    thickness -= half_gap;
                }
                if place.x > innermost {
                    side += half_gap;
                    thickness -= half_gap;
                }
            } else if touching && place.polar.is_some() && place.x < outermost {
                thickness += SEAM_OVERLAP;
            }
            if thickness <= 0.0 {
                return;
            }
            let parts = Parts::of(dataset);
            let shares = parts.shares();
            let gap = if shares.len() > 1 { self.spacing_in_frame(&frame, side + thickness / 2.0) / 2.0 } else { 0.0 };

            for (part, from, to) in shares {
                // A radius narrows the part's share of the slot: out from the ring's inner edge
                // around a circle, and about the middle of the bar along a side.
                let reach = Parts::reach(part, largest);
                if reach <= 0.0 {
                    continue;
                }
                let (part_side, part_thickness) = match frame {
                    BarFrame::Around(_) => (side, thickness * reach),
                    BarFrame::Sides { .. } => (side + thickness * (1.0 - reach) / 2.0, thickness * reach),
                };

                let (base, head) = (frame.value(y_scale, from), frame.value(y_scale, to));
                let sign = if head >= base { 1.0 } else { -1.0 };
                let (base, head) = (base + sign * gap, head - sign * gap);
                if (head - base) * sign <= 0.0 {
                    continue;
                }

                // Each part is a datum of its ring, at the ring's place on the x scale.
                let datum = Datum { index: part.index, place: Some(shapes.place(&place.options.x_scale, place.x, frame.along(center))) };
                let color = part_color(part, &place.options, dataset_index, &palette);
                let item = shapes.add(dataset_index, Some(datum), color, frame.shape(part_side, part_thickness, base, head));

                ctx.begin_path();
                item.shape.trace(ctx);
                ctx.set_fill_style(item.color);
                ctx.fill();
                // The outline is at most a quarter of the part's thickness and length, so a thin part
                // keeps its colour instead of vanishing under it.
                let length = match frame {
                    BarFrame::Around(_) => (head - base).abs() * (part_side + part_thickness / 2.0),
                    BarFrame::Sides { .. } => (head - base).abs(),
                };
                let width = get(&self.outline_width).max(0.0).min(part_thickness.min(length) / 4.0);
                if width > 0.0 {
                    ctx.set_line_width(width);
                    ctx.set_stroke_style(outline);
                    ctx.stroke();
                }
                drawn.push((dataset_index, part.index, part_side + part_thickness));
            }
        });

        self.draw_labels(ctx, scales, &placed, chart_area, &drawn);
        ctx.restore();
    }
}

impl<D: Sequence<dyn PieDataSet>> PieController<D> {
    /// The spacing between parts in `frame`'s units along the value axis: pixels along a side, and
    /// around a circle the angle the spacing spans `distance` from the centre.
    fn spacing_in_frame(&self, frame: &BarFrame, distance: Scalar) -> Scalar {
        match frame {
            BarFrame::Sides { .. } => get(&self.spacing).max(0.0),
            BarFrame::Around(_) => get(&self.spacing).max(0.0) / distance.max(1.0),
        }
    }

    /// Draws the outermost ring's labels outside the circle, each joined to the middle of its
    /// slice's outer edge by a leader line: out along the slice's middle, then level to its
    /// side's column (see `layout_labels`). Only parts whose slices were drawn (`drawn`, by
    /// dataset and point index) are labelled.
    fn draw_labels(&self, ctx: &mut CanvasContext, scales: &Scales, placed: &[Option<Placed>], chart_area: Rect, drawn: &[(usize, usize, Scalar)]) {
        let Some((outer_index, labels)) = self.outer_ring_labels(scales) else {
            return;
        };
        let Some(Some(Placed { polar: Some(polar), .. })) = placed.get(outer_index) else {
            return;
        };

        let rim = polar.polar.circle().radius;
        // A part whose slice wasn't drawn, swallowed by the spacing around it, has no label; the
        // others' lines start where their slices end.
        let measured: Vec<(Scalar, Scalar, String, Dimension, Scalar)> = labels.into_iter()
            .filter_map(|(index, angle, share, text)| {
                let (_, _, reach) = drawn.iter().find(|(dataset, part, _)| *dataset == outer_index && *part == index)?;
                let size = ctx.measure_text(&text);
                Some((angle, share, text, size, *reach))
            })
            .collect();

        let laid_out = layout_labels(polar.polar.circle().center, rim, &measured, chart_area);

        let style = axis_style(ctx.env());
        let line_color = style.tick_color(ctx.env());
        let label_color = style.label_color(ctx.env());

        ctx.begin_path();
        for label in &laid_out {
            PathSink::move_to(ctx, label.rim);
            PathSink::line_to(ctx, label.elbow);
            PathSink::line_to(ctx, label.end);
        }
        ctx.set_line_width(1.0);
        ctx.set_stroke_style(line_color);
        ctx.stroke();

        ctx.set_fill_style(label_color);
        for label in &laid_out {
            let (x, alignment) = if label.right {
                (label.end.x + LABEL_GAP, Alignment::Leading)
            } else {
                (label.end.x - LABEL_GAP, Alignment::Trailing)
            };
            ctx.set_text_align(alignment);
            ctx.fill_text(&label.text, x, label.end.y);
        }
    }
}

/// A label outside a pie, laid out: where its leader line leaves the rim, turns, and ends, and
/// which side it is on.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LaidOutLabel {
    pub(crate) text: String,
    pub(crate) rim: Position,
    pub(crate) elbow: Position,
    pub(crate) end: Position,
    pub(crate) right: bool,
}

/// Lays out labels outside a circle of `radius` around `center`, from each one's screen angle,
/// share (0 to 1), text, size, and how far out its slice reaches, within `area`. Each starts
/// where its slice's middle meets the slice's outer edge, runs out to `RADIAL_RUN` beyond the rim, then level to its side's column, `LEVEL_RUN` further out
/// than the circle's widest point, so each side's labels line up. The labels on a side are spread
/// apart, top to bottom, `LABEL_SPACING` apart at least, and pulled back up from the area's lower
/// edge; while they still don't fit in the area, the smallest slice's label on that side is left
/// out and the side laid out again.
pub(crate) fn layout_labels(center: Position, radius: Scalar, labels: &[(Scalar, Scalar, String, Dimension, Scalar)], area: Rect) -> Vec<LaidOutLabel> {
    let (screen_top, screen_bottom) = (area.top().min(area.bottom()), area.top().max(area.bottom()));
    let mut laid_out = vec![];

    for right in [true, false] {
        let side_sign = if right { 1.0 } else { -1.0 };
        let column = center.x + side_sign * (radius + RADIAL_RUN + LEVEL_RUN);

        // (index into `labels`, the y its line turns at)
        let mut side: Vec<(usize, Scalar)> = labels.iter().enumerate()
            .filter(|(_, (angle, _, _, _, _))| (direction(*angle).x >= 0.0) == right)
            .map(|(index, (angle, _, _, _, _))| {
                let out = direction(*angle);
                (index, center.y + out.y * (radius + RADIAL_RUN))
            })
            .collect();
        side.sort_by(|a, b| a.1.total_cmp(&b.1));

        let placed = loop {
            let heights: Vec<Scalar> = side.iter().map(|(index, _)| labels[*index].3.height).collect();
            let mut ys: Vec<Scalar> = side.iter().map(|(_, y)| *y).collect();

            // Down from the top, each at least a gap below the one above.
            for i in 0..ys.len() {
                let lowest = if i == 0 { screen_top + heights[0] / 2.0 } else { ys[i - 1] + heights[i - 1] / 2.0 + LABEL_SPACING + heights[i] / 2.0 };
                ys[i] = ys[i].max(lowest);
            }
            // Back up from the bottom, each at least a gap above the one below.
            for i in (0..ys.len()).rev() {
                let highest = if i + 1 == ys.len() { screen_bottom - heights[i] / 2.0 } else { ys[i + 1] - heights[i + 1] / 2.0 - LABEL_SPACING - heights[i] / 2.0 };
                ys[i] = ys[i].min(highest);
            }

            let fits = ys.first().is_none_or(|y| y - heights[0] / 2.0 >= screen_top - 0.5);
            if fits || side.is_empty() {
                break ys;
            }

            // Leave out the smallest slice's label, and lay the side out again.
            let smallest = (0..side.len()).min_by(|a, b| labels[side[*a].0].1.total_cmp(&labels[side[*b].0].1)).expect("not empty");
            side.remove(smallest);
        };

        for ((index, _), y) in side.iter().zip(placed) {
            let (angle, _, text, _, reach) = &labels[*index];
            let out = direction(*angle);
            laid_out.push(LaidOutLabel {
                text: text.clone(),
                rim: Position::new(center.x + out.x * reach, center.y + out.y * reach),
                elbow: Position::new(center.x + out.x * (radius + RADIAL_RUN), center.y + out.y * (radius + RADIAL_RUN)),
                end: Position::new(column, y),
                right,
            });
        }
    }

    laid_out
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};
    use carbide::draw::{Dimension, Position, Rect};
    use super::{layout_labels, Parts, LABEL_SPACING, LEVEL_RUN, RADIAL_RUN};

    /// A part is its absolute value's share of the total, and a missing value has none.
    #[test]
    fn parts_share_the_absolute_total() {
        let values: Vec<Option<f64>> = vec![Some(30.0), None, Some(-10.0), Some(60.0)];
        let parts = Parts::of(&values);

        assert_eq!(parts.total, 100.0);
        let shares: Vec<(usize, f64, f64)> = parts.shares().into_iter().map(|(part, from, to)| (part.index, from, to)).collect();
        assert_eq!(shares, vec![(0, 0.0, 30.0), (2, 30.0, 40.0), (3, 40.0, 100.0)]);

        let zeros = vec![0.0, 0.0];
        assert!(Parts::of(&zeros).shares().is_empty(), "nothing to share");
    }

    /// Text parts are named by their x; others by their index.
    #[test]
    fn parts_are_named_by_their_text() {
        let named = vec![("Red", 2.0), ("Blue", 3.0)];
        assert_eq!(Parts::of(&named).parts.iter().map(|part| part.name.as_str()).collect::<Vec<_>>(), vec!["Red", "Blue"]);

        let plain = vec![2.0, 3.0];
        assert_eq!(Parts::of(&plain).parts.iter().map(|part| part.name.as_str()).collect::<Vec<_>>(), vec!["0", "1"]);
    }

    /// Labels go to the column on their slice's side, their lines turning `RADIAL_RUN` out; labels
    /// that would overlap are spread apart; and when a side runs out of room, the smallest slices'
    /// labels are left out.
    #[test]
    fn labels_line_up_spread_apart_and_give_way() {
        let center = Position::new(100.0, 100.0);
        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(200.0, 200.0));
        let size = Dimension::new(20.0, 10.0);

        let two = layout_labels(center, 50.0, &[(FRAC_PI_2, 0.5, "right".into(), size, 50.0), (-FRAC_PI_2, 0.5, "left".into(), size, 50.0)], area);
        let right = two.iter().find(|label| label.right).unwrap();
        assert_eq!(right.end, Position::new(100.0 + 50.0 + RADIAL_RUN + LEVEL_RUN, 100.0), "level with its slice, in the right column");
        assert_eq!(right.elbow, Position::new(100.0 + 50.0 + RADIAL_RUN, 100.0));
        assert!(two.iter().any(|label| !label.right && label.end.x < 100.0));

        // Two slices whose middles are close: their labels are spread a gap apart.
        let close = layout_labels(center, 50.0, &[(PI / 2.0, 0.3, "a".into(), size, 50.0), (PI / 2.0 + 0.05, 0.3, "b".into(), size, 50.0)], area);
        let gap = (close[0].end.y - close[1].end.y).abs();
        assert!(gap >= 10.0 + LABEL_SPACING - 1e-9, "{}", gap);

        // Twenty labels on one side don't fit in 200 pixels: the smallest give way.
        let many: Vec<(f64, f64, String, Dimension, f64)> = (0..20).map(|i| (0.1 + i as f64 * 0.14, i as f64, format!("{i}"), size, 50.0)).collect();
        let fitted = layout_labels(center, 50.0, &many, area);
        assert!(fitted.len() < 20 && fitted.len() >= 10, "{}", fitted.len());
        assert!(!fitted.iter().any(|label| label.text == "0"), "the smallest slice gave way first");
    }
}
