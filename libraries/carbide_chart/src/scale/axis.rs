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
use crate::scale::{Axis, AxisPosition, Scale};
use crate::style::axis_style;

/// How far tick marks reach out from the inner edge of their scale's band.
const TICK_LENGTH: Scalar = 10.0;

/// The gap between an axis's labels and its title.
const TITLE_GAP: Scalar = 4.0;

/// The text shown for the tick at `value`: the scale's own label for it if it has one (a category
/// scale's names), otherwise the number, formatted for the environment's locale.
pub(crate) fn tick_label(scale: &dyn Scale, value: Scalar, env: &mut Environment) -> String {
    scale.label(value).unwrap_or_else(|| format_number(value, env))
}

/// How far the scale's band reaches out from the chart area: its tick marks plus its widest tick
/// label for a vertical scale, or its tallest for a horizontal one, plus its title's height (a
/// vertical scale's title is turned along the axis, so its height also lies across it). Scales on
/// the same side are stacked outwards, each in its own band.
pub(crate) fn size(scale: &dyn Scale, ctx: &mut CanvasContext) -> Scalar {
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
            Axis::Vertical | Axis::Radial => size.width,
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
