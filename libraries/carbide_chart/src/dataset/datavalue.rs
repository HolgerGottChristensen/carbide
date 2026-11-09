use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::math::{One, Zero};

/// One coordinate of a data point. `PartialOrd` is for finding a dataset's bounds.
pub trait DataValue: Copy + Zero + One + Clone + Debug + PartialOrd + 'static {}

impl DataValue for Scalar {}
impl DataValue for usize {}