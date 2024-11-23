use carbide::state::StateContract;
use carbide::widget::{AnyWidget, WidgetExt};

pub trait Identifiable<I: StateContract + PartialEq> {
    fn id(&self) -> I;
}

pub trait AnyIdentifiableWidget<T>: AnyWidget + Identifiable<T> where T: StateContract + PartialEq {
    fn as_widget(&self) -> &dyn AnyWidget;
}

impl<T: StateContract + PartialEq, W> AnyIdentifiableWidget<T> for W where W: AnyWidget + Identifiable<T> + Clone {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }
}

dyn_clone::clone_trait_object!(<T> AnyIdentifiableWidget<T>);

pub trait IdentifiableWidget<T>: AnyIdentifiableWidget<T> + WidgetExt + Clone where T: StateContract + PartialEq  {}

impl<T: StateContract + PartialEq, W> IdentifiableWidget<T> for W where W: AnyWidget + Identifiable<T> + WidgetExt + Clone {}