use widget::common_widget::CommonWidget;

pub trait Layouter<S> {
    fn position(&self, widget: &mut dyn CommonWidget<S>) -> fn(&mut dyn CommonWidget<S>);
}