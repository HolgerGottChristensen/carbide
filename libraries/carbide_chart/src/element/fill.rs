use carbide::draw::Scalar;
use carbide::impl_state_value;

/// What the area under (or over) a dataset's line is filled to, set with `DatasetExt::fill`.
/// The area follows the line, smooth, monotone or stepped as its `Interpolation`, and is drawn
/// in the dataset's colour, translucent, under the lines and points of its controller.
///
/// An area assumes the points are in x order, as `Interpolation::Monotone` does: it follows the
/// line in data order and closes along the baseline from the last point back to the first. A
/// line that doubles back, as a scatter chart's can, crosses its own area and fills into
/// overlapping, meaningless shapes.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum Fill {
    /// Not filled. The default.
    #[default]
    None,
    /// To zero. Zero is included in the y bounds, so the area always has its base on the chart,
    /// and parts of the line below zero are filled upwards to it.
    Origin,
    /// To the bottom edge of the chart area.
    Bottom,
    /// To the top edge of the chart area.
    Top,
    /// To the value. It is included in the y bounds, as zero is for `Origin`.
    Value(Scalar),
    /// Down to the line of the dataset below it in its stack (see `DatasetExt::stack`), so a
    /// stack of lines becomes a stacked area chart. The bottom dataset of a stack, a dataset
    /// without a stack, and any dataset in a controller that doesn't stack fill to zero, as
    /// `Origin` does.
    Stack,
}

impl Fill {
    /// The value the area is filled to, when it is one: it is included in the y bounds.
    pub(crate) fn baseline_value(&self) -> Option<Scalar> {
        match self {
            Fill::Origin | Fill::Stack => Some(0.0),
            Fill::Value(value) => Some(*value),
            Fill::None | Fill::Bottom | Fill::Top => None,
        }
    }
}

impl_state_value!(Fill);
