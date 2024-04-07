use std::fmt::Debug;

use crate::widget::{AnyWidget, Widget};

pub trait WidgetSequence: Clone + Debug + 'static {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget));
    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget));
    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget));
    fn foreach_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget));
    fn foreach_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget));
}

impl WidgetSequence for () {
    fn foreach<'a>(&'a self, _f: &mut dyn FnMut(&'a dyn AnyWidget)) {}
    fn foreach_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}
    fn foreach_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}
    fn foreach_direct<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}
    fn foreach_direct_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}
}

impl<W: Widget + 'static> WidgetSequence for Vec<W> {
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

macro_rules! reverse {
    ([] $($reversed:tt)*) => {
        ($($reversed),*)  // base case
    };
    ([$first:tt $($rest:tt)*] $($reversed:tt)*) => {
        reverse!([$($rest)*] $first $($reversed)*)  // recursion
    };
}

macro_rules! tuple_sequence_impl {
    ($($generic:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<$($generic: Widget),*> WidgetSequence for ($($generic),*) {
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