mod dataset;
mod datapoint;
mod datavalue;
mod dataset_sequence;
mod dataset_options;
mod dataset_ext;
mod options_dataset;
mod stack_id;
mod range_point;
mod sized_point;
mod box_point;
mod ohlc_point;

use carbide::draw::Color;
use carbide::impl_state_value;
pub use datapoint::*;
pub use dataset::*;
pub use datavalue::*;
pub use dataset_sequence::*;
pub use stack_id::StackId;
pub use dataset_options::*;
pub use dataset_ext::*;
pub use range_point::{Estimate, Range, RangePoint, Spread};
pub use sized_point::{Bubble, SizedPoint};
pub use box_point::{BoxPoint, BoxStats};
pub use ohlc_point::{Ohlc, OhlcPoint};
pub(crate) use options_dataset::OptionsDataset;

/// The colour of a dataset, set with `DatasetExt::color`, or of a single point
/// (`DataPoint::color`), which overrides its dataset's.
#[derive(Debug, Copy, Clone)]
pub enum DataColor {
    /// The palette colour for the dataset's index in its controller, shifted by the controller's
    /// palette offset (`BarController::palette_offset` and the like). The default.
    ByDataSet,
    /// This colour, whatever the chart's style.
    Color(Color),
    /// A palette colour per point, by the point's index in its dataset (shifted by the
    /// controller's palette offset), so each bar or point of the dataset gets the next colour,
    /// and the same index always gets the same colour. A line's stroke and its filled area,
    /// which are one shape for the whole dataset, keep the palette colour for the dataset's
    /// index, as with `ByDataSet`.
    ByPoint,
    /// The palette's colour at this index, wrapping around at its end, whatever the dataset's
    /// index and the controller's palette offset. It follows the chart's `SeriesStyle`, as
    /// `ByDataSet` does, but picks the colour itself: datasets in different controllers or
    /// stacks can share a colour, or keep one while datasets are added before them.
    Palette(usize),
}

impl From<Color> for DataColor {
    fn from(value: Color) -> Self {
        DataColor::Color(value)
    }
}

impl_state_value!(DataColor);