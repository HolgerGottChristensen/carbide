mod dataset;
mod datapoint;
mod datavalue;
mod dataset_sequence;
mod dataset_options;
mod dataset_ext;
mod options_dataset;

use carbide::draw::Color;
use carbide::impl_state_value;
pub use datapoint::*;
pub use dataset::*;
pub use datavalue::*;
pub use dataset_sequence::*;
pub use dataset_options::*;
pub use dataset_ext::*;

#[derive(Debug, Copy, Clone)]
pub enum DataColor {
    Inherit,
    Color(Color)
}

impl From<Color> for DataColor {
    fn from(value: Color) -> Self {
        DataColor::Color(value)
    }
}

impl_state_value!(DataColor);