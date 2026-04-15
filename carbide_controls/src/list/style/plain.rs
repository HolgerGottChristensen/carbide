use carbide::draw::Scalar;
use crate::list::row_delegate::{RowDelegate, SelectableRowDelegate};
use crate::list::style::ListStyle;
use carbide::widget::{AnySequence, AnyWidget, LazyVStack, Scroll, WidgetExt};
use crate::identifiable::AnySelectableWidget;

#[derive(Copy, Clone, Debug)]
pub struct PlainStyle(pub Scalar);

impl ListStyle for PlainStyle {
    fn base(&self, content: Box<dyn AnySequence>) -> Box<dyn AnyWidget> {
        Scroll::new(
            LazyVStack::new(
                content
            ).spacing(self.0)
        )
            .clip()
            .boxed()
    }

    fn requires_row_wrapping(&self) -> bool {
        false
    }

    fn row(&self) -> RowDelegate {
        unimplemented!("Should never be called because we do not require wrapping")
    }

    fn selectable_row(&self) -> SelectableRowDelegate {
        unimplemented!("Should never be called because we do not require wrapping")
    }
}