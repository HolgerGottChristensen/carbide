//! Contains all render used to describe the input events that `Widget`s may handle.
//!
//! The two primary render of this module are:
//!
//! - `Input`: conrod's input type passed by the user to `Ui::handle_event` in order to drive the
//! `Ui`.
//! - `Event`: enumerates all possible events interpreted by conrod that may be propagated to
//! widgets.
//!
//! The Event System
//! ----------------
//!
//! Conrod's event system looks like this:
//!
//! *Input -> Ui -> Event -> Widget*
//!
//! The **Ui** receives **Input**s such as `Press` and `Release` via the `Ui::handle_event` method.
//! It interprets these **Input**s to create higher-level **Event**s such as `DoubleClick`,
//! `WidgetCapturesKeyboard`, etc. These **Event**s are stored and then fed to each **Widget** when
//! `Ui::set_widgets` is called. At the end of `Ui::set_widgets` the stored **Event**s are flushed
//! ready for the next incoming **Input**s.
//!
//! Conrod uses the `pistoncore-input` crate's `Input` type. There are a few reasons for this:
//!
//! 1. This `Input` type already provides a number of useful variants of events that we wish to
//!    provide and handle within conrod, and we do not yet see any great need to re-write it and
//!    duplicate code.
//! 2. The `Input` type is already compatible with all `pistoncore-window` backends including
//!    `glfw_window`, `sdl2_window` and `glutin_window`. That said, co-ordinates and scroll
//!    directions may need to be translated to conrod's orientation.
//! 3. The `pistoncore-input` crate also provides a `GenericEvent` trait which allows us to easily
//!    provide a blanket implementation of `ToRawEvent` for all event render that already implement
//!    this trait.
//!
//! Because we use the `pistoncore-input` `Event` type, we also re-export its associated data
//! render (`Button`, `ControllerAxisArgs`, `Key`, etc).

use crate::Scalar;

pub mod event;
pub mod input;
//pub mod motion;

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