use crate::node3d::{AnyNode3d, Node3d};
use std::fmt::Debug;

pub trait Node3dSequence: Clone + Debug + 'static {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d));
    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d));
    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d));
}

impl Node3dSequence for () {
    fn foreach<'a>(&'a self, _f: &mut dyn FnMut(&'a dyn AnyNode3d)) {}
    fn foreach_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}
    fn foreach_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {}
}

impl<W: Node3d + 'static> Node3dSequence for Vec<W> {
    fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d)) {
        for element in self {
            f(element);
        }
    }

    fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
        for element in self {
            f(element);
        }
    }

    fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
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
        impl<$($generic: Node3d),*> Node3dSequence for ($($generic),*) {
            fn foreach<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyNode3d)) {
                let ($($generic),*) = self;
                $(
                    f($generic);
                )*
            }

            fn foreach_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
                let ($($generic),*) = self;
                $(
                    f($generic);
                )*
            }

            fn foreach_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyNode3d)) {
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