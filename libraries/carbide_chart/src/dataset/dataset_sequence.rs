use std::fmt::Debug;
use crate::{DataSet, DataValue};

pub trait DataSetSequence: Debug + 'static {
    type X: DataValue;
    type Y: DataValue;
    type Z: DataValue;

    /// The smallest x, y and z over every dataset, or `None` when no dataset has points.
    fn min(&self) -> Option<(Self::X, Self::Y, Self::Z)>;
    /// The largest x, y and z over every dataset, or `None` when no dataset has points.
    fn max(&self) -> Option<(Self::X, Self::Y, Self::Z)>;

    fn datasets<'a, F: FnMut(usize, &'a dyn DataSet<X=Self::X, Y=Self::Y, Z=Self::Z>)>(&'a self, f: F) where Self::X: 'a, Self::Y: 'a, Self::Z: 'a;
}

#[derive(Copy, Clone)]
enum Extreme {
    Min,
    Max,
}

fn pick<T: PartialOrd>(current: T, other: T, extreme: Extreme) -> T {
    let keep_current = match extreme {
        Extreme::Min => current < other,
        Extreme::Max => current > other,
    };

    if keep_current { current } else { other }
}

/// Combines two optional (x, y, z) extremes component-wise. A `None` side (an empty dataset)
/// contributes nothing.
fn combine<X, Y, Z>(current: Option<(X, Y, Z)>, other: Option<(X, Y, Z)>, extreme: Extreme) -> Option<(X, Y, Z)>
where
    X: DataValue + PartialOrd,
    Y: DataValue + PartialOrd,
    Z: DataValue + PartialOrd,
{
    match (current, other) {
        (Some((cx, cy, cz)), Some((ox, oy, oz))) => Some((pick(cx, ox, extreme), pick(cy, oy, extreme), pick(cz, oz, extreme))),
        (current, None) => current,
        (None, other) => other,
    }
}

macro_rules! tuple_sequence_impl {
    ($first_generic:ident $(, $generic:ident)*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        #[allow(unused_mut)]
        impl<X: DataValue + PartialOrd, Y: DataValue + PartialOrd, Z: DataValue + PartialOrd, $first_generic: DataSet<X=X, Y=Y, Z=Z> $(, $generic: DataSet<X=X, Y=Y, Z=Z>)*> DataSetSequence for ($first_generic $(, $generic)*) {
            type X = X;
            type Y = Y;
            type Z = Z;

            fn min(&self) -> Option<(Self::X, Self::Y, Self::Z)> {
                let ($first_generic $(, $generic)*) = self;
                let mut min = $first_generic.min();

                $(
                    min = combine(min, $generic.min(), Extreme::Min);
                )*

                min
            }

            fn max(&self) -> Option<(Self::X, Self::Y, Self::Z)> {
                let ($first_generic $(, $generic)*) = self;
                let mut max = $first_generic.max();

                $(
                    max = combine(max, $generic.max(), Extreme::Max);
                )*

                max
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