use carbide::draw::Dimension;
use crate::draw::{Angle, Position};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PathInstruction {
    MoveTo {
        to: Position,
    },
    Close,
    LineTo {
        to: Position,
    },
    QuadraticBezierTo {
        ctrl: Position,
        to: Position,
    },
    CubicBezierTo {
        ctrl1: Position,
        ctrl2: Position,
        to: Position,
    },
    Arc {
        center: Position,
        radius: Dimension,
        start_angle: Angle,
        end_angle: Angle,
    },
}