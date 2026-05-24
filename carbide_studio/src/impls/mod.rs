use carbide::render::Style;
use carbide::state::ReadState;
use carbide::widget::{AnyWidget, Rectangle, VStack, WidgetExt};
use crate::core::AnyStudioWidget;

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> AnyStudioWidget for Rectangle<S, F> {
    fn editor(&self) -> Box<dyn AnyWidget> {
        VStack::new((
            Rectangle::new(),
            Rectangle::new(),
            Rectangle::new()
        )).boxed()
    }
}