use layout::layout::Layout;
use widget::common_widget::CommonWidget;
use position::Dimensions;
use ::{Point, Scalar};

pub enum BasicLayouter {
    TopLeading, Top, TopTrailing,
    Leading, Center, Trailing,
    BottomLeading, Bottom, BottomTrailing

}

impl BasicLayouter {

    fn leading_x(x: Scalar, _: Scalar, child: &mut CommonWidget) {
        child.set_x(x);
    }

    fn trailing_x(x: Scalar, width: Scalar, child: &mut CommonWidget) {
        child.set_x(x + width - child.get_width());
    }

    fn center_x(x: Scalar, width: Scalar, child: &mut CommonWidget) {
        child.set_x(x + width/2.0 - child.get_width()/2.0);
    }

    fn center_y(y: Scalar, height: Scalar, child: &mut CommonWidget) {
        child.set_y(y + height/2.0 - child.get_height()/2.0);
    }

    fn top_y(y: Scalar, _: Scalar, child: &mut CommonWidget) {
        child.set_y(y);
    }

    fn bottom_y(y: Scalar, height: Scalar, child: &mut CommonWidget) {
        child.set_y(y + height - child.get_height());
    }

    fn top_leading(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget) {
        BasicLayouter::leading_x(relative_to[0], dimensions[0], child);
        BasicLayouter::top_y(relative_to[1], dimensions[1], child);
    }

    fn top(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget) {
        BasicLayouter::center_x(relative_to[0], dimensions[0], child);
        BasicLayouter::top_y(relative_to[1], dimensions[1], child);
    }

    fn top_trailing(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget) {
        BasicLayouter::trailing_x(relative_to[0], dimensions[0], child);
        BasicLayouter::top_y(relative_to[1], dimensions[1], child);
    }

    fn leading(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget) {
        BasicLayouter::leading_x(relative_to[0], dimensions[0], child);
        BasicLayouter::center_y(relative_to[1], dimensions[1], child);
    }

    fn center(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget) {
        BasicLayouter::center_x(relative_to[0], dimensions[0], child);
        BasicLayouter::center_y(relative_to[1], dimensions[1], child);
    }

    fn trailing(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget) {
        BasicLayouter::trailing_x(relative_to[0], dimensions[0], child);
        BasicLayouter::center_y(relative_to[1], dimensions[1], child);
    }

    fn bottom_leading(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget) {
        BasicLayouter::leading_x(relative_to[0], dimensions[0], child);
        BasicLayouter::bottom_y(relative_to[1], dimensions[1], child);
    }

    fn bottom(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget) {
        BasicLayouter::center_x(relative_to[0], dimensions[0], child);
        BasicLayouter::bottom_y(relative_to[1], dimensions[1], child);
    }

    fn bottom_trailing(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget) {
        BasicLayouter::trailing_x(relative_to[0], dimensions[0], child);
        BasicLayouter::bottom_y(relative_to[1], dimensions[1], child);
    }

    pub fn position(&self) -> fn(Point, Dimensions, &mut dyn CommonWidget) {
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