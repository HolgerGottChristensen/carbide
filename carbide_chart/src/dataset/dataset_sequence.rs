use std::fmt::Debug;
use crate::{DataPoint, DataSet, DataValue};

pub trait DataSetSequence: Debug + 'static {
    type X: DataValue;
    type Y: DataValue;
    type Z: DataValue;

    fn min(&self) -> (Self::X, Self::Y, Self::Z);
    fn max(&self) -> (Self::X, Self::Y, Self::Z);

    fn datasets<'a, F: FnMut(usize, &'a dyn DataSet<X=Self::X, Y=Self::Y, Z=Self::Z>)>(&'a self, f: F) where Self::X: 'a, Self::Y: 'a, Self::Z: 'a;
}

macro_rules! tuple_sequence_impl {
    ($first_generic:ident $(, $generic:ident)*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<X: DataValue + PartialOrd, Y: DataValue + PartialOrd, Z: DataValue + PartialOrd, $first_generic: DataSet<X=X, Y=Y, Z=Z> $(, $generic: DataSet<X=X, Y=Y, Z=Z>)*> DataSetSequence for ($first_generic $(, $generic)*) {
            type X = X;
            type Y = Y;
            type Z = Z;

            fn min(&self) -> (Self::X, Self::Y, Self::Z) {
                let ($first_generic $(, $generic)*) = self;
                let (mut min_x, mut min_y, mut min_z) = $first_generic.min();

                $(
                    let (x, y, z) = $generic.min();
                    min_x = if PartialOrd::lt(&min_x, &x) { min_x } else { x };
                    min_y = if PartialOrd::lt(&min_y, &y) { min_y } else { y };
                    min_z = if PartialOrd::lt(&min_z, &z) { min_z } else { z };
                )*

                (min_x, min_y, min_z)
            }

            fn max(&self) -> (Self::X, Self::Y, Self::Z) {
                let ($first_generic $(, $generic)*) = self;
                let (mut max_x, mut max_y, mut max_z) = $first_generic.max();

                $(
                    let (x, y, z) = $generic.max();
                    max_x = if PartialOrd::gt(&max_x, &x) { max_x } else { x };
                    max_y = if PartialOrd::gt(&max_y, &y) { max_y } else { y };
                    max_z = if PartialOrd::gt(&max_z, &z) { max_z } else { z };
                )*

                (max_x, max_y, max_z)
            }

            fn datasets<'a, F: FnMut(usize, &'a dyn DataSet<X=Self::X, Y=Self::Y, Z=Self::Z>)>(&'a self, mut f: F) where Self::X: 'a, Self::Y: 'a, Self::Z: 'a {
                let ($first_generic $(, $generic)*) = self;
                let mut index = 0;

                f(index, $first_generic);

                $(
                    index += 1;
                    f(index, $generic);
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