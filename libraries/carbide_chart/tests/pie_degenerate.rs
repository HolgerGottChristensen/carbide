//! Pies with too little to draw, or parts too small to see: no data, nothing to share, a single
//! part, parts swallowed by spacing, rings swallowed by ring spacing, a chart too small for its
//! labels, repeated names, and only negative values.

use carbide_chart::{Axis, CategoryScale, Chart, LegendPosition, PieController, ScaleId, SizedDataSetExt};
use carbide_core::draw::{Dimension, Scalar};
use carbide_testing::Harness;

mod common;

use common::Framed;

/// An empty pie, and a pie whose values are all zero or missing, each with a legend. Pins down
/// that they render without panicking and draw nothing: no slices, no labels or leader lines, and
/// for the empty one no legend entries; the parts of the zeros, which have names, are still listed.
#[test]
fn nothing_to_share() {
    let empty: Vec<(&str, Scalar)> = vec![];

    let chart = Chart::new(PieController::new(empty))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("empty");

    let nothing: Vec<(&str, Option<Scalar>)> = vec![("Red", Some(0.0)), ("Blue", None), ("Green", Some(0.0))];

    let chart = Chart::new(PieController::new(nothing))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("zeros_and_missing");
}

/// A pie of one part, with spacing set, and a doughnut of one part. Pins down that a single part
/// is a whole circle, closed where it starts, with no gap taken out of it, as spacing only goes
/// between parts; and that its one label leaves the middle of its slice, half a turn round, at the
/// bottom.
#[test]
fn a_single_part() {
    let chart = Chart::new(PieController::new(vec![("Everything", 42.0)]).spacing(6.0))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("pie");

    let chart = Chart::new(PieController::new(vec![("Everything", 42.0)]))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).inner_radius(0.6))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("doughnut");
}

/// Parts too small to draw: a part of zero next to others, and, with spacing wider than it, a
/// sliver. Pins down that a part with no slice to show, of zero size or swallowed by the gaps
/// around it, gets no label or leader line pointing at nothing, while the parts around it are
/// drawn and labelled as usual.
#[test]
fn parts_too_small_to_draw() {
    let parts = vec![("Big", 60.0), ("Nothing", 0.0), ("Sliver", 0.2), ("Medium", 30.0), ("Small", 9.8)];

    let chart = Chart::new(PieController::new(parts).spacing(6.0))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("parts_too_small_to_draw");
}

/// Ring spacing wider than the rings: ten nested rings in a small pie, 20 pixels apart, each ring's
/// slot narrower than that. Pins down that rings too thin to keep any thickness after their gaps
/// are left out, without panicking: the rings in between, which give up a whole gap, vanish, while
/// the innermost and the outermost, which give up half a gap each, keep a sliver, the outermost
/// labelled.
#[test]
fn rings_swallowed_by_ring_spacing() {
    let rings: Vec<Vec<(&str, Scalar)>> = (0..10).map(|_| vec![("A", 1.0), ("B", 2.0), ("C", 3.0)]).collect();

    let chart = Chart::new(PieController::new(rings).ring_spacing(20.0))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(400.0, 300.0));

    harness.snapshot("rings_swallowed_by_ring_spacing");
}

/// A pie too small for its long labels. Pins down that the circle shrinks for the label columns
/// only down to half its size without them, and stays there, the labels running out of the chart
/// area to the sides rather than the circle vanishing, as the polar scales' labels do; labels are
/// only left out when a side runs out of room from top to bottom.
#[test]
fn too_small_for_its_labels() {
    let parts = vec![
        ("A very long name for the biggest part", 40.0),
        ("Another long name", 25.0),
        ("Short", 15.0),
        ("A name that goes on and on", 12.0),
        ("Tiny", 8.0),
    ];

    let chart = Chart::new(PieController::new(parts))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(260.0, 160.0));

    harness.snapshot("too_small_for_its_labels");
}

/// Two parts with the same name, and a pie of only negative values, with a legend. Pins down that
/// parts with the same name are two slices in their own colours, the legend listing the name once,
/// in the first's colour; and that a pie of negative values is drawn as the pie of their absolute
/// values.
#[test]
fn repeated_names_and_negative_values() {
    let repeated = vec![("Other", 20.0), ("Food", 50.0), ("Other", 30.0)];

    let chart = Chart::new(PieController::new(repeated))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("repeated_names");

    let losses = vec![("Rent", -45.0), ("Wages", -35.0), ("Other", -20.0)];

    let chart = Chart::new(PieController::new(losses).label_text(|part| format!("{} {}", part.name, part.value)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("negative_values");
}

/// Radii that aren't plain positive sizes: a zero, a negative, and a missing one, beside the one
/// positive size, the largest. Pins down that a part whose radius is zero or less has no slice and
/// no label, its share left empty, and that a part without a radius reaches the rim, as though its
/// size were the largest.
#[test]
fn radii_zero_negative_and_missing() {
    let parts: Vec<(&str, Scalar, Option<Scalar>)> = vec![("Largest", 25.0, Some(2.0)), ("Zero", 25.0, Some(0.0)), ("Negative", 25.0, Some(-3.0)), ("Missing", 25.0, None)];

    let chart = Chart::new(PieController::new(parts.radii()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("radii_zero_negative_and_missing");
}
