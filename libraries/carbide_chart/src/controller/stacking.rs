//! Stacking shared by the bar and line controllers: which datasets stack together, and how far
//! each stack has reached.

use std::collections::HashMap;
use carbide::draw::Scalar;
use crate::{DataSetOptions, StackId};
use crate::scale::ScaleId;

/// Which datasets stack together: those in one controller with the same stack on the same x and
/// y scales. A dataset without a stack stands alone.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum StackKey {
    Stacked { x_scale: ScaleId, y_scale: ScaleId, stack: StackId },
    Alone(usize),
}

impl StackKey {
    pub(crate) fn new(options: &DataSetOptions, dataset_index: usize) -> StackKey {
        match &options.stack {
            Some(stack) => StackKey::Stacked {
                x_scale: options.x_scale.clone(),
                y_scale: options.y_scale.clone(),
                stack: stack.clone(),
            },
            None => StackKey::Alone(dataset_index),
        }
    }
}

/// How far each stack has reached at each x value. Points are matched by x value, so a dataset
/// adds to a stack only at the x values where it has points.
pub(crate) struct StackSums {
    sums: HashMap<(StackKey, u64, bool), Scalar>,
    split_by_sign: bool,
}

impl StackSums {
    /// Sums positive and negative values apart, so each grows away from zero on its own side,
    /// as bars stack.
    pub(crate) fn split_by_sign() -> StackSums {
        StackSums { sums: HashMap::new(), split_by_sign: true }
    }

    /// One running sum per stack, as lines stack: each line is drawn at the total of the
    /// datasets up to it, so the bands between neighbouring lines never have gaps.
    pub(crate) fn single() -> StackSums {
        StackSums { sums: HashMap::new(), split_by_sign: false }
    }

    /// Adds `y` to the stack `key` at `x`, and returns where it starts and ends: from the stack's
    /// sum so far (on `y`'s side, when split by sign) to that sum plus `y`. A dataset on its own
    /// starts at zero.
    pub(crate) fn add(&mut self, key: &StackKey, x: Scalar, y: Scalar) -> (Scalar, Scalar) {
        let side = !self.split_by_sign || y >= 0.0;
        // Adding 0.0 turns -0.0 into 0.0, so both find the same stack.
        let sum = self.sums.entry((key.clone(), (x + 0.0).to_bits(), side)).or_insert(0.0);
        let start = *sum;
        *sum += y;
        (start, *sum)
    }
}

#[cfg(test)]
mod tests {
    use super::{StackKey, StackSums};

    /// Split by sign, a negative value starts from the negative sums, not from the top of the
    /// positive ones; with a single sum, it subtracts from the total.
    #[test]
    fn sums_split_by_sign_or_run_together() {
        let key = StackKey::Alone(0);

        let mut split = StackSums::split_by_sign();
        assert_eq!(split.add(&key, 0.0, 10.0), (0.0, 10.0));
        assert_eq!(split.add(&key, 0.0, -4.0), (0.0, -4.0));
        assert_eq!(split.add(&key, 0.0, 5.0), (10.0, 15.0));

        let mut single = StackSums::single();
        assert_eq!(single.add(&key, 0.0, 10.0), (0.0, 10.0));
        assert_eq!(single.add(&key, 0.0, -4.0), (10.0, 6.0));
        assert_eq!(single.add(&key, -0.0, 5.0), (6.0, 11.0), "-0.0 and 0.0 are the same x");
        assert_eq!(single.add(&key, 1.0, 5.0), (0.0, 5.0), "each x value has its own sum");
    }
}
