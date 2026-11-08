use carbide::draw::{DrawStyle, Scalar};

pub struct CellBorders {
    pub top: CellBorder,
    pub bottom: CellBorder,
    pub left: CellBorder,
    pub right: CellBorder,
}

pub struct CellBorder {
    pub style: DrawStyle,
    pub width: Scalar
}