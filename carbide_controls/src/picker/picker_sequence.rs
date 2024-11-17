use crate::identifiable::{IdentifiableWidgetSequence, SelectableSequence};
use crate::picker::picker_selection::PickerSelection;
use carbide::state::{AnyState, Map2, ReadStateExtNew, StateContract, StateExtNew};
use carbide::widget::{AnyWidget, WidgetId};

#[derive(Clone, Debug)]
pub struct PickerSequence<T, W> where T: StateContract + PartialEq, W: IdentifiableWidgetSequence<T> {
    pub selected: PickerSelection<T>,
    pub inner: W,
}

impl<T: StateContract + PartialEq, W: IdentifiableWidgetSequence<T>> SelectableSequence for PickerSequence<T, W> {
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
                PickerSelection::Optional(optional) => {}
                PickerSelection::Multi(multi) => {}
            }
        });
    }
}