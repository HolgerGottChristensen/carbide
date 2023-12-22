use carbide_core::state::AnyReadState;

use crate::draw::Scalar;
use crate::state::{ConvertIntoRead, Map1, RMap1};

#[derive(Debug, Copy, Clone)]
pub struct EdgeInsets {
    pub top: Scalar,
    pub bottom: Scalar,
    pub left: Scalar,
    pub right: Scalar,
}

impl EdgeInsets {
    pub fn single(top: Scalar, bottom: Scalar, left: Scalar, right: Scalar) -> Self {
        EdgeInsets {
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn vertical_horizontal(vertical: Scalar, horizontal: Scalar) -> Self {
        EdgeInsets {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }

    pub fn all(amount: Scalar) -> Self {
        EdgeInsets {
            top: amount,
            bottom: amount,
            left: amount,
            right: amount,
        }
    }
}

impl Into<EdgeInsets> for f64 {
    fn into(self) -> EdgeInsets {
        EdgeInsets::all(self)
    }
}

impl Into<EdgeInsets> for u32 {
    fn into(self) -> EdgeInsets {
        EdgeInsets::all(self as f64)
    }
}

impl ConvertIntoRead<EdgeInsets> for f64 {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&f64)->EdgeInsets, f64, EdgeInsets, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |a| {
            EdgeInsets::all(*a)
        })
    }
}