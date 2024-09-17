use std::fmt::Debug;
use crate::{DataPoint, DataSet, DataValue};

pub trait DataSetSequence: Debug + Clone + 'static {
    type X: DataValue;
    type Y: DataValue;
    type Z: DataValue;
    fn min(&self) -> (Self::X, Self::Y, Self::Z);
    fn max(&self) -> (Self::X, Self::Y, Self::Z);

    fn foreach<F: FnMut(usize, &dyn DataPoint<X=Self::X, Y=Self::Y, Z=Self::Z>)>(&self, f: F);
}

impl<X: DataValue + PartialOrd, Y: DataValue + PartialOrd, Z: DataValue + PartialOrd, T> DataSetSequence for Vec<T>
    where
        T: DataSet,
        for<'a> <T as DataSet>::Item<'a>: DataPoint<X=X, Y=Y, Z=Z>
{
    type X = X;
    type Y = Y;
    type Z = Z;

    fn min(&self) -> (Self::X, Self::Y, Self::Z) {
        let mut iter = self.iter();
        if let Some(next) = iter.next() {
            let (mut min_x, mut min_y, mut min_z) = next.min();

            for item in iter {
                let (x, y, z) = item.min();

                min_x = if PartialOrd::lt(&min_x, &x) { min_x } else { x };
                min_y = if PartialOrd::lt(&min_y, &y) { min_y } else { y };
                min_z = if PartialOrd::lt(&min_z, &z) { min_z } else { z };
            }

            (min_x, min_y, min_z)
        } else {
            (X::zero(), Y::zero(), Z::zero())
        }
    }

    fn max(&self) -> (Self::X, Self::Y, Self::Z) {
        let mut iter = self.iter();
        if let Some(next) = iter.next() {
            let (mut max_x, mut max_y, mut max_z) = next.max();

            for item in iter {
                let (x, y, z) = item.max();

                max_x = if PartialOrd::gt(&max_x, &x) { max_x } else { x };
                max_y = if PartialOrd::gt(&max_y, &y) { max_y } else { y };
                max_z = if PartialOrd::gt(&max_z, &z) { max_z } else { z };
            }

            (max_x, max_y, max_z)
        } else {
            (X::one(), Y::one(), Z::one())
        }
    }

    fn foreach<F: FnMut(usize, &dyn DataPoint<X=X, Y=Y, Z=Z>)>(&self, mut f: F) {
        for (index, items) in self.iter().enumerate() {
            for point in items.points() {
                f(index, &point)
            }
        }
    }
}

macro_rules! tuple_sequence_impl {
    ($first_generic:ident $(, $generic:ident)*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<X: DataValue + PartialOrd, Y: DataValue + PartialOrd, Z: DataValue + PartialOrd, $first_generic $(, $generic)*> DataSetSequence for ($first_generic $(, $generic)*)
        where
                $first_generic: DataSet,
                for<'a> <$first_generic as DataSet>::Item<'a>: DataPoint<X=X, Y=Y, Z=Z>,
            $(
                $generic: DataSet,
                for<'a> <$generic as DataSet>::Item<'a>: DataPoint<X=X, Y=Y, Z=Z>,
            )*
        {
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

            fn foreach<F: FnMut(usize, &dyn DataPoint<X=X, Y=Y, Z=Z>)>(&self, mut f: F) {
                let ($first_generic $(, $generic)*) = self;
                let mut index = 0;
                for point in $first_generic.points() {
                        f(index, &point)
                    }
                    index += 1;

                $(
                    for point in $generic.points() {
                        f(index, &point)
                    }
                    index += 1;
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