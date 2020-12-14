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

    fn leading_x<S>(x: Scalar, _: Scalar, child: &mut CommonWidget<S>) {
        child.set_x(x);
    }

    fn trailing_x<S>(x: Scalar, width: Scalar, child: &mut CommonWidget<S>) {
        child.set_x(x + width - child.get_width());
    }

    fn center_x<S>(x: Scalar, width: Scalar, child: &mut CommonWidget<S>) {
        child.set_x(x + width/2.0 - child.get_width()/2.0);
    }

    fn center_y<S>(y: Scalar, height: Scalar, child: &mut CommonWidget<S>) {
        child.set_y(y + height/2.0 - child.get_height()/2.0);
    }

    fn top_y<S>(y: Scalar, _: Scalar, child: &mut CommonWidget<S>) {
        child.set_y(y);
    }

    fn bottom_y<S>(y: Scalar, height: Scalar, child: &mut CommonWidget<S>) {
        child.set_y(y + height - child.get_height());
    }

    fn top_leading<S>(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget<S>) {
        BasicLayouter::leading_x(relative_to[0], dimensions[0], child);
        BasicLayouter::top_y(relative_to[1], dimensions[1], child);
    }

    fn top<S>(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget<S>) {
        BasicLayouter::center_x(relative_to[0], dimensions[0], child);
        BasicLayouter::top_y(relative_to[1], dimensions[1], child);
    }

    fn top_trailing<S>(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget<S>) {
        BasicLayouter::trailing_x(relative_to[0], dimensions[0], child);
        BasicLayouter::top_y(relative_to[1], dimensions[1], child);
    }

    fn leading<S>(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget<S>) {
        BasicLayouter::leading_x(relative_to[0], dimensions[0], child);
        BasicLayouter::center_y(relative_to[1], dimensions[1], child);
    }

    fn center<S>(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget<S>) {
        BasicLayouter::center_x(relative_to[0], dimensions[0], child);
        BasicLayouter::center_y(relative_to[1], dimensions[1], child);
    }

    fn trailing<S>(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget<S>) {
        BasicLayouter::trailing_x(relative_to[0], dimensions[0], child);
        BasicLayouter::center_y(relative_to[1], dimensions[1], child);
    }

    fn bottom_leading<S>(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget<S>) {
        BasicLayouter::leading_x(relative_to[0], dimensions[0], child);
        BasicLayouter::bottom_y(relative_to[1], dimensions[1], child);
    }

    fn bottom<S>(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget<S>) {
        BasicLayouter::center_x(relative_to[0], dimensions[0], child);
        BasicLayouter::bottom_y(relative_to[1], dimensions[1], child);
    }

    fn bottom_trailing<S>(relative_to: Point, dimensions: Dimensions, child: &mut CommonWidget<S>) {
        BasicLayouter::trailing_x(relative_to[0], dimensions[0], child);
        BasicLayouter::bottom_y(relative_to[1], dimensions[1], child);
    }

    pub fn position<S>(&self) -> fn(Point, Dimensions, &mut dyn CommonWidget<S>) {
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