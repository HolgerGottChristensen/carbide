use crate::Scalar;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Motion {
    /// Absolute cursor position within the window.
    ///
    /// For more details on co-ordinate orientation etc, see the `Input` docs.
    MouseCursor { x: Scalar, y: Scalar },
    /// Relative mouse movement.
    MouseRelative { x: Scalar, y: Scalar },
    /// x and y in scroll ticks.
    Scroll { x: Scalar, y: Scalar },
    // /// controller axis move event.
    //ControllerAxis(crate::piston_input::ControllerAxisArgs),
}