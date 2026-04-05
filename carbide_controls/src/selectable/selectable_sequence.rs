use crate::identifiable::{AnySelectableWidget, SelectableWidget};
use carbide::reverse;
use carbide::widget::{CommonWidget, ReverseAnySequence};

impl<W: SelectableWidget> ReverseAnySequence<Vec<W>> for dyn AnySelectableWidget {
    fn index(value: &mut Vec<W>, index: usize) -> &mut dyn AnySelectableWidget {
        let mut passed = 0;

        for element in value.iter_mut() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                let child_count = element.child_count();

                if index < passed + child_count {
                    return AnySelectableWidget::child(element, index - passed);
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

    fn foreach(value: &mut Vec<W>, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        for element in value {
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

    fn foreach_rev(value: &mut Vec<W>, f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
        for element in &mut value.iter_mut().rev() {
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
        impl<$($generic: SelectableWidget),*> ReverseAnySequence<($($generic),*)> for dyn AnySelectableWidget {
            fn index(value: &mut ($($generic),*), index: usize) -> &mut dyn AnySelectableWidget {
                let ($($generic),*) = value;

                let mut passed = 0;

                $(
                    if $generic.is_ignore() {} else if $generic.is_proxy() {
                        let child_count = $generic.child_count();
                        if index < passed + child_count {
                            return AnySelectableWidget::child($generic, index - passed);
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

            fn foreach(value: &mut ($($generic),*), f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
                let ($($generic),*) = value;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        AnySelectableWidget::foreach_child($generic, f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_rev(value: &mut ($($generic),*), f: &mut dyn FnMut(&mut dyn AnySelectableWidget)) {
                let reverse!([$($generic)*]) = value;
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

tuple_sequence_impl!(W1);
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