//! Stacking shared by the bar and line controllers: which datasets stack together, and how far
//! each stack has reached.

use std::collections::HashMap;
use carbide::draw::Scalar;
use crate::{DataSetOptions, StackId};
use crate::controller::orientation::Orientation;
use crate::mark::AsDataSet;
use crate::scale::{Coordinates, Places, ScaleId};
use crate::sequence::Sequence;

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
    /// For normalised stacks, the sum of the absolute values at each x (see `normalized_totals`),
    /// which each value is scaled by.
    totals: NormalizedTotals,
}

/// The sum of the absolute values of each normalised stack at each x value (see
/// `StackId::normalized`).
pub(crate) type NormalizedTotals = HashMap<(StackKey, u64), Scalar>;

impl StackSums {
    /// Sums positive and negative values apart, so each grows away from zero on its own side,
    /// as bars stack.
    pub(crate) fn split_by_sign() -> StackSums {
        StackSums { sums: HashMap::new(), split_by_sign: true, totals: HashMap::new() }
    }

    /// One running sum per stack, as lines stack: each line is drawn at the total of the
    /// datasets up to it, so the bands between neighbouring lines never have gaps.
    pub(crate) fn single() -> StackSums {
        StackSums { sums: HashMap::new(), split_by_sign: false, totals: HashMap::new() }
    }

    /// Scales the values of normalised stacks by `totals` (see `normalized_totals`), so each such
    /// stack sums to 100 at every x.
    pub(crate) fn normalized(mut self, totals: NormalizedTotals) -> StackSums {
        self.totals = totals;
        self
    }

    /// Adds `y` to the stack `key` at `x`, and returns where it starts and ends: from the stack's
    /// sum so far (on `y`'s side, when split by sign) to that sum plus `y`. A dataset on its own
    /// starts at zero.
    pub(crate) fn add(&mut self, key: &StackKey, x: Scalar, y: Scalar) -> (Scalar, Scalar) {
        let y = self.share(key, x, y);
        let side = !self.split_by_sign || y >= 0.0;
        // Adding 0.0 turns -0.0 into 0.0, so both find the same stack.
        let sum = self.sums.entry((key.clone(), (x + 0.0).to_bits(), side)).or_insert(0.0);
        let start = *sum;
        *sum += y;
        (start, *sum)
    }
}

impl StackSums {
    /// `y` as drawn in its stack: its share of 100 in a normalised stack, itself otherwise. A
    /// stack whose values at `x` are all zero has nothing to share, and is drawn at zero.
    fn share(&self, key: &StackKey, x: Scalar, y: Scalar) -> Scalar {
        let StackKey::Stacked { stack, .. } = key else {
            return y;
        };
        if !stack.is_normalized() {
            return y;
        }

        match self.totals.get(&(key.clone(), (x + 0.0).to_bits())) {
            Some(total) if *total > 0.0 => y / total * 100.0,
            _ => 0.0,
        }
    }
}

/// The sum of the absolute values of each normalised stack at each x, over the datasets `counted`
/// accepts, each read as `orientation` reads it (see `Orientation::for_dataset`), for
/// `StackSums::normalized`. Empty, without reading any points, when no dataset is in a normalised
/// stack.
pub(crate) fn normalized_totals<T, D>(datasets: &D, places: &dyn Places, orientation: Orientation, counted: &dyn Fn(usize) -> bool) -> NormalizedTotals
where
    T: AsDataSet + ?Sized,
    D: Sequence<T>,
{
    let mut totals = HashMap::new();

    datasets.foreach(&mut |dataset_index, dataset| {
        if !counted(dataset_index) {
            return;
        }

        let dataset = dataset.as_dataset();
        let options = dataset.options();
        // A hidden dataset adds nothing to the bounds, its stack or what is drawn.
        if !options.visible {
            return;
        }
        if !options.stack.as_ref().is_some_and(StackId::is_normalized) {
            return;
        }

        let key = StackKey::new(&options, dataset_index);
        let (index_id, value_id) = orientation.for_dataset(&options, places).index_and_value_scales(&options);
        let coordinates = Coordinates::new(places, index_id, value_id);

        dataset.points(&mut |_index, point| {
            if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
                *totals.entry((key.clone(), (x + 0.0).to_bits())).or_insert(0.0) += y.abs();
            }
        });
    });

    totals
}

/// How far stacking moves the bars or line of the datasets whose overlay marks need it, such as
/// error bars, by dataset index and x value: where the bar ends or the line is drawn, less the
/// point's own value. Datasets and x values that aren't in it are moved by zero: those that don't
/// stack, and those without an overlay mark, as nothing else reads it. A chart without stacked
/// overlays leaves it empty, which allocates nothing.
#[derive(Debug, Default)]
pub(crate) struct StackOffsets {
    by_dataset: HashMap<usize, HashMap<u64, Scalar>>,
}

impl StackOffsets {
    /// Records that stacking moves the dataset at `dataset_index` by `offset` at `x`.
    pub(crate) fn record(&mut self, dataset_index: usize, x: Scalar, offset: Scalar) {
        // Adding 0.0 turns -0.0 into 0.0, as for the sums.
        self.by_dataset.entry(dataset_index).or_default().insert((x + 0.0).to_bits(), offset);
    }

    /// The offsets of one dataset, or `None` when it isn't moved.
    pub(crate) fn dataset(&self, dataset_index: usize) -> Option<&HashMap<u64, Scalar>> {
        self.by_dataset.get(&dataset_index)
    }

    /// Whether no dataset is moved.
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.by_dataset.is_empty()
    }
}

/// How far stacking moves a dataset at `x`, given its offsets (see `StackOffsets::dataset`).
pub(crate) fn stack_offset(offsets: Option<&HashMap<u64, Scalar>>, x: Scalar) -> Scalar {
    offsets.and_then(|offsets| offsets.get(&(x + 0.0).to_bits())).copied().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::StackId;
    use crate::scale::ScaleId;
    use super::{StackKey, StackSums};

    /// In a normalised stack each value is its share of the absolute values' total at its x, so a
    /// stack sums to 100, its negative values below zero; a stack of zeros is drawn at zero.
    #[test]
    fn normalised_stacks_share_100() {
        let key = StackKey::Stacked { x_scale: ScaleId::X, y_scale: ScaleId::Y, stack: StackId::new("parts").normalized() };
        let totals = HashMap::from([((key.clone(), 0.0f64.to_bits()), 40.0), ((key.clone(), 1.0f64.to_bits()), 0.0)]);

        let mut sums = StackSums::split_by_sign().normalized(totals);
        assert_eq!(sums.add(&key, 0.0, 10.0), (0.0, 25.0));
        assert_eq!(sums.add(&key, 0.0, 20.0), (25.0, 75.0));
        assert_eq!(sums.add(&key, 0.0, -10.0), (0.0, -25.0), "a negative value takes its share below zero");
        assert_eq!(sums.add(&key, 1.0, 0.0), (0.0, 0.0));

        let plain = StackKey::Stacked { x_scale: ScaleId::X, y_scale: ScaleId::Y, stack: StackId::new("parts") };
        assert_eq!(sums.add(&plain, 0.0, 10.0), (0.0, 10.0), "the same name without normalising is another stack");
    }

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
