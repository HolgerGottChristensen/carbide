use layout::layout::Layout;
use widget::common_widget::CommonWidget;
use position::Dimensions;
use Point;

pub enum BasicLayouter {
    TopLeading, Top, TopTrailing,
    Leading, Center, Trailing,
    BottomLeading, Bottom, BottomTrailing

}

impl BasicLayouter {
    /*fn get(&self, widget: &mut dyn CommonWidget, dimensions: Dimensions) -> &dyn Fn(&mut CommonWidget, Dimensions){
        match self {
            BasicLayouter::TopLeading => {

            },
            BasicLayouter::Top => {},
            BasicLayouter::TopTrailing => {},
            BasicLayouter::Leading => {},
            BasicLayouter::Center => {},
            BasicLayouter::Trailing => {},
            BasicLayouter::BottomLeading => {},
            BasicLayouter::Bottom => {},
            BasicLayouter::BottomTrailing => {},
        }
    }*/
}