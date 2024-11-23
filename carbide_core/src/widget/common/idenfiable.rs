use crate::widget::WidgetId;

pub trait Identifiable {
    fn id(&self) -> WidgetId;
}