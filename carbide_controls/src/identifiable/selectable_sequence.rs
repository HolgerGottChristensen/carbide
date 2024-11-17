use carbide::state::AnyState;
use carbide::widget::{AnyWidget, WidgetId};
use dyn_clone::{clone_trait_object, DynClone};
use std::fmt::Debug;
use std::ops::Deref;

pub trait SelectableSequence: DynClone + Debug + 'static {
    fn has_changed(&self, existing: &mut dyn Iterator<Item=WidgetId>) -> bool;
    fn update(&self, f: &mut dyn FnMut(&dyn AnyWidget, Box<dyn AnyState<T=bool>>));
}

clone_trait_object!(SelectableSequence);

impl SelectableSequence for Box<dyn SelectableSequence> {
    fn has_changed(&self, existing: &mut dyn Iterator<Item=WidgetId>) -> bool {
        self.deref().has_changed(existing)
    }

    fn update(&self, f: &mut dyn FnMut(&dyn AnyWidget, Box<dyn AnyState<T=bool>>)) {
        self.deref().update(f)
    }
}