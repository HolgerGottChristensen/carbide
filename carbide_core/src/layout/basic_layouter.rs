use crate::draw::{Dimension, Position, Scalar};
use crate::layout::layouter::Layouter;
use crate::prelude::Widget;

#[derive(Copy, Clone, Debug)]
pub enum BasicLayouter {
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

impl BasicLayouter {
    fn leading_x(x: Scalar, _: Scalar, child: &mut dyn Widget) {
        child.set_x(x);
    }

    fn trailing_x(x: Scalar, width: Scalar, child: &mut dyn Widget) {
        child.set_x(x + width - child.width());
    }

    fn center_x(x: Scalar, width: Scalar, child: &mut dyn Widget) {
        child.set_x(x + width / 2.0 - child.width() / 2.0);
    }

    fn center_y(y: Scalar, height: Scalar, child: &mut dyn Widget) {
        child.set_y(y + height / 2.0 - child.height() / 2.0);
    }

    fn top_y(y: Scalar, _: Scalar, child: &mut dyn Widget) {
        child.set_y(y);
    }

    fn bottom_y(y: Scalar, height: Scalar, child: &mut dyn Widget) {
        child.set_y(y + height - child.height());
    }

    fn top_leading(relative_to: Position, dimensions: Dimension, child: &mut dyn Widget) {
        BasicLayouter::leading_x(relative_to.x, dimensions.width, child);
        BasicLayouter::top_y(relative_to.y, dimensions.height, child);
    }

    fn top(relative_to: Position, dimensions: Dimension, child: &mut dyn Widget) {
        BasicLayouter::center_x(relative_to.x, dimensions.width, child);
        BasicLayouter::top_y(relative_to.y, dimensions.height, child);
    }

    fn top_trailing(relative_to: Position, dimensions: Dimension, child: &mut dyn Widget) {
        BasicLayouter::trailing_x(relative_to.x, dimensions.width, child);
        BasicLayouter::top_y(relative_to.y, dimensions.height, child);
    }

    fn leading(relative_to: Position, dimensions: Dimension, child: &mut dyn Widget) {
        BasicLayouter::leading_x(relative_to.x, dimensions.width, child);
        BasicLayouter::center_y(relative_to.y, dimensions.height, child);
    }

    fn center(relative_to: Position, dimensions: Dimension, child: &mut dyn Widget) {
        BasicLayouter::center_x(relative_to.x, dimensions.width, child);
        BasicLayouter::center_y(relative_to.y, dimensions.height, child);
    }

    fn trailing(relative_to: Position, dimensions: Dimension, child: &mut dyn Widget) {
        BasicLayouter::trailing_x(relative_to.x, dimensions.width, child);
        BasicLayouter::center_y(relative_to.y, dimensions.height, child);
    }

    fn bottom_leading(relative_to: Position, dimensions: Dimension, child: &mut dyn Widget) {
        BasicLayouter::leading_x(relative_to.x, dimensions.width, child);
        BasicLayouter::bottom_y(relative_to.y, dimensions.height, child);
    }

    fn bottom(relative_to: Position, dimensions: Dimension, child: &mut dyn Widget) {
        BasicLayouter::center_x(relative_to.x, dimensions.width, child);
        BasicLayouter::bottom_y(relative_to.y, dimensions.height, child);
    }

    fn bottom_trailing(relative_to: Position, dimensions: Dimension, child: &mut dyn Widget) {
        BasicLayouter::trailing_x(relative_to.x, dimensions.width, child);
        BasicLayouter::bottom_y(relative_to.y, dimensions.height, child);
    }
}

impl Layouter for BasicLayouter {
    fn positioner(&self) -> fn(Position, Dimension, &mut dyn Widget) {
        match self {
            BasicLayouter::TopLeading => BasicLayouter::top_leading,
            BasicLayouter::Top => BasicLayouter::top,
            BasicLayouter::TopTrailing => BasicLayouter::top_trailing,
            BasicLayouter::Leading => BasicLayouter::leading,
            BasicLayouter::Center => BasicLayouter::center,
            BasicLayouter::Trailing => BasicLayouter::trailing,
            BasicLayouter::BottomLeading => BasicLayouter::bottom_leading,
            BasicLayouter::Bottom => BasicLayouter::bottom,
            BasicLayouter::BottomTrailing => BasicLayouter::bottom_trailing,
        }
    }
}
