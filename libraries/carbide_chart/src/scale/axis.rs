//! The axis of a scale: measuring its band and drawing its grid lines, tick marks, labels and
//! border. Written once for every scale type, from what the scale maps (`Scale`), how its axis is
//! laid out (`AxisOptions`), and the colours of the chart's `AxisStyle`.
//!
//! In the drawing functions, `offset` is the distance from the chart area to the inner edge of the
//! scale's band (0 for the scale closest to the area). Grid lines span the chart area; tick marks,
//! labels and the border sit in the band.
//!
//! `Rect`'s y grows downwards on screen, so `area.bottom()` is the screen's top edge of the area
//! and `area.top()` its bottom edge.

use carbide::draw::{Alignment, Angle, Position, Rect, Scalar};
use carbide::math::Vector3;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::scale::{Axis, AxisPosition, GridShape, PolarRole, Scale, Scales};
use crate::scale::polar::{self, Circle, Polar, RimLabel, Ring};
use crate::style::axis_style;

/// How far tick marks reach out from the inner edge of their scale's band.
const TICK_LENGTH: Scalar = 10.0;

/// The gap between an axis's labels and its title.
const TITLE_GAP: Scalar = 4.0;

/// The text shown for the tick at `value`: the scale's own label for it if it has one (a category
/// scale's names), otherwise the number, formatted for the environment's locale.
pub(crate) fn tick_label(scale: &dyn Scale, value: Scalar, env: &mut Environment) -> String {
    scale.label(value, env).unwrap_or_else(|| format_number(value, env))
}

/// How far the scale's band reaches out from the chart area: its tick marks plus its widest tick
/// label for a vertical scale, or its tallest for a horizontal one, plus its title's height (a
/// vertical scale's title is turned along the axis, so its height also lies across it). Scales on
/// the same side are stacked outwards, each in its own band.
pub(crate) fn size(scale: &dyn Scale, ctx: &mut CanvasContext) -> Scalar {
    // Polar scales take no band: their labels sit around and inside the circle.
    if scale.axis() == Axis::Radial {
        return 0.0;
    }

    let tick_length = if scale.axis_options().tick_marks() { TICK_LENGTH } else { 0.0 };

    let mut label_extent: Scalar = 0.0;
    for tick in scale.ticks() {
        let label = tick_label(scale, *tick, ctx.env());
        let size = ctx.measure_text(&label);

        label_extent = label_extent.max(match scale.axis() {
            Axis::Horizontal => size.height,
            _ => size.width,
        });
    }

    let title_extent = match (scale.axis(), scale.axis_options().title()) {
        (Axis::Horizontal | Axis::Vertical, Some(title)) => TITLE_GAP + ctx.measure_text(&title).height,
        _ => 0.0,
    };

    tick_length + label_extent + title_extent
}

/// Draws the scale's title, in the `AxisStyle`'s label colour, centred along the chart area at the
/// outer edge of its band, `size` (see `size`) from its inner edge. A vertical scale's title is
/// turned along the axis: on the left it reads from bottom to top, on the right from top to
/// bottom, so the top of the text faces the chart area on both sides.
pub(crate) fn draw_title(scale: &dyn Scale, ctx: &mut CanvasContext, area: Rect, offset: Scalar, size: Scalar) {
    let Some(title) = scale.axis_options().title() else {
        return;
    };

    let height = ctx.measure_text(&title).height;
    let middle_x = area.left() + area.width() / 2.0;
    let middle_y = area.bottom() + area.height() / 2.0;

    // Where the middle of the title goes, and how far it is turned.
    let (x, y, degrees) = match (scale.axis(), scale.axis_options().position()) {
        (Axis::Horizontal, AxisPosition::Top) => (middle_x, area.bottom() - offset - size + height / 2.0, 0.0),
        (Axis::Horizontal, _) => (middle_x, area.top() + offset + size - height / 2.0, 0.0),
        (Axis::Vertical, AxisPosition::Right) => (area.right() + offset + size - height / 2.0, middle_y, 90.0),
        (Axis::Vertical, _) => (area.left() - offset - size + height / 2.0, middle_y, -90.0),
        (Axis::Radial, _) => return,
    };

    let style = axis_style(ctx.env());
    let color = style.label_color(ctx.env());

    ctx.save();
    ctx.translate(Vector3::new(x, y, 0.0));
    ctx.rotate(Angle::Degrees(degrees));
    ctx.set_fill_style(color);
    ctx.set_text_align(Alignment::Center);
    ctx.fill_text(&title, 0.0, 0.0);
    ctx.restore();
}

/// Draws the scale's grid lines across the chart area (if it has grid lines), its tick marks in
/// its band (if it has tick marks), and its tick labels, in the chart's `AxisStyle`. Turning the grid lines off keeps the
/// labels, which is what secondary scales usually want.
pub(crate) fn draw_grid(scale: &dyn Scale, ctx: &mut CanvasContext, area: Rect, offset: Scalar) {
    let options = scale.axis_options();

    // Per side: the chart area's edge the band starts at, which way is outwards across the axis
    // (+1 or -1), the opposite edge the grid lines run to, and how labels hang off the end of the
    // tick marks.
    let (horizontal, area_edge, outward, far_edge, alignment) = match (scale.axis(), options.position()) {
        (Axis::Horizontal, AxisPosition::Top) => (true, area.bottom(), -1.0, area.top(), Alignment::Bottom),
        (Axis::Horizontal, _) => (true, area.top(), 1.0, area.bottom(), Alignment::Top),
        (Axis::Vertical, AxisPosition::Right) => (false, area.right(), 1.0, area.left(), Alignment::Leading),
        (Axis::Vertical, _) => (false, area.left(), -1.0, area.right(), Alignment::Trailing),
        (Axis::Radial, _) => return,
    };

    let grid_lines = options.grid_lines();
    let tick_marks = options.tick_marks();
    let tick_length = if tick_marks { TICK_LENGTH } else { 0.0 };

    let band_edge = area_edge + outward * offset;
    let tick_end = band_edge + outward * tick_length;

    let style = axis_style(ctx.env());
    let grid_color = style.grid_color(ctx.env());
    let minor_grid_color = style.minor_grid_color(ctx.env());
    let tick_color = style.tick_color(ctx.env());
    let label_color = style.label_color(ctx.env());

    ctx.save();
    ctx.set_line_width(1.0);
    ctx.set_text_align(alignment);

    // `along` is a value's coordinate along the axis; `across` a coordinate across it.
    let along = |value: Scalar| {
        let fraction = scale.normalize(value);

        if horizontal {
            area.left() + fraction * area.width()
        } else {
            area.bottom() + (1.0 - fraction) * area.height()
        }
    };
    let point = |along: Scalar, across: Scalar| if horizontal { (along, across) } else { (across, along) };

    // The values grid lines and tick marks are drawn at, skipping those that would fall outside
    // the chart area.
    let lines: Vec<Scalar> = scale.grid_line_values()
        .into_iter()
        .filter(|value| (-1e-9..=1.0 + 1e-9).contains(&scale.normalize(*value)))
        .map(along)
        .collect();

    // Minor ticks get a fainter grid line and a tick mark half as long, without a label, under the
    // major ones.
    let minor: Vec<Scalar> = scale.minor_ticks()
        .iter()
        .filter(|value| (-1e-9..=1.0 + 1e-9).contains(&scale.normalize(**value)))
        .map(|value| along(*value))
        .collect();

    if grid_lines && !minor.is_empty() {
        ctx.begin_path();
        for along in &minor {
            let (x, y) = point(*along, area_edge);
            ctx.move_to(x, y);
            let (x, y) = point(*along, far_edge);
            ctx.line_to(x, y);
        }
        ctx.set_stroke_style(minor_grid_color);
        ctx.stroke();
    }

    if tick_marks && !minor.is_empty() {
        let minor_end = band_edge + outward * tick_length / 2.0;
        ctx.begin_path();
        for along in &minor {
            let (x, y) = point(*along, band_edge);
            ctx.move_to(x, y);
            let (x, y) = point(*along, minor_end);
            ctx.line_to(x, y);
        }
        ctx.set_stroke_style(tick_color);
        ctx.stroke();
    }

    if grid_lines {
        ctx.begin_path();
        for along in &lines {
            let (x, y) = point(*along, area_edge);
            ctx.move_to(x, y);
            let (x, y) = point(*along, far_edge);
            ctx.line_to(x, y);
        }
        ctx.set_stroke_style(grid_color);
        ctx.stroke();
    }

    if tick_marks {
        ctx.begin_path();
        for along in &lines {
            let (x, y) = point(*along, band_edge);
            ctx.move_to(x, y);
            let (x, y) = point(*along, tick_end);
            ctx.line_to(x, y);
        }
        ctx.set_stroke_style(tick_color);
        ctx.stroke();
    }

    ctx.set_fill_style(label_color);

    for tick in scale.ticks() {
        let label = &tick_label(scale, *tick, ctx.env());
        let (x, y) = point(along(*tick), tick_end);
        ctx.fill_text(label, x, y);
    }

    ctx.restore();
}

/// Draws the scale's border: a line along the inner edge of its band, in the chart's
/// `AxisStyle`.
pub(crate) fn draw_border(scale: &dyn Scale, ctx: &mut CanvasContext, area: Rect, offset: Scalar) {
    let (from, to) = match (scale.axis(), scale.axis_options().position()) {
        (Axis::Horizontal, AxisPosition::Top) => {
            (Position::new(area.left(), area.bottom() - offset), Position::new(area.right(), area.bottom() - offset))
        }
        (Axis::Horizontal, _) => {
            (Position::new(area.left(), area.top() + offset), Position::new(area.right(), area.top() + offset))
        }
        (Axis::Vertical, AxisPosition::Right) => {
            (Position::new(area.right() + offset, area.bottom()), Position::new(area.right() + offset, area.top()))
        }
        (Axis::Vertical, _) => {
            (Position::new(area.left() - offset, area.bottom()), Position::new(area.left() - offset, area.top()))
        }
        (Axis::Radial, _) => return,
    };

    let style = axis_style(ctx.env());
    let color = style.border_color(scale.axis(), ctx.env());

    ctx.save();

    ctx.begin_path();
    ctx.set_line_width(style.border_width());
    ctx.set_stroke_style(color);
    ctx.move_to(from.x, from.y);
    ctx.line_to(to.x, to.y);
    ctx.stroke();

    ctx.restore();
}

/// The labels of `angle_scale`'s ticks around the rim, with their screen angles and sizes, for
/// fitting the circle (see `polar::fit_circle`): every tick within the sweep, except one that
/// lands where an earlier one did, as the end of a full circle lands on its start.
pub(crate) fn rim_labels(angle_scale: &dyn Scale, ctx: &mut CanvasContext) -> Vec<(String, RimLabel)> {
    let (start, sweep) = polar::angle_range(angle_scale);
    let mut labels: Vec<(String, RimLabel)> = vec![];

    for tick in angle_scale.ticks() {
        let normalized = angle_scale.normalize(*tick);
        if !(-1e-9..=1.0 + 1e-9).contains(&normalized) {
            continue;
        }

        let angle = start + normalized * sweep;
        if labels.iter().any(|(_, label)| polar::same_direction(label.angle, angle)) {
            continue;
        }

        let text = tick_label(angle_scale, *tick, ctx.env());
        let size = ctx.measure_text(&text);
        labels.push((text, RimLabel { angle, size }));
    }

    labels
}

/// The circle for the chart's polar scales inside `area`, made room for the first angle scale's
/// labels around the rim, or `None` when the chart has no polar scales.
pub(crate) fn polar_circle(scales: &Scales, ctx: &mut CanvasContext, area: Rect) -> Option<Circle> {
    let (angle_scale, radius_scale) = scales.first_polar_pair();
    if angle_scale.is_none() && radius_scale.is_none() {
        return None;
    }

    let (start, sweep) = angle_scale.map(polar::angle_range).unwrap_or((0.0, std::f64::consts::TAU));
    let inner = radius_scale.map(|scale| scale.axis_options().inner_radius()).unwrap_or(0.0);
    let labels: Vec<RimLabel> = angle_scale
        .map(|scale| rim_labels(scale, ctx).into_iter().map(|(_, label)| label).collect())
        .unwrap_or_default();

    Some(polar::fit_circle(area, start, sweep, inner, &labels))
}

/// Draws the grid of the chart's polar scales on its circle, in the chart's `AxisStyle`: a ring
/// at each of each radius scale's grid lines (and fainter ones at its minor ticks), as circles,
/// or polygons with a corner at each of the angle scale's ticks (see `Scales::grid_shape`); a
/// spoke from the inner radius to the rim at each of the first angle scale's grid lines; the
/// angle scale's labels around the rim; and each radius scale's labels along the start spoke.
pub(crate) fn draw_polar_grid(scales: &Scales, ctx: &mut CanvasContext) {
    let (Some(angle_scale), Some(circle)) = (scales.first_polar_pair().0, scales.circle()) else {
        return;
    };

    let style = axis_style(ctx.env());
    let grid_color = style.grid_color(ctx.env());
    let minor_grid_color = style.minor_grid_color(ctx.env());
    let label_color = style.label_color(ctx.env());

    ctx.save();
    ctx.set_line_width(1.0);

    let corners = angle_corners(angle_scale);

    for (id, radius_scale) in scales.with_role(PolarRole::Radius) {
        if !radius_scale.axis_options().grid_lines() {
            continue;
        }

        let polar = Polar::new(angle_scale, radius_scale, circle);
        let polygon = (scales.grid_shape(id) == GridShape::Polygon && corners.len() >= 3).then_some(corners.as_slice());

        for (values, color) in [(radius_scale.minor_ticks().to_vec(), minor_grid_color), (radius_scale.grid_line_values(), grid_color)] {
            ctx.begin_path();
            for value in values {
                let normalized = radius_scale.normalize(value);
                if !(-1e-9..=1.0 + 1e-9).contains(&normalized) {
                    continue;
                }

                let distance = polar.distance(normalized);
                if distance > 0.5 {
                    trace_ring(ctx, &polar, distance, polygon);
                }
            }
            ctx.set_stroke_style(color);
            ctx.stroke();
        }
    }

    let radius_scale = scales.first_polar_pair().1;
    let polar = radius_scale.map(|radius_scale| Polar::new(angle_scale, radius_scale, circle));

    if let Some(polar) = polar && angle_scale.axis_options().grid_lines() {
        ctx.begin_path();
        let mut drawn: Vec<Scalar> = vec![];
        for value in angle_scale.grid_line_values() {
            let normalized = angle_scale.normalize(value);
            if !(-1e-9..=1.0 + 1e-9).contains(&normalized) {
                continue;
            }

            let angle = polar.angle(normalized);
            if drawn.iter().any(|earlier| polar::same_direction(*earlier, angle)) {
                continue;
            }
            drawn.push(angle);

            let from = polar.at(angle, polar.inner_radius());
            let to = polar.at(angle, circle.radius);
            ctx.move_to(from.x, from.y);
            ctx.line_to(to.x, to.y);
        }
        ctx.set_stroke_style(grid_color);
        ctx.stroke();
    }

    ctx.set_fill_style(label_color);

    let rim_labels = rim_labels(angle_scale, ctx);
    // A label around the rim at the start spoke takes the place of a number at the rim; a radius
    // scale's categories are all kept.
    let (start, _) = polar::angle_range(angle_scale);
    let rim_label_on_start = rim_labels.iter().any(|(_, label)| polar::same_direction(label.angle, start));

    for (text, label) in rim_labels {
        let direction = polar::direction(label.angle);
        let anchor = Position::new(
            circle.center.x + (circle.radius + polar::RIM_LABEL_GAP) * direction.x,
            circle.center.y + (circle.radius + polar::RIM_LABEL_GAP) * direction.y,
        );

        ctx.set_text_align(rim_alignment(direction));
        ctx.fill_text(&text, anchor.x, anchor.y);
    }

    // Each radius scale's labels sit beside the start spoke, on its clockwise side: to the right of
    // a spoke pointing up, above one pointing left.
    for (_, radius_scale) in scales.with_role(PolarRole::Radius) {
        let polar = Polar::new(angle_scale, radius_scale, circle);
        let (start, _) = polar.angle_range();
        let side = Position::new(start.cos(), start.sin());
        let beside = Position::new(side.x * RADIUS_LABEL_GAP, side.y * RADIUS_LABEL_GAP);
        ctx.set_text_align(rim_alignment(side));

        for tick in radius_scale.ticks() {
            let normalized = radius_scale.normalize(*tick);
            let text = tick_label(radius_scale, *tick, ctx.env());
            let is_category = radius_scale.category_index(&text) == Some(*tick);
            if !(-1e-9..=1.0 + 1e-9).contains(&normalized) || (rim_label_on_start && !is_category && normalized > 1.0 - 1e-9) {
                continue;
            }

            let at = polar.at(start, polar.distance(normalized));
            ctx.fill_text(&text, at.x + beside.x, at.y + beside.y);
        }
    }

    ctx.restore();
}

/// Draws the border of the chart's polar scales: the rim, in the shape of the first radius
/// scale's rings, and, on a partial circle, the edges of the sector: the inner ring and the
/// spokes at both ends.
pub(crate) fn draw_polar_border(scales: &Scales, ctx: &mut CanvasContext) {
    let (Some(angle_scale), Some(radius_scale)) = scales.first_polar_pair() else {
        return;
    };
    let Some(circle) = scales.circle() else {
        return;
    };

    let polar = Polar::new(angle_scale, radius_scale, circle);
    let corners = angle_corners(angle_scale);
    let radius_id = scales.with_role(PolarRole::Radius).next().map(|(id, _)| id.clone());
    let polygon = (radius_id.is_some_and(|id| scales.grid_shape(&id) == GridShape::Polygon) && corners.len() >= 3).then_some(corners.as_slice());

    let style = axis_style(ctx.env());

    ctx.save();
    ctx.begin_path();
    ctx.set_line_width(style.border_width());
    let color = style.border_color(Axis::Radial, ctx.env());
    ctx.set_stroke_style(color);

    trace_ring(ctx, &polar, circle.radius, polygon);

    if !polar.is_full_circle() {
        let (start, end) = polar.angle_range();
        if polar.inner_radius() > 0.5 {
            trace_ring(ctx, &polar, polar.inner_radius(), polygon);
        }

        for angle in [start, end] {
            let from = polar.at(angle, polar.inner_radius());
            let to = polar.at(angle, circle.radius);
            ctx.move_to(from.x, from.y);
            ctx.line_to(to.x, to.y);
        }
    }

    ctx.stroke();
    ctx.restore();
}

/// The gap between the start spoke and the radius scale's labels beside it.
const RADIUS_LABEL_GAP: Scalar = 4.0;

/// The screen angles of `angle_scale`'s ticks within its sweep, each direction once: the corners
/// of polygon rings.
pub(crate) fn angle_corners(angle_scale: &dyn Scale) -> Vec<Scalar> {
    let (start, sweep) = polar::angle_range(angle_scale);
    let mut corners: Vec<Scalar> = vec![];

    for tick in angle_scale.ticks() {
        let normalized = angle_scale.normalize(*tick);
        if !(-1e-9..=1.0 + 1e-9).contains(&normalized) {
            continue;
        }

        let angle = start + normalized * sweep;
        if !corners.iter().any(|corner| polar::same_direction(*corner, angle)) {
            corners.push(angle);
        }
    }

    corners
}

/// Adds a ring `distance` from the centre to the path, as a shape of its own: a circle, or the
/// arc of a partial sweep, or, with `corners`, a polygon through those screen angles, closed on a
/// full circle.
fn trace_ring(path: &mut CanvasContext, polar: &Polar, distance: Scalar, corners: Option<&[Scalar]>) {
    Ring { polar, distance, corners }.trace_whole(path, false);

    if polar.is_full_circle() {
        path.close_path();
    }
}

/// How a label around the rim hangs off its anchor, `direction` being the unit direction from
/// the centre: away from the circle, so its nearest edge or corner faces the rim.
fn rim_alignment(direction: Position) -> Alignment {
    let horizontal = if direction.x > 0.2 { 1 } else if direction.x < -0.2 { -1 } else { 0 };
    let vertical = if direction.y > 0.2 { 1 } else if direction.y < -0.2 { -1 } else { 0 };

    // The alignment names the point of the text box at the anchor: a label below the circle
    // hangs from its top edge, one to the right starts at its leading edge.
    match (horizontal, vertical) {
        (0, -1) => Alignment::Bottom,
        (0, 1) => Alignment::Top,
        (1, 0) => Alignment::Leading,
        (-1, 0) => Alignment::Trailing,
        (1, -1) => Alignment::BottomLeading,
        (-1, -1) => Alignment::BottomTrailing,
        (1, 1) => Alignment::TopLeading,
        (-1, 1) => Alignment::TopTrailing,
        _ => Alignment::Center,
    }
}

#[cfg(feature = "carbide_fluent")]
fn format_number(x: Scalar, env: &mut Environment) -> String {
    use carbide::state::{ReadState, StateSync};
    use carbide_fluent::LocalizedNumber;

    let mut number = LocalizedNumber::new(x);
    number.sync(env);

    number.value().to_string()
}

#[cfg(not(feature = "carbide_fluent"))]
fn format_number(x: Scalar, _env: &mut Environment) -> String {
    format!("{}", x)
}
