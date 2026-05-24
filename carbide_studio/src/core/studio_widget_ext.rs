use carbide::widget::WidgetProperties;
use crate::core::any_studio_widget::AnyStudioWidget;

pub trait StudioWidgetExt: AnyStudioWidget + WidgetProperties + Clone + Sized {
    fn studio_boxed(self) -> Box<dyn AnyStudioWidget> {
        Box::new(self)
    }
}

impl<T> StudioWidgetExt for T where T: AnyStudioWidget + WidgetProperties + Clone + Sized {}