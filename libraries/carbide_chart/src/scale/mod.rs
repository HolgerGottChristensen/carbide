pub(crate) mod axis;
mod axis_options;
mod category_scale;
mod linear_scale;
mod scales;
mod scale_id;

use std::fmt::Debug;
use dyn_clone::DynClone;
use std::ops::Range;
use carbide_core::draw::Scalar;
pub use axis_options::AxisOptions;
pub use category_scale::CategoryScale;
pub use linear_scale::*;
pub use scales::*;
pub use scale_id::ScaleId;

/// A scale maps data values to positions along an axis, and picks the ticks along it. It is pure
/// maths: measuring and drawing the axis is done once for every scale type (see `scale/axis.rs`),
/// from the scale and its `AxisOptions`. Charts own their scales as `Box<dyn Scale>`, so different
/// kinds of scales can sit side by side.
pub trait Scale: DynClone + Debug + 'static {
    /// The direction the scale runs in.
    fn axis(&self) -> Axis;

    /// Where and how the scale's axis is laid out and drawn: its side, its weight, and whether it
    /// draws grid lines and tick marks.
    fn axis_options(&self) -> &AxisOptions;

    /// Where `value` falls along the scale, from 0 at the start of the chart area to 1 at its
    /// end. Controllers and the grid map values to positions through this, so a scale's offset
    /// (see `set_offset`) applies to everything drawn on it.
    fn normalize(&self, value: Scalar) -> Scalar {
        let range = self.range();
        (value - range.start) / (range.end - range.start)
    }

    /// Pads the mapping by half a tick step at both ends, so values on the first and last tick sit
    /// inside the chart area instead of on its edges. The chart sets it on every fit, for the
    /// scales a controller requested it on (see `ScaleBounds::request_offset`); bar charts use it
    /// so their first and last bars aren't cut in half. Scales that don't support it ignore it.
    fn set_offset(&mut self, _offset: bool) {}

    /// Whether a controller asked for grid lines between ticks instead of on them, set by the
    /// chart on every fit (see `ScaleBounds::request_grid_offset`). Bar charts ask for it so grid
    /// lines separate the bar groups. A scale whose grid offset was set explicitly keeps that
    /// setting; scales that don't support it ignore it.
    fn set_grid_offset_requested(&mut self, _requested: bool) {}

    /// The scale's own text for the tick at `value`, such as a category's name. `None`, the
    /// default, shows the number, formatted for the environment's locale.
    fn label(&self, _value: Scalar) -> Option<String> {
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
    Radial,
}

impl Axis {
    /// Where a scale on this axis sits unless told otherwise: below the chart area for
    /// horizontal scales, left of it for the others.
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

/// The side of the chart area a scale is drawn on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AxisPosition {
    Left,
    Right,
    Top,
    Bottom,
}
