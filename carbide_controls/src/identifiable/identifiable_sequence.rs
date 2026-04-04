use crate::identifiable::{AnyIdentifiableWidget, IdentifiableWidget};
use carbide::reverse;
use carbide::state::StateContract;
use carbide::widget::{AnySequence, Content};

impl<T: PartialEq + StateContract, S: IdentifiableWidget<T>> AnySequence<dyn AnyIdentifiableWidget<T>> for Vec<S> {

    fn index_mut(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T> {
        todo!()
    }

    fn count(&mut self) -> usize {
        todo!()
    }

    fn foreach_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnyIdentifiableWidget::foreach_child_mut(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        for element in &mut self.iter_mut().rev() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnyIdentifiableWidget::foreach_child_rev(element, f);
                continue;
            }

            f(element);
        }
    }
}

impl<W: IdentifiableWidget<T>, T: StateContract + PartialEq> AnySequence<dyn AnyIdentifiableWidget<T>> for Content<W> {
    fn index_mut(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T> {
        todo!()
    }

    fn count(&mut self) -> usize {
        todo!()
    }

    fn foreach_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        for (_, element) in self.0.iter_mut().take(self.1) {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnyIdentifiableWidget::foreach_child_mut(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        for (_, element) in self.0.iter_mut().take(self.1).rev() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnyIdentifiableWidget::foreach_child_rev(element, f);
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
        impl<T: StateContract + PartialEq, $($generic: IdentifiableWidget<T>),*> AnySequence<dyn AnyIdentifiableWidget<T>> for ($($generic),*) {
            fn index_mut(&mut self, index: usize) -> &mut dyn AnyIdentifiableWidget<T> {
                todo!()
            }

            fn count(&mut self) -> usize {
                todo!()
            }

            fn foreach_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        AnyIdentifiableWidget::foreach_child_mut($generic, f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
                let reverse!([$($generic)*]) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        AnyIdentifiableWidget::foreach_child_rev($generic, f);
                    } else {
                        f($generic);
                    }
                )*
            }
        }
    };
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