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
mod label_rotation;
mod rim_orientation;
pub(crate) mod tick_labels;

use std::fmt::Debug;
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
pub use label_rotation::LabelRotation;
pub use rim_orientation::RimOrientation;
pub(crate) use zoom::MIN_ZOOM_SPAN;
pub(crate) use coordinates::Places;

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
