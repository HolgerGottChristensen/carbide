use crate::identifiable::Identifiable;
use carbide::state::StateContract;
use carbide::widget::{AnyWidget, WidgetExt, WidgetId};
use std::fmt::Debug;

pub trait AnyIdentifiableWidget<T>: AnyWidget + Identifiable<T> where T: StateContract + PartialEq {
    fn as_widget(&self) -> &dyn AnyWidget;
}

impl<T: StateContract + PartialEq, W> AnyIdentifiableWidget<T> for W where W: AnyWidget + Identifiable<T> + Clone {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }
}

dyn_clone::clone_trait_object!(<T> AnyIdentifiableWidget<T>);

pub trait IdentifiableWidget<T>: AnyIdentifiableWidget<T> + WidgetExt + Clone where T: StateContract + PartialEq  {
    fn has_changed_child(&self, existing: &mut dyn Iterator<Item=WidgetId>) -> bool;
}

impl<T: StateContract + PartialEq, W> IdentifiableWidget<T> for W where W: AnyWidget + Identifiable<T> + WidgetExt + Clone {
    fn has_changed_child(&self, existing: &mut dyn Iterator<Item=WidgetId>) -> bool {
        let mut changed = false;

        self.foreach_child(&mut |identifiable| {
            if !changed {
                if let Some(id) = existing.next() {
                    changed = changed | (identifiable.id() == id)
                } else {
                    changed = true;
                }
            }
        });

        changed
    }
}