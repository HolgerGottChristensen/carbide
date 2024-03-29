use crate::draw::{Dimension, Position};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Alignment {
    TopLeading,
    Top,
    TopTrailing,
    Leading,
    Center,
    Trailing,
    BottomLeading,
    Bottom,
    BottomTrailing,
    Custom(f64, f64),
}

impl Alignment {
    pub fn position(&self, position: Position, outer: Dimension, inner: Dimension) -> Position {
        match self {
            Alignment::TopLeading => Alignment::top_leading(position, outer, inner),
            Alignment::Top => Alignment::top(position, outer, inner),
            Alignment::TopTrailing => Alignment::top_trailing(position, outer, inner),
            Alignment::Leading => Alignment::leading(position, outer, inner),
            Alignment::Center => Alignment::center(position, outer, inner),
            Alignment::Trailing => Alignment::trailing(position, outer, inner),
            Alignment::BottomLeading => Alignment::bottom_leading(position, outer, inner),
            Alignment::Bottom => Alignment::bottom(position, outer, inner),
            Alignment::BottomTrailing => Alignment::bottom_trailing(position, outer, inner),
            Alignment::Custom(x, y) => {
                Position::new(position.x + outer.width * *x - inner.width / 2.0, position.y + outer.height * *y - inner.height / 2.0)
            }
        }
    }

    fn top_leading(position: Position, _: Dimension, _: Dimension) -> Position {
        position
    }

    fn top(position: Position, outer: Dimension, inner: Dimension) -> Position {
        Position::new(
            position.x + outer.width / 2.0 - inner.width / 2.0,
            position.y
        )
    }

    fn top_trailing(position: Position, outer: Dimension, inner: Dimension) -> Position {
        Position::new(
            position.x + outer.width - inner.width,
            position.y
        )
    }

    fn leading(position: Position, outer: Dimension, inner: Dimension) -> Position {
        Position::new(
            position.x,
            position.y + outer.height / 2.0 - inner.height / 2.0
        )
    }

    fn center(position: Position, outer: Dimension, inner: Dimension) -> Position {
        Position::new(
            position.x + outer.width / 2.0 - inner.width / 2.0,
            position.y + outer.height / 2.0 - inner.height / 2.0,
        )
    }

    fn trailing(position: Position, outer: Dimension, inner: Dimension) -> Position {
        Position::new(
            position.x + outer.width - inner.width,
            position.y + outer.height / 2.0 - inner.height / 2.0,
        )
    }

    fn bottom_leading(position: Position, outer: Dimension, inner: Dimension) -> Position {
        Position::new(
            position.x,
            position.y + outer.height - inner.height
        )
    }

    fn bottom(position: Position, outer: Dimension, inner: Dimension) -> Position {
        Position::new(
            position.x + outer.width / 2.0,
            position.y + outer.height - inner.height,
        )
    }

    fn bottom_trailing(position: Position, outer: Dimension, inner: Dimension) -> Position {
        Position::new(
            position.x + outer.width - inner.width,
            position.y + outer.height - inner.height
        )
    }
}
