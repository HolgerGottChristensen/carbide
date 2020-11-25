use widget::common_widget::CommonWidget;

pub trait Layouter {
    fn position(&self, widget: &mut dyn CommonWidget) -> fn(&mut dyn CommonWidget);
}