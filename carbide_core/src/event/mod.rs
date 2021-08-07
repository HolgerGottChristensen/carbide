pub use event::Event;
pub use event_handler::*;

use crate::Scalar;

pub use self::input::Input;

mod event;
mod input;
mod event_handler;

#[allow(missing_docs)]
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
    /// controller axis move event.
    ControllerAxis(crate::piston_input::ControllerAxisArgs),
}

/// Touch-related items.
pub mod touch {
    use crate::Point;

    /// A type for uniquely identifying the source of a touch interaction.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Id(u64);

    /// The stage of the touch interaction.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub enum Phase {
        /// The start of a touch interaction.
        Start,
        /// A touch moving across a surface.
        Move,
        /// The touch interaction was cancelled.
        Cancel,
        /// The end of a touch interaction.
        End,
    }

    /// Represents a touch interaction.
    ///
    /// Each time a user touches the surface with a new finger, a new series of `Touch` events
    /// `Start`, each with a unique identifier.
    ///
    /// For every `Id` there should be at least 2 events with `Start` and `End` (or `Cancel`led)
    /// `Phase`s.
    ///
    /// A `Start` input received with the same `Id` as a previously received `End` does *not*
    /// indicate that the same finger was used. `Id`s are only used to distinguish between
    /// overlapping touch interactions.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Touch {
        /// The stage of the touch interaction.
        pub phase: Phase,
        /// A unique identifier associated with the source of the touch interaction.
        pub id: Id,
        /// The location of the touch on the surface/screen. See `Input` docs for information on
        /// the co-ordinate system.
        pub xy: Point,
    }

    impl Id {
        /// Construct a new identifier.
        pub fn new(id: u64) -> Self {
            Id(id)
        }
    }

    impl Touch {
        /// Returns a copy of the `Touch` relative to the given `xy`.
        pub fn relative_to(&self, xy: Point) -> Self {
            Touch {
                xy: [self.xy[0] - xy[0], self.xy[1] - xy[1]],
                ..*self
            }
        }
    }
}
