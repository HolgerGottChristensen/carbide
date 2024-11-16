mod radio_style;

use std::any::{type_name, TypeId};
pub use radio_style::*;

use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use indexmap::IndexMap;
use indexmap::map::Keys;
use carbide::environment::Key;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, LocalState, Map2, ReadStateExtNew, State, StateContract, StateExtNew};
use carbide::widget::{AnyWidget, BuildWidgetIdHasher, Widget, WidgetExt, WidgetId};
use crate::identifiable::{AnyIdentifiableWidget, ExistingOrNew, Identifiable, IdentifiableWidget, IdentifiableWidgetSequence};
use crate::picker::picker_selection::PickerSelection;

#[derive(Debug, Copy, Clone)]
pub(crate) struct PickerStyleKey;

impl Key for PickerStyleKey {
    type Value = Box<dyn PickerStyle>;
}

pub trait PickerStyle: Debug + DynClone {
    fn create(
        &self,
        focus: Box<dyn AnyState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        label: Box<dyn AnyReadState<T=String>>,
        model: Box<dyn SelectableWidgetSequence>,
    ) -> Box<dyn AnyWidget>;

    fn test<T>(self, t: T) -> T where Self: Sized;
}

impl<'a> PickerStyle for &'a dyn PickerStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn SelectableWidgetSequence>) -> Box<dyn AnyWidget> {
        todo!()
    }

    fn test<T>(self, t: T) -> T
    where
        Self: Sized
    {
        println!("Ref implementation called with: {}", type_name::<T>());
        t
    }
}

clone_trait_object!(PickerStyle);

pub trait SelectableWidgetSequence: DynClone + Debug + 'static {
    fn has_changed(&self, existing: &mut Keys<'_, WidgetId, Box<dyn AnyWidget>>) -> bool;
    fn update(&self, f: &mut dyn FnMut(Box<dyn AnyWidget>, Box<dyn AnyState<T=bool>>));
}

clone_trait_object!(SelectableWidgetSequence);

#[derive(Clone, Debug)]
pub struct TestSelectableWidgetSequence<T, W> where T: StateContract + PartialEq, W: IdentifiableWidgetSequence<T> {
    pub selected: PickerSelection<T>,
    pub inner: W,
}

impl<T: StateContract + PartialEq, W: IdentifiableWidgetSequence<T>> SelectableWidgetSequence for TestSelectableWidgetSequence<T, W> {
    fn has_changed(&self, existing: &mut Keys<'_, WidgetId, Box<dyn AnyWidget>>) -> bool {
        self.inner.has_changed(existing)
    }

    fn update(&self, f: &mut dyn FnMut(Box<dyn AnyWidget>, Box<dyn AnyState<T=bool>>)) {
        self.inner.update(&mut |widget, state| {
            match self.selected.clone() {
                PickerSelection::Single(single) => {
                    f(widget, Map2::map(
                        state.ignore_writes(),
                        single,
                        |value, selection| {
                            value == selection
                        },
                        |new, value, mut selection| {
                            if new {
                                *selection = value.clone();
                            }
                        }
                    ).as_dyn())
                }
                PickerSelection::Optional(optional) => {}
                PickerSelection::Multi(multi) => {}
            }

        });
    }
}

