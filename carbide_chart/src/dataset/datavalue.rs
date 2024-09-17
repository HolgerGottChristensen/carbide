use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::render::matrix::{One, Zero};

pub trait DataValue: Copy + Zero + One + Clone + Debug {}

impl DataValue for Scalar {}
impl DataValue for usize {}