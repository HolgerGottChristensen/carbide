mod dataset;
mod datapoint;
mod datavalue;
mod dataset_sequence;

use carbide::draw::Color;
pub use datapoint::*;
pub use dataset::*;
pub use datavalue::*;
pub use dataset_sequence::*;

pub enum DataColor {
    Inherit,
    Color(Color)
}