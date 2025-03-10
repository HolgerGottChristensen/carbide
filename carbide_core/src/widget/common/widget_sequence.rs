use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use dyn_clone::{clone_box, clone_trait_object, DynClone};
use carbide::state::AnyReadState;
use crate::state::{Map1, ReadStateExtNew};
use crate::widget::{AnyWidget, Content, Widget};

pub trait AnySequence<T=dyn AnyWidget>: Debug + DynClone + 'static where T: ?Sized {
    fn len(&self) -> Box<dyn AnyReadState<T=usize>> where Self: Clone {
        let s = clone_box(self);

        Map1::read_map(0, move |_| {
            let mut count = 0;
            s.foreach(&mut |_| {
                count += 1;
            });

            count
        }).as_dyn_read()
    }

    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a T));
    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut T));
    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut T));
    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut T));
    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut T));
}

clone_trait_object!(<T: ?Sized> AnySequence<T>);

pub trait Sequence<T=dyn AnyWidget>: AnySequence<T> + Clone where T: ?Sized {}

impl<T: ?Sized, W> Sequence<T> for W where W: AnySequence<T> + Clone {}

impl<T: ?Sized + 'static> AnySequence<T> for Box<dyn AnySequence<T>> {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a T)) {
        self.deref().foreach(f)
    }

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut T)) {
        self.deref_mut().foreach_mut(f)
    }

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut T)) {
        self.deref_mut().foreach_rev(f)
    }

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut T)) {
        self.deref_mut().foreach_direct(f)
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut T)) {
        self.deref_mut().foreach_direct_rev(f)
    }
}

impl<T: ?Sized> AnySequence<T> for () {
    fn foreach<'a>(&'a self, _f: &mut dyn FnMut(&'a T)) {}
    fn foreach_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut T)) {}
    fn foreach_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut T)) {}
    fn foreach_direct<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut T)) {}
    fn foreach_direct_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut T)) {}
}

impl<W: Widget> AnySequence for Vec<W> {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        for element in self {
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

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
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

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
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

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for element in &mut self.iter_mut() {
            f(element);
        }
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for element in &mut self.iter_mut().rev() {
            f(element);
        }
    }
}

impl<W: Widget> AnySequence for Content<W> {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        for (_, element) in self.0.iter().take(self.1) {
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

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
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

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
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

    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, element) in self.0.iter_mut().take(self.1) {
            f(element);
        }
    }

    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, element) in self.0.iter_mut().take(self.1).rev() {
            f(element);
        }
    }
}

macro_rules! tuple_sequence_impl {
    ($($generic:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<$($generic: Widget),*> AnySequence for ($($generic),*) {
            fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
                let ($($generic),*) = self;
                $(
                    if $generic.is_ignore() {

                    } else if $generic.is_proxy() {
                        $generic.foreach_child(f);
                    } else {
                        f($generic);
                    }
                )*
            }

            fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
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

            fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
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

            fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
                let ($($generic),*) = self;
                $(
                    f($generic);
                )*
            }

            fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
                let reverse!([$($generic)*]) = self;
                $(
                    f($generic);
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