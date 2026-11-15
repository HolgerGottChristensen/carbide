mod dataset;
mod datapoint;
mod datavalue;
mod dataset_sequence;
mod dataset_options;
mod dataset_ext;
mod options_dataset;
mod stack_id;

use carbide::draw::Color;
use carbide::impl_state_value;
pub use datapoint::*;
pub use dataset::*;
pub use datavalue::*;
pub use dataset_sequence::*;
pub use stack_id::StackId;
pub use dataset_options::*;
pub use dataset_ext::*;

/// The colour of a dataset, set with `DatasetExt::color`, or of a single point
/// (`DataPoint::color`), which overrides its dataset's.
#[derive(Debug, Copy, Clone)]
pub enum DataColor {
    /// The palette colour for the dataset's index in its controller. The default.
    ByDataSet,
    /// This colour.
    Color(Color),
    /// A palette colour per point, by the point's index in its dataset, so each bar or point of
    /// the dataset gets the next colour, and the same index always gets the same colour. A
    /// line's stroke and its filled area, which are one shape for the whole dataset, keep the
    /// palette colour for the dataset's index, as with `Inherit`.
    ByPoint,
}

impl From<Color> for DataColor {
    fn from(value: Color) -> Self {
        DataColor::Color(value)
    }
}

impl_state_value!(DataColor);