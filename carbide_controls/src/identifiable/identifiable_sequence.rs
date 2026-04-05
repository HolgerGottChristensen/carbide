use crate::identifiable::{AnyIdentifiableWidget, AnySelectableWidget, IdentifiableWidget, SelectableWidget};
use carbide::reverse;
use carbide::state::StateContract;
use carbide::widget::{AnySequence, ReverseAnySequence};

impl<T: PartialEq + StateContract, W: IdentifiableWidget<T>> ReverseAnySequence<Vec<W>> for dyn AnyIdentifiableWidget<T> {
    fn index(value: &mut Vec<W>, index: usize) -> &mut dyn AnyIdentifiableWidget<T> {
        let mut passed = 0;

        for element in value.iter_mut() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                let child_count = element.child_count();

                if index < passed + child_count {
                    return AnyIdentifiableWidget::child(element, index - passed);
                }

                passed += child_count;

                continue;
            }

            if index == passed {
                return element;
            }

            passed += 1;
        }

        panic!("Index out of bounds. Index: {}, Passed: {}", index, passed);
    }

    fn count(value: &mut Vec<W>) -> usize {
        let mut count = 0;
        for element in value {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                count += element.child_count();
                continue;
            }

            count += 1;
        }

        count
    }

    fn foreach(value: &mut Vec<W>, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        for element in value {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                AnyIdentifiableWidget::foreach_child(element, f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev(value: &mut Vec<W>, f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        for element in &mut value.iter_mut().rev() {
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
        impl<T: PartialEq + StateContract, $($generic: IdentifiableWidget<T>),*> ReverseAnySequence<($($generic),*)> for dyn AnyIdentifiableWidget<T> {
            fn index(value: &mut ($($generic),*), index: usize) -> &mut dyn AnyIdentifiableWidget<T> {
                let ($($generic),*) = value;

                let mut passed = 0;

                $(
                    if $generic.is_ignore() {} else if $generic.is_proxy() {
                        let child_count = $generic.child_count();
                        if index < passed + child_count {
                            return AnyIdentifiableWidget::child($generic, index - passed);
                        }

                        passed += child_count;
                    } else {
                        if index == passed {
                            return $generic;
                        }

                        passed += 1;
                    }
                )*

                panic!("Index out of bounds. Index: {}, Passed: {}", index, passed);
            }

            fn count(value: &mut ($($generic),*)) -> usize {
                let ($($generic),*) = value;

                let mut count = 0;

                $(
                    if $generic.is_ignore() {} else if $generic.is_proxy() {
                        count += $generic.child_count();
                    } else {
                        count += 1;
                    }
                )*

                count
            }

            fn foreach(value: &mut ($($generic),*), f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
                let ($($generic),*) = value;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        AnyIdentifiableWidget::foreach_child($generic, f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_rev(value: &mut ($($generic),*), f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
                let reverse!([$($generic)*]) = value;
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
//tuple_sequence_impl!(W1, W2);
//tuple_sequence_impl!(W1, W2, W3);
//tuple_sequence_impl!(W1, W2, W3, W4);
//tuple_sequence_impl!(W1, W2, W3, W4, W5);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12);

#[allow(non_snake_case)]
#[allow(unused_parens)]
impl<T: PartialEq + StateContract, W1: IdentifiableWidget<T>> ReverseAnySequence<(   W1   )> for dyn AnyIdentifiableWidget<T> {
    fn index(value: &mut (   W1   ), index: usize) -> &mut dyn AnyIdentifiableWidget<T> {
        let (   W1   ) = value;

        let mut passed = 0;

        if W1.is_ignore() {} else if W1.is_proxy() {
            let child_count = W1.child_count();
            if index < passed + child_count {
                return AnyIdentifiableWidget::child(W1, index - passed);
            }

            passed += child_count;
        } else {
            if index == passed {
                return W1;
            }

            passed += 1;
        }

        panic!("Index out of bounds. Index: {}, Passed: {}", index, passed);
    }

    fn count(value: &mut (   W1   )) -> usize {
        let (   W1   ) = value;

        let mut count = 0;

        if W1.is_ignore() {} else if W1.is_proxy() {
            count += W1.child_count();
        } else {
            count += 1;
        }

        count
    }

    fn foreach(value: &mut (   W1   ), f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        let (   W1   ) = value;
        if W1.is_ignore() {} else if W1.is_proxy() {
            AnyIdentifiableWidget::foreach_child(W1, f);
        } else {
            f(W1);
        }
    }

    fn foreach_rev(value: &mut (   W1   ), f: &mut dyn FnMut(&mut dyn AnyIdentifiableWidget<T>)) {
        let (   W1    ) = value;
        if W1.is_ignore() {} else if W1.is_proxy() {
            AnyIdentifiableWidget::foreach_child_rev(W1, f);
        } else {
            f(W1);
        }
    }
}