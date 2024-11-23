use std::hash::Hash;
use crate::identifiable::{AnyIdentifiableWidget};
use crate::picker::picker_selection::PickerSelection;
use carbide::state::{AnyState, Map2, ReadStateExtNew, StateContract, StateExtNew};
use carbide::widget::{AnyWidget, Sequence, WidgetId};

#[derive(Clone, Debug)]
pub struct PickerSequence<T, W> where T: StateContract + PartialEq + Eq + Hash, W: Sequence<dyn AnyIdentifiableWidget<T>> {
    pub selected: PickerSelection<T>,
    pub inner: W,
}

/*impl<T: StateContract + PartialEq + Eq + Hash, W: Sequence<dyn AnyIdentifiableWidget<T>>> SelectableSequence for PickerSequence<T, W> {
    fn has_changed(&self, existing: &mut dyn Iterator<Item=WidgetId>) -> bool {
        self.inner.has_changed(existing)
    }

    fn update(&self, f: &mut dyn FnMut(&dyn AnyWidget, Box<dyn AnyState<T=bool>>)) {
        self.inner.update(&mut |widget| {
            match self.selected.clone() {
                PickerSelection::Single(single) => {
                    f(widget.as_widget(), Map2::map(
                        widget.identifier().ignore_writes(),
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
                PickerSelection::Optional(optional) => {
                    f(widget.as_widget(), Map2::map(
                        widget.identifier().ignore_writes(),
                        optional,
                        |value, selection| {
                            selection.as_ref().is_some_and(|x| x == value)
                        },
                        |new, value, mut selection| {
                            if new {
                                *selection = Some(value.clone());
                            } else {
                                *selection = None;
                            }
                        }
                    ).as_dyn())
                }
                PickerSelection::Multi(multi) => {
                    f(widget.as_widget(), Map2::map(
                        widget.identifier().ignore_writes(),
                        multi,
                        |value, selection| {
                            selection.contains(value)
                        },
                        |new, value, mut selection| {
                            if new {
                                selection.insert(value.clone());
                            } else {
                                selection.remove(&*value);
                            }
                        }
                    ).as_dyn())
                }
            }
        });
    }
}*/