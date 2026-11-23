//! Categories named by the data: text coordinates placed on a category scale, in the order the
//! data first names them, after the scale's own labels.

use carbide_chart::{Axis, BarController, BoxDataSetExt, BoxStats, CategoryScale, Chart, DatasetExt, Estimate, LineController, RangeDataSetExt, ScaleId, ScatterController, StackId};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Votes for six colours, each bar named by its colour, with no scale set. Pins down that text
/// makes the default x scale a category scale, with a category per name in the order of the
/// data, labelled with the names.
#[test]
fn named_by_the_data() {
    let votes = vec![("Red", 12.0), ("Blue", 19.0), ("Yellow", 3.0), ("Green", 5.0), ("Purple", 2.0), ("Orange", 3.0)];

    let chart = Chart::new(BarController::new(votes)).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("named_by_the_data");
}

/// Two lines naming weekdays: the second names one the first does, and one it doesn't. Pins down
/// that categories are shared by name across datasets, numbered in the order they are first named
/// (Mon, Tue, Wed, then Thu), so the second line's points sit on Tue and Thu.
#[test]
fn shared_by_datasets_in_first_seen_order() {
    let first = vec![("Mon", 3.0), ("Tue", 5.0), ("Wed", 2.0)];
    let second = vec![("Tue", 1.0), ("Thu", 4.0)];

    let chart = Chart::new(LineController::new((first, second))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("shared_by_datasets_in_first_seen_order");
}

/// A category scale with labels of its own, and bars naming one of them and two it doesn't have.
/// Pins down that the labels come first and keep their places, a bar naming a label sits on it,
/// and names the data adds follow the labels in the order they are named.
#[test]
fn after_the_scale_labels() {
    let levels = vec![("High", 7.0), ("Extreme", 9.0), ("Unknown", 1.0)];

    let chart = Chart::new(BarController::new(levels))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(["Low", "Medium", "High"]))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("after_the_scale_labels");
}

/// Horizontal bars, stacked, naming their categories, with the second dataset naming them in another
/// order and one the first doesn't. Pins down that the categories go on the y scale, from the top,
/// and that bars stack by category name, not by their position in their dataset.
#[test]
fn stacked_and_horizontal() {
    const COSTS: StackId = StackId::new("costs");

    let materials = vec![("Kitchen", 12.0), ("Bath", 8.0), ("Garden", 5.0)];
    let labour = vec![("Garden", 6.0), ("Kitchen", 9.0), ("Garage", 4.0)];

    let chart = Chart::new(BarController::new((materials.stack(COSTS), labour.stack(COSTS))).horizontal(true)).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_and_horizontal");
}

/// Scatter points whose y names a status, at times along x. Pins down that text works on the y
/// scale too: a vertical category scale, first named at the top, and points placed on their
/// status.
#[test]
fn naming_the_y_value() {
    let checks = vec![(1.0, "Up"), (2.0, "Up"), (3.0, "Degraded"), (4.0, "Down"), (5.0, "Degraded"), (6.0, "Up")];

    let chart = Chart::new(ScatterController::new(checks)).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("naming_the_y_value");
}

/// Marks on named categories: box plots and bars with error bars, both naming their weekdays,
/// one in another order. Pins down that marks place their points through the categories as bars
/// do, each in its own share of its category's slot.
#[test]
fn marks_on_named_categories() {
    let waits = vec![
        BoxStats { x: "Mon", min: 2.0, q1: 4.0, median: 5.0, q3: 7.0, max: 11.0 },
        BoxStats { x: "Tue", min: 3.0, q1: 5.0, median: 6.0, q3: 8.0, max: 12.0 },
        BoxStats { x: "Wed", min: 1.0, q1: 3.0, median: 4.0, q3: 6.0, max: 9.0 },
    ];
    let measured = vec![
        Estimate { x: "Wed", y: 6.0, low: 5.0, high: 7.5 },
        Estimate { x: "Mon", y: 8.0, low: 6.5, high: 9.0 },
        Estimate { x: "Tue", y: 7.0, low: 6.0, high: 8.0 },
    ];

    let chart = Chart::new(BarController::new((waits.box_plot(), measured.error_bars()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("marks_on_named_categories");
}
