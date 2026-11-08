use std::fmt::Debug;

use crate::controller::DatasetController;

/// The controllers in a chart: a single controller, or a tuple of up to 12 controllers.
/// Controllers are visited in sequence order, which is also the order they are drawn in.
pub trait ControllerSequence: Debug + Clone + 'static {
    fn controllers(&self, f: &mut dyn FnMut(&dyn DatasetController));
}

impl<C: DatasetController + Clone> ControllerSequence for C {
    fn controllers(&self, f: &mut dyn FnMut(&dyn DatasetController)) {
        f(self);
    }
}

macro_rules! tuple_sequence_impl {
    ($($generic:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($generic: DatasetController + Clone),*> ControllerSequence for ($($generic,)*) {
            fn controllers(&self, f: &mut dyn FnMut(&dyn DatasetController)) {
                let ($($generic,)*) = self;
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
