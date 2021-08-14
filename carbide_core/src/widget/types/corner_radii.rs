use crate::draw::Scalar;

#[derive(Debug, Copy, Clone)]
pub struct CornerRadii {
    pub top_left: Scalar,
    pub top_right: Scalar,
    pub bottom_left: Scalar,
    pub bottom_right: Scalar,
}

impl CornerRadii {
    pub fn single(
        top_left: Scalar,
        top_right: Scalar,
        bottom_left: Scalar,
        bottom_right: Scalar,
    ) -> Self {
        CornerRadii {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    pub fn all(amount: Scalar) -> Self {
        CornerRadii {
            top_left: amount,
            top_right: amount,
            bottom_left: amount,
            bottom_right: amount,
        }
    }
}

impl Into<CornerRadii> for f64 {
    fn into(self) -> CornerRadii {
        CornerRadii::all(self)
    }
}
