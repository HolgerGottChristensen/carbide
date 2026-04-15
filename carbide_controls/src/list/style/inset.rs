use carbide::draw::{Alignment, Scalar};
use carbide::environment::EnvironmentColor;
use crate::list::row_delegate::{AnyRowDelegate, AnySelectableRowDelegate, RowDelegate, SelectableRowDelegate};
use crate::list::style::ListStyle;
use carbide::widget::{AnySequence, AnyWidget, CommonWidget, EdgeInsets, Frame, LazyVStack, Rectangle, RoundedRectangle, Scroll, VStack, WidgetExt};
use crate::identifiable::AnySelectableWidget;

#[derive(Copy, Clone, Debug)]
pub struct InsetStyle;

impl ListStyle for InsetStyle {
    fn base(&self, content: Box<dyn AnySequence>) -> Box<dyn AnyWidget> {
        Scroll::new(
            LazyVStack::new(
                content
            ).spacing(0.0)
                .padding(EdgeInsets::vertical_horizontal(4.0, 0.0))
        )
            .clip()
            .boxed()
    }

    fn row(&self) -> RowDelegate {
        RowDelegate(Box::new(InsetRowDelegate))
    }

    fn selectable_row(&self) -> SelectableRowDelegate {
        SelectableRowDelegate(Box::new(InsetSelectableRowDelegate))
    }
}

#[derive(Clone, Debug)]
struct InsetRowDelegate;

impl AnyRowDelegate for InsetRowDelegate {
    fn call(&self, child: &dyn AnyWidget) -> Box<dyn AnyWidget> {
        VStack::new((
            child
                .boxed()
                .fit()
                .expand_width()
                .alignment(Alignment::Leading)
                .padding(EdgeInsets::vertical_horizontal(3.0, 0.0)),
            Rectangle::new()
                .fill(EnvironmentColor::Separator)
                .frame_fixed_height(0.5)
                .padding(EdgeInsets::vertical_horizontal(0.0, 2.0))
        )).spacing(0.0)
            .padding(EdgeInsets::vertical_horizontal(0.0, 10.0))
            .boxed()
    }
}

#[derive(Clone, Debug)]
struct InsetSelectableRowDelegate;

impl AnySelectableRowDelegate for InsetSelectableRowDelegate {
    fn call(&self, child: &dyn AnySelectableWidget) -> Box<dyn AnyWidget> {
        child
            .as_widget()
            .boxed()
            .frame(0.0, 0.0)
            .fit_height()
            .expand_width()
            .alignment(Alignment::Leading)
            .padding(EdgeInsets::vertical_horizontal(3.0, 6.0))
            .background(
                RoundedRectangle::new(4.0)
                    .fill(EnvironmentColor::SystemFill)
                    .padding(EdgeInsets::vertical_horizontal(-4.0, 0.0))
            )
            .boxed()
    }
}
