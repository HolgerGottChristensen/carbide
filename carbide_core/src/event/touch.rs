use crate::draw::Position;

/// A type for uniquely identifying the source of a touch interaction.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TouchId(u64);

/// The stage of the touch interaction.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TouchPhase {
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
    pub phase: TouchPhase,
    /// A unique identifier associated with the source of the touch interaction.
    pub id: TouchId,
    /// The location of the touch on the surface/screen. See `Input` docs for information on
    /// the co-ordinate system.
    pub position: Position,
}

impl TouchId {
    /// Construct a new identifier.
    pub fn new(id: u64) -> Self {
        TouchId(id)
    }
}

impl Touch {
    /// Returns a copy of the `Touch` relative to the given `position`.
    pub fn relative_to(&self, position: Position) -> Self {
        Touch {
            position: Position::new(self.position.x - position.x, self.position.y - position.y),
            ..*self
        }
    }
}
