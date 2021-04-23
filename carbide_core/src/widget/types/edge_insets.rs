use crate::Scalar;

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
            right
        }
    }

    pub fn all(amount: Scalar) -> Self {
        EdgeInsets {
            top: amount,
            bottom: amount,
            left: amount,
            right: amount
        }
    }
}