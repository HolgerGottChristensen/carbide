use crate::draw::Scalar;
use crate::event::TouchPhase;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Gesture {
    Rotate(Scalar, TouchPhase),
    Scale(Scalar, TouchPhase),
    SmartScale,
}
