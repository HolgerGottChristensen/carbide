use crate::draw::{Dimension, Position};

#[derive(Clone, Debug, PartialEq)]
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
}

impl Alignment {
    pub fn position(&self, position: Position, dimension: Dimension) -> Position {
        match self {
            Alignment::TopLeading => Alignment::top_leading(position, dimension),
            Alignment::Top => Alignment::top(position, dimension),
            Alignment::TopTrailing => Alignment::top_trailing(position, dimension),
            Alignment::Leading => Alignment::leading(position, dimension),
            Alignment::Center => Alignment::center(position, dimension),
            Alignment::Trailing => Alignment::trailing(position, dimension),
            Alignment::BottomLeading => Alignment::bottom_leading(position, dimension),
            Alignment::Bottom => Alignment::bottom(position, dimension),
            Alignment::BottomTrailing => Alignment::bottom_trailing(position, dimension),
        }
    }

    fn top_leading(position: Position, _: Dimension) -> Position {
        position
    }

    fn top(position: Position, dimension: Dimension) -> Position {
        Position::new(position.x + dimension.width / 2.0, position.y)
    }

    fn top_trailing(position: Position, dimension: Dimension) -> Position {
        Position::new(position.x + dimension.width, position.y)
    }

    fn leading(position: Position, dimension: Dimension) -> Position {
        Position::new(position.x, position.y + dimension.height / 2.0)
    }

    fn center(position: Position, dimension: Dimension) -> Position {
        Position::new(
            position.x + dimension.width / 2.0,
            position.y + dimension.height / 2.0,
        )
    }

    fn trailing(position: Position, dimension: Dimension) -> Position {
        Position::new(
            position.x + dimension.width,
            position.y + dimension.height / 2.0,
        )
    }

    fn bottom_leading(position: Position, dimension: Dimension) -> Position {
        Position::new(position.x, position.y + dimension.height)
    }

    fn bottom(position: Position, dimension: Dimension) -> Position {
        Position::new(
            position.x + dimension.width / 2.0,
            position.y + dimension.height,
        )
    }

    fn bottom_trailing(position: Position, dimension: Dimension) -> Position {
        Position::new(position.x + dimension.width, position.y + dimension.height)
    }
}
