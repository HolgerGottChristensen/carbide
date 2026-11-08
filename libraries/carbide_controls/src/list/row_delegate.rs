use dyn_clone::{clone_trait_object, DynClone};
use carbide::widget::AnyWidget;
use carbide::widget::foreach_widget::Delegate;
use crate::identifiable::AnySelectableWidget;

pub trait AnyRowDelegate: DynClone + 'static {
    fn call(&self, child: &dyn AnyWidget) -> Box<dyn AnyWidget>;
}

clone_trait_object!(AnyRowDelegate);

#[derive(Clone)]
pub struct RowDelegate(pub Box<dyn AnyRowDelegate>);

impl Delegate<dyn AnyWidget, Box<dyn AnyWidget>> for RowDelegate {
    fn call(&self, child: &dyn AnyWidget) -> Box<dyn AnyWidget> {
        self.0.call(child)
    }
}

pub trait AnySelectableRowDelegate: DynClone + 'static {
    fn call(&self, child: &dyn AnySelectableWidget) -> Box<dyn AnyWidget>;
}

clone_trait_object!(AnySelectableRowDelegate);

#[derive(Clone)]
pub struct SelectableRowDelegate(pub Box<dyn AnySelectableRowDelegate>);

impl Delegate<dyn AnySelectableWidget, Box<dyn AnyWidget>> for SelectableRowDelegate {
    fn call(&self, child: &dyn AnySelectableWidget) -> Box<dyn AnyWidget> {
        self.0.call(child)
    }
}