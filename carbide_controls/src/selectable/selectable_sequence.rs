use dyn_clone::DynClone;
use carbide::identifiable::Identifiable;
use crate::identifiable::{AnySelectableWidget, SelectableWidget};
use carbide::reverse;
use carbide::widget::{AnySequence, Sequence, WidgetId, WidgetSync};
use carbide::widget::foreach_widget::{Delegate, ForEachWidget};

impl<S: SelectableWidget> AnySequence<dyn AnySelectableWidget> for Vec<S> {
    fn index(&mut self, index: usize) -> &mut dyn AnySelectableWidget {
        todo!()
    }

    fn count(&mut self) -> usize {
        todo!()
    }

    fn foreach(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnySelectableWidget::foreach_child(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        for element in &mut self.iter_mut().rev() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnySelectableWidget::foreach_child_rev(element, f);
                continue;
            }

            f(element);
        }
    }
}

macro_rules! tuple_sequence_impl {
    ($($generic:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<$($generic: SelectableWidget),*> AnySequence<dyn AnySelectableWidget> for ($($generic),*) {
            fn index(&mut self, index: usize) -> &mut dyn AnySelectableWidget {
                todo!()
            }

            fn count(&mut self) -> usize {
                todo!()
            }

            fn foreach(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        AnySelectableWidget::foreach_child($generic, f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
                let reverse!([$($generic)*]) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        AnySelectableWidget::foreach_child_rev($generic, f);
                    } else {
                        f($generic);
                    }
                )*
            }
        }
    };
}

#[allow(non_snake_case)]
#[allow(unused_parens)]
impl<
    T: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static,
    W: Sequence<T>,
    O: SelectableWidget,
    D: Delegate<T, O>
> AnySequence<dyn AnySelectableWidget> for ForEachWidget<W, O, D, T> {
    fn index(&mut self, index: usize) -> &mut dyn AnySelectableWidget {
        todo!()
    }

    fn count(&mut self) -> usize {
        todo!()
    }

    fn foreach(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        todo!()//AnySelectableWidget::foreach_child(self, f);
    }

    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        todo!()//AnySelectableWidget::foreach_child_rev(self, f);
    }
}

//tuple_sequence_impl!(W1);
tuple_sequence_impl!(W1, W2);
tuple_sequence_impl!(W1, W2, W3);
tuple_sequence_impl!(W1, W2, W3, W4);
tuple_sequence_impl!(W1, W2, W3, W4, W5);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11);
tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12);


impl AnySequence<dyn AnySelectableWidget> for dyn AnySelectableWidget {
    fn index(&mut self, index: usize) -> &mut dyn AnySelectableWidget {
        todo!()
    }

    fn count(&mut self) -> usize {
        todo!()
    }

    fn foreach(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        todo!()
    }

    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        todo!()
    }
}