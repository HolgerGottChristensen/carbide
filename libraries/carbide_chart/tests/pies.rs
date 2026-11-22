//! Pies, doughnuts and parts of a whole: a `PieController`, each dataset a whole and its points
//! its parts. Around a circle, rings of slices, labelled outside; along the sides, bars of 100%.

use carbide_chart::{Axis, CategoryScale, Chart, DataColor, DatasetExt, LegendPosition, LinearScale, PieController, ScaleId};
use carbide_core::color::GREY;
use carbide_core::draw::{Angle, Dimension};
use carbide_testing::Harness;

mod common;

use common::Framed;

/// A plain pie: one dataset of named parts, with no scales set. Pins down that the chart's default
/// scales become radial because the pie controller comes first, so a pie needs nothing else; that
/// its slices start at the top and run clockwise, each its share of the total, coloured by point;
/// that the scales draw no rings, spokes or labels; that each slice's name is outside the circle,
/// joined to the middle of its edge by a line out and then level to its side's column; and that the
/// legend lists the slices.
#[test]
fn pie() {
    let votes = vec![("Red", 12.0), ("Blue", 19.0), ("Yellow", 3.0), ("Green", 5.0), ("Purple", 2.0), ("Orange", 3.0)];

    let chart = Chart::new(PieController::new(votes))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("pie");
}

/// A doughnut with spacing and labels showing shares: a hole set on the radius scale, a gap
/// between slices, and label text from a closure. Pins down that the inner radius makes a hole
/// while the scale still draws no grid, as its grid lines weren't set; that `spacing` leaves a
/// gap of that many pixels between slices across the middle of the ring; and that `label_text`
/// turns each part, its name, value and share, into its label.
#[test]
fn doughnut_with_spacing_and_shares() {
    let browsers = vec![("Chrome", 64.0), ("Safari", 19.0), ("Edge", 5.0), ("Firefox", 3.0), ("Other", 9.0)];

    let chart = Chart::new(PieController::new(browsers).spacing(4.0).label_text(|part| format!("{} {:.0}%", part.name, part.share)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).inner_radius(0.5))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("doughnut_with_spacing_and_shares");
}

/// Nested rings: two datasets, this year inside last year, with a hole. Pins down that each
/// dataset is a ring in an equal slot out along the radius, the first innermost, with the default
/// ring spacing between them; that each ring's slices are shares of its own total, so both go all
/// the way round; that only the outer ring is labelled outside; and that the legend lists each
/// part once by name.
#[test]
fn nested_rings() {
    let this_year = vec![("Food", 30.0), ("Rent", 45.0), ("Travel", 10.0), ("Other", 15.0)].label("2026");
    let last_year = vec![("Food", 25.0), ("Rent", 40.0), ("Travel", 20.0), ("Other", 15.0)].label("2025");

    let chart = Chart::new(PieController::new((this_year, last_year)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).inner_radius(0.3))
        .legend(LegendPosition::Right)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("nested_rings");
}

/// A half doughnut, a gauge: the angle scale from -90° to 90°, with a hole. Pins down that a
/// partial sweep needs nothing new: the slices share the half circle, which is fitted by its
/// bounding box, and the labels sit beside it.
#[test]
fn half_doughnut() {
    let seats = vec![("Left", 72.0), ("Centre", 48.0), ("Right", 59.0), ("Other", 11.0)];

    let chart = Chart::new(PieController::new(seats))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).inner_radius(0.5))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).start_angle(Angle::Degrees(-90.0)).end_angle(Angle::Degrees(90.0)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("half_doughnut");
}

/// Many slices, most of them small and close together. Pins down that labels on a side are
/// spread apart, their lines bending to follow, and that when a side runs out of room the
/// smallest slices' labels are left out.
#[test]
fn many_small_slices() {
    let mut parts: Vec<(&str, f64)> = vec![("Big", 50.0), ("Large", 25.0)];
    let names = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x"];
    for (index, name) in names.iter().enumerate() {
        parts.push((name, 1.0 + index as f64 * 0.05));
    }

    let chart = Chart::new(PieController::new(parts))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("many_small_slices");
}

/// Values that aren't plain positive parts: a negative value, a missing one, and a dataset with
/// a fixed colour. Pins down that a negative value's slice is the size of its absolute value, a
/// missing value has no slice, the parts unnamed by their x are named by their index, and a dataset
/// coloured with one colour draws every slice in it, beside a pie coloured by point.
#[test]
fn negative_missing_and_fixed_colours() {
    let mixed: Vec<Option<f64>> = vec![Some(30.0), Some(-20.0), None, Some(50.0)];

    let chart = Chart::new(PieController::new((mixed, vec![1.0, 1.0, 1.0].color(DataColor::Color(GREY)))).category_percentage(0.9))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("negative_missing_and_fixed_colours");
}

/// Pies on cartesian scales: the same controller with a category x scale and a linear y scale
/// set, so each dataset is an upright bar of 100%. Pins down that each whole is a bar at its name,
/// its parts stacked from zero to 100 as their shares; that the axes stay along the sides; and that
/// there are no labels outside, as along the sides the legend and axes name the parts.
#[test]
fn bars_of_100_percent_along_the_sides() {
    let this_year = vec![("Food", 30.0), ("Rent", 45.0), ("Travel", 10.0), ("Other", 15.0)].label("2026");
    let last_year = vec![("Food", 25.0), ("Rent", 40.0), ("Travel", 20.0), ("Other", 15.0)].label("2025");

    let chart = Chart::new(PieController::new((this_year, last_year)).category_percentage(0.6))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bars_of_100_percent_along_the_sides");
}

/// A pie whose angle scale draws its grid, as its grid lines were set, with labels off. Pins down
/// that a scale whose grid lines were set keeps its spokes and labels around the rim when the pie
/// asks for its decoration to be hidden, while the radius scale, left as it was, draws none; and
/// that `labels(false)` leaves out the labels outside.
#[test]
fn grid_brought_back() {
    let votes = vec![("Red", 12.0), ("Blue", 19.0), ("Yellow", 3.0), ("Green", 5.0)];

    let chart = Chart::new(PieController::new(votes).labels(false))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).grid_lines(true))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("grid_brought_back");
}

/// Nested rings that touch: `ring_spacing(0.0)`. Pins down that the rings meet without a gap, and
/// without the pixel-wide holes their seam had before rings reached a little into the next.
#[test]
fn touching_rings() {
    let this_year = vec![("Food", 30.0), ("Rent", 45.0), ("Travel", 10.0), ("Other", 15.0)].label("2026");
    let last_year = vec![("Food", 25.0), ("Rent", 40.0), ("Travel", 20.0), ("Other", 15.0)].label("2025");

    let chart = Chart::new(PieController::new((this_year, last_year)).ring_spacing(0.0).labels(false))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).inner_radius(0.3))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("touching_rings");
}

/// A plain pie with its labels turned off and a legend. Pins down that `labels(false)` draws no
/// labels or leader lines outside the circle, and that without them the circle, having no labels to
/// make room for, fills the chart area, its slices named by the legend alone.
#[test]
fn pie_without_labels() {
    let votes = vec![("Red", 12.0), ("Blue", 19.0), ("Yellow", 3.0), ("Green", 5.0), ("Purple", 2.0), ("Orange", 3.0)];

    let chart = Chart::new(PieController::new(votes).labels(false))
        .legend(LegendPosition::Right)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("pie_without_labels");
}
