pub(crate) mod axis;
mod axis_options;
mod category_scale;
mod linear_scale;
mod time_scale;
mod log_scale;
mod scales;
mod scale_id;
mod coordinates;
pub(crate) mod polar;
mod zoom;

use std::fmt::Debug;
use dyn_clone::DynClone;
use std::ops::Range;
use carbide::environment::Environment;
use carbide_core::draw::Scalar;
use carbide_derive::StateValue;
pub use axis_options::AxisOptions;
pub use category_scale::CategoryScale;
pub use linear_scale::*;
pub use log_scale::LogScale;
pub use time_scale::{TimeScale, TimeScaleZone, TimeUnit};
pub use scales::*;
pub use scale_id::ScaleId;
pub use coordinates::Coordinates;
pub use polar::{Circle, Polar};
pub use zoom::{SnapMode, ZoomWindow};
pub(crate) use zoom::MIN_ZOOM_SPAN;
pub(crate) use coordinates::Places;

/// A scale maps data values to positions along an axis, and picks the ticks along it. It is pure
/// maths: measuring and drawing the axis is done once for every scale type (see `scale/axis.rs`),
/// from the scale and its `AxisOptions`. Charts own their scales as `Box<dyn Scale>`, so different
/// kinds of scales can sit side by side.
pub trait Scale: DynClone + Debug + 'static {
    /// The direction the scale runs in.
    fn axis(&self) -> Axis;

    /// Where and how the scale's axis is laid out and drawn: its side, its weight, whether it
    /// draws grid lines and tick marks, and its title.
    fn axis_options(&self) -> &AxisOptions;

    /// The options, to change: the chart's gestures set a zoomable scale's zoom through them (see
    /// `AxisOptions::set_zoom`). `None`, the default, for a scale whose options can't change, which
    /// gestures leave alone.
    fn axis_options_mut(&mut self) -> Option<&mut AxisOptions> {
        None
    }

    /// Whether normalized positions run against the values, so the first value is at the end of
    /// the chart area, as on a vertical category scale, whose first category is at the top. The
    /// chart's gestures read the pointer's place along the scale by it. `false` by default.
    fn is_reversed(&self) -> bool {
        false
    }

    /// Syncs the states in the scale with the environment, such as its axis title (see
    /// `AxisOptions::sync`), and returns whether anything may have changed. The chart calls it
    /// before every update, render and event. The default syncs nothing: a scale of your own
    /// with states syncs them here.
    fn sync(&mut self, _env: &mut Environment) -> bool {
        false
    }

    /// Where `value` falls along the scale, from 0 at the start of the chart area to 1 at its
    /// end. Controllers and the grid map values to positions through this, so a scale's offset
    /// (see `set_offset`) applies to everything drawn on it.
    fn normalize(&self, value: Scalar) -> Scalar {
        let range = self.range();
        (value - range.start) / (range.end - range.start)
    }

    /// The value at `normalized` along the scale: the inverse of `normalize`, so a position in the
    /// chart area maps back to the value drawn there. Positions outside 0 to 1 give values past
    /// the scale's ends. A scale that overrides `normalize` overrides this too.
    fn denormalize(&self, normalized: Scalar) -> Scalar {
        let range = self.range();
        range.start + normalized * (range.end - range.start)
    }

    /// The category at `value`, a position on the scale such as `denormalize` gives: its index,
    /// as a value on the scale (see `label` for its name), and how far across the category's band
    /// `value` is, from 0 at the band's start to 1 at its end, in the order the values run. Each
    /// category's band is one wide, centred on its index. `None`, the default, for scales without
    /// categories.
    fn category_at(&self, _value: Scalar) -> Option<(Scalar, Scalar)> {
        None
    }

    /// Pads the mapping by half a tick step at both ends, so values on the first and last tick sit
    /// inside the chart area instead of on its edges. The chart sets it on every fit, for the
    /// scales a controller requested it on (see `ScaleRequests::request_offset`); bar charts use it
    /// so their first and last bars aren't cut in half. Scales that don't support it ignore it.
    fn set_offset(&mut self, _offset: bool) {}

    /// Whether a controller asked for grid lines between ticks instead of on them, set by the
    /// chart on every fit (see `ScaleRequests::request_grid_offset`). Bar charts ask for it so grid
    /// lines separate the bar groups. A scale whose grid offset was set explicitly keeps that
    /// setting; scales that don't support it ignore it.
    fn set_grid_offset_requested(&mut self, _requested: bool) {}

    /// The scale's own text for the tick at `value`, such as a category's name or a date, which
    /// it can format for the locale in `env`. `None`, the default, shows the number, formatted for
    /// the environment's locale.
    fn label(&self, _value: Scalar, _env: &mut Environment) -> Option<String> {
        None
    }

    /// The scale's own categories, in index order, which come before any the data adds: a
    /// category scale's labels. The chart registers them before collecting the bounds, so data
    /// naming them is placed on them. The default has none.
    fn categories(&self, _f: &mut dyn FnMut(&str)) {}

    /// Gives the scale its categories on a fit, in index order: its own, then the ones the data
    /// named, in the order they were first seen (see `ScaleBounds::register_category`). Scales
    /// without categories ignore them.
    fn set_categories(&mut self, _categories: &[String]) {}

    /// Where the scale places the number `value`, or `None` when it has no place for it, as a log
    /// scale has none at or below zero. Points whose x or y has no place aren't drawn (see
    /// `Coordinates`). The default places every value at itself.
    fn place(&self, value: Scalar) -> Option<Scalar> {
        Some(value)
    }

    /// Gives the scale the smallest positive end of its data on a fit, before `set_range` (see
    /// `DataBounds::smallest_positive`), for scales without a place for zero and below, whose
    /// range starts there. Other scales ignore it.
    fn set_smallest_positive(&mut self, _smallest: Option<Scalar>) {}

    /// The scale's minor ticks: values between its ticks that get a grid line and a short tick
    /// mark, but no label, such as 2 to 9 times each power on a log scale. Empty by default.
    fn minor_ticks(&self) -> &[Scalar] {
        &[]
    }

    /// Gives a radial scale its role on a fit, the angle or the radius, from how datasets bind
    /// it, or `None` when none does (see `PolarRole`). The built-in scales keep it in their
    /// `AxisOptions`, which space categories around a full circle and pick fewer ticks along a
    /// radius by it. Other scales ignore it.
    fn set_polar_role(&mut self, _role: Option<PolarRole>) {}

    /// Whether the scale's ticks are categories, as a category scale's are. A radius scale's
    /// rings are only drawn as polygons around an angle scale with categories (see
    /// `GridShape::Polygon`). The default is `false`.
    fn has_categories(&self) -> bool {
        false
    }

    /// The index of the category `text` on the scale, which is where a value naming it is placed
    /// (see `Value::Text`), or `None` when the scale has no such category. Scales without
    /// categories have none, so text isn't drawn on them.
    fn category_index(&self, _text: &str) -> Option<Scalar> {
        None
    }

    /// Where grid lines and tick marks are drawn: on the ticks themselves, unless the scale puts
    /// them between ticks instead (see `CategoryScale::grid_offset`). Labels are always drawn on
    /// the ticks.
    fn grid_line_values(&self) -> Vec<Scalar> {
        self.ticks().to_vec()
    }

    fn min(&self) -> Scalar;
    fn max(&self) -> Scalar;
    fn set_range(&mut self, min: Scalar, max: Scalar);
    fn recalculate_ticks(&mut self);
    fn ticks(&self) -> &[Scalar];
    fn range(&self) -> Range<Scalar>;
}

dyn_clone::clone_trait_object!(Scale);

/// The direction a scale runs in: along a side of the chart area, or around and out from the
/// centre of a polar chart.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
    /// Around or out from the centre of a polar chart, as the datasets bound to it say (see
    /// `PolarRole`): a dataset's x scale is the angle and its y scale the radius, or the other way
    /// round when its controller is `horizontal`.
    Radial,
}

impl Axis {
    /// Where a scale on this axis sits unless told otherwise: below the chart area for
    /// horizontal scales, left of it for the others. Radial scales take no side.
    pub fn default_position(&self) -> AxisPosition {
        match self {
            Axis::Horizontal => AxisPosition::Bottom,
            Axis::Vertical | Axis::Radial => AxisPosition::Left,
        }
    }

    /// Whether a scale on this axis can sit at `position`: horizontal scales above or below the
    /// chart area, vertical scales left or right of it.
    pub fn allows(&self, position: AxisPosition) -> bool {
        match self {
            Axis::Horizontal => matches!(position, AxisPosition::Top | AxisPosition::Bottom),
            Axis::Vertical => matches!(position, AxisPosition::Left | AxisPosition::Right),
            Axis::Radial => position == self.default_position(),
        }
    }
}

/// What a radial scale is in its polar chart, an angle scale or a radius scale: the angle, around the circle from its start angle
/// to its end angle (see `AxisOptions::start_angle`), or the radius, out from the inner radius to
/// the rim (see `AxisOptions::inner_radius`). The chart gives each radial scale its role on every
/// fit, from how datasets bind it (see `ScaleRequests::request_polar_role`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PolarRole {
    Angle,
    Radius,
}

/// Which way an angle scale runs around the circle.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, StateValue)]
pub enum AngleDirection {
    #[default]
    Clockwise,
    CounterClockwise,
}

/// The shape of a polar chart's rings, drawn at the radius scale's ticks.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, StateValue)]
pub enum GridShape {
    /// Circles around the centre.
    #[default]
    Circle,
    /// Polygons with a corner on each of the angle scale's categories, the radar convention. Only
    /// around an angle scale with categories (see `Scale::has_categories`) and at least three of
    /// them; otherwise the rings are circles.
    Polygon,
}

/// The side of the chart area a scale is drawn on.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, StateValue)]
pub enum AxisPosition {
    Left,
    Right,
    Top,
    Bottom,
}
