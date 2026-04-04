use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use dyn_clone::{clone_box, clone_trait_object, DynClone};
use crate::state::{AnyReadState, StateExtNew, ValueState};
use crate::state::{Map1, ReadStateExtNew};
use crate::widget::{AnyWidget, Content, Widget};

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

    fn index_mut(&mut self, index: usize) -> &mut T;
    fn count(&mut self) -> usize;

    fn foreach_mut(&mut self, f: &mut dyn FnMut(&mut T));
    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut T));
}

clone_trait_object!(<T: ?Sized> AnySequence<T>);

pub trait Sequence<T=dyn AnyWidget>: AnySequence<T> + Clone where T: ?Sized {}

impl<T: ?Sized, W> Sequence<T> for W where W: AnySequence<T> + Clone {}

impl<T: ?Sized + 'static> AnySequence<T> for Box<dyn AnySequence<T>> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.deref_mut().index_mut(index)
    }

    fn count(&mut self) -> usize {
        self.deref_mut().count()
    }

    fn foreach_mut(&mut self, f: &mut dyn FnMut(&mut T)) {
        self.deref_mut().foreach_mut(f)
    }

    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut T)) {
        self.deref_mut().foreach_rev(f)
    }
}

impl<T: ?Sized> AnySequence<T> for () {
    fn index_mut(&mut self, index: usize) -> &mut T {
        panic!("Index out of bounds of empty sequence")
    }

    fn count(&mut self) -> usize {
        0
    }

    fn len(&self) -> Box<dyn AnyReadState<T=usize>>
    where
        Self: Clone,
    {
        ValueState::new(0).as_dyn()
    }

    fn foreach_mut(&mut self, _f: &mut dyn FnMut(&mut T)) {}
    fn foreach_rev(&mut self, _f: &mut dyn FnMut(&mut T)) {}
}

impl<W: Widget> AnySequence for Vec<W> {
    fn index_mut(&mut self, index: usize) -> &mut dyn AnyWidget {
        let mut passed = 0;

        for element in self.iter_mut() {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                let child_count = element.child_count();

                if index < passed + child_count {
                    return element.child_mut(index - passed);
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

    fn count(&mut self) -> usize {
        let mut count = 0;
        for element in self {
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

    fn foreach_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        for element in self {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                element.foreach_child_mut(f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        for element in &mut self.iter_mut().rev() {
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

impl<W: Widget> AnySequence for Content<W> {
    fn index_mut(&mut self, index: usize) -> &mut dyn AnyWidget {
        let mut passed = 0;

        for (_, element) in self.0.iter_mut().take(self.1) {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                let child_count = element.child_count();

                if index < passed + child_count {
                    return element.child_mut(index - passed);
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

    fn count(&mut self) -> usize {
        let mut count = 0;
        for (_, element) in self.0.iter_mut().take(self.1) {
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

    fn foreach_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        for (_, element) in self.0.iter_mut().take(self.1) {
            if element.is_ignore() {
                continue;
            }

            if element.is_proxy() {
                element.foreach_child_mut(f);
                continue;
            }

            f(element);
        }
    }

    fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        for (_, element) in self.0.iter_mut().take(self.1).rev() {
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
        impl<$($generic: Widget),*> AnySequence for ($($generic),*) {
            fn index_mut(&mut self, index: usize) -> &mut dyn AnyWidget {
                let ($($generic),*) = self;

                let mut passed = 0;

                $(
                    {
                        if $generic.is_ignore() {

                        } else if $generic.is_proxy() {
                            let child_count = $generic.child_count();
                            if index < passed + child_count {
                                return $generic.child_mut(index - passed);
                            }

                            passed += child_count;
                        } else {
                            if index == passed {
                                return $generic;
                            }

                            passed += 1;
                        }
                    }
                )*

                panic!("Index out of bounds. Index: {}, Passed: {}", index, passed);
            }

            fn count(&mut self) -> usize {
                let ($($generic),*) = self;

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

            fn foreach_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        $generic.foreach_child_mut(f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
                let reverse!([$($generic)*]) = self;
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