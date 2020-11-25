use Scalar;

#[derive(Debug, Copy, Clone)]
pub struct EdgeInsets {
    pub top: Scalar,
    pub bottom: Scalar,
    pub left: Scalar,
    pub right: Scalar,
}

impl EdgeInsets {
    pub fn all(amount: Scalar) -> Self {
        EdgeInsets {
            top: amount,
            bottom: amount,
            left: amount,
            right: amount
        }
    }
}