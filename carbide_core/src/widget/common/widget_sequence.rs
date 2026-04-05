use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use dyn_clone::{clone_box, clone_trait_object, DynClone};
use crate::state::{AnyReadState, StateExtNew, ValueState};
use crate::state::{Map1, ReadStateExtNew};
use crate::widget::{AnyWidget, Widget};

// Created to fix rust E210 in foreign crates, because when we flip the sequence, we can implement it for dyn LocalTrait
pub trait ReverseAnySequence<T> {
    fn index(value: &mut T, index: usize) -> &mut Self;
    fn count(value: &mut T) -> usize;

    fn foreach(value: &mut T, f: &mut dyn FnMut(&mut Self));
    fn foreach_rev(value: &mut T, f: &mut dyn FnMut(&mut Self));
}

impl<T: ?Sized, U: Debug + Clone + 'static> AnySequence<T> for U where T: ReverseAnySequence<U> {
    fn index(&mut self, index: usize) -> &mut T {
        <T as ReverseAnySequence<U>>::index(self, index)
    }

    fn count(&mut self) -> usize {
        <T as ReverseAnySequence<U>>::count(self)
    }

    fn foreach(&mut self, f: &mut dyn FnMut(&mut T)) {
        <T as ReverseAnySequence<U>>::foreach(self, f)
    }

    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut T)) {
        <T as ReverseAnySequence<U>>::foreach_rev(self, f)
    }
}

pub trait AnySequence<T=dyn AnyWidget>: Debug + DynClone + 'static where T: ?Sized {
    fn len(&self) -> Box<dyn AnyReadState<T=usize>> where Self: Clone {
        /*let mut s = clone_box(self);

        Map1::read_map(0, move |_| {
            let mut count = 0;
            s.foreach_mut(&mut |_| {
                count += 1;
            });

            count
        }).as_dyn_read()*/

        todo!()
    }

    fn index(&mut self, index: usize) -> &mut T;
    fn count(&mut self) -> usize;

    fn foreach(&mut self, f: &mut dyn FnMut(&mut T));
    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut T));
}

clone_trait_object!(<T: ?Sized> AnySequence<T>);

pub trait Sequence<T=dyn AnyWidget>: AnySequence<T> + Clone where T: ?Sized {}

impl<T: ?Sized, W> Sequence<T> for W where W: AnySequence<T> + Clone {}

impl<T: ?Sized + 'static> ReverseAnySequence<Box<dyn AnySequence<T>>> for T {
    fn index(value: &mut Box<dyn AnySequence<T>>, index: usize) -> &mut T {
        value.deref_mut().index(index)
    }

    fn count(value: &mut Box<dyn AnySequence<T>>) -> usize {
        value.deref_mut().count()
    }

    fn foreach(value: &mut Box<dyn AnySequence<T>>, f: &mut dyn FnMut(&mut T)) {
        value.deref_mut().foreach(f)
    }

    fn foreach_rev(value: &mut Box<dyn AnySequence<T>>, f: &mut dyn FnMut(&mut T)) {
        value.deref_mut().foreach_rev(f)
    }
}

impl<T: ?Sized> ReverseAnySequence<()> for T {
    fn index(_: &mut (), _: usize) -> &mut Self {
        panic!("Index out of bounds of empty sequence")
    }

    fn count(_: &mut ()) -> usize {
        0
    }

    fn foreach(_: &mut (), _: &mut dyn FnMut(&mut Self)) {}
    fn foreach_rev(_: &mut (), _: &mut dyn FnMut(&mut Self)) {}
}

impl<W: Widget> ReverseAnySequence<Vec<W>> for dyn AnyWidget {
    fn index(value: &mut Vec<W>, index: usize) -> &mut dyn AnyWidget {
        let mut passed = 0;

        for element in value.iter_mut() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                let child_count = element.child_count();

                if index < passed + child_count {
                    return element.child(index - passed);
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

    fn foreach(value: &mut Vec<W>, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        for element in value {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                element.foreach_child(f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev(value: &mut Vec<W>, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        for element in &mut value.iter_mut().rev() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                element.foreach_child_rev(f);
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
        impl<$($generic: Widget),*> ReverseAnySequence<($($generic),*)> for dyn AnyWidget {
            fn index(value: &mut ($($generic),*), index: usize) -> &mut dyn AnyWidget {
                let ($($generic),*) = value;

                let mut passed = 0;

                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        let child_count = $generic.child_count();
                        if index < passed + child_count {
                            return $generic.child(index - passed);
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
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        count += $generic.child_count();
                    } else {
                        count += 1;
                    }
                )*

                count
            }

            fn foreach(value: &mut ($($generic),*), f: &mut dyn FnMut(&mut dyn AnyWidget)) {
                let ($($generic),*) = value;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        $generic.foreach_child(f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_rev(value: &mut ($($generic),*), f: &mut dyn FnMut(&mut dyn AnyWidget)) {
                let reverse!([$($generic)*]) = value;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        $generic.foreach_child_rev(f);
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
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15, W16);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15, W16, W17);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15, W16, W17, W18);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15, W16, W17, W18, W19);
//tuple_sequence_impl!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15, W16, W17, W18, W19, W20);