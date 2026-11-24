//! Tick labels fitted to their axis: turned by a fixed angle or only when they don't fit, aligned
//! automatically or by hand, and thinned out when turning isn't enough.

use carbide_chart::{Axis, AxisPosition, BarController, CategoryScale, Chart, LabelRotation, ScaleId};
use carbide_core::draw::{Alignment, Dimension};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{VStack, WidgetExt};
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::{days, products};

/// Product names turned by 45°, rising to the right, below the chart and above it. Pins down that
/// a fixed rotation turns every label, whether it fits or not, and that the automatic alignment
/// puts the end of each label at its tick below the chart, and the start of each at its tick above
/// it, so the labels point at their ticks; and that the bands are as deep as the turned labels.
#[test]
fn turned_by_a_fixed_angle() {
    let below = Chart::new(BarController::new(products()))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).label_rotation(-45.0))
        .frame(600.0, 250.0);
    let above = Chart::new(BarController::new(products()))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).position(AxisPosition::Top).label_rotation(-45.0))
        .frame(600.0, 250.0);

    let mut harness = Harness::new(VStack::new((below, above)).spacing(0.0).framed(), Dimension::new(600.0, 500.0));

    harness.snapshot("turned_by_a_fixed_angle");
}

/// Product names aligned by hand: flat and hanging right of their ticks (`TopLeading`), flat and
/// hanging left of them (`TopTrailing`), and turned by 60° around their top right corner. Pins
/// down that a set alignment names the point of the label at the end of its tick mark, in the
/// label's own frame, and that a turned label turns around that point.
#[test]
fn aligned_by_hand() {
    let chart = |rotation: f64, alignment: Alignment, height: f64| {
        Chart::new(BarController::new(products()))
            .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).label_rotation(rotation).label_alignment(alignment))
            .frame(600.0, height)
    };

    let charts = VStack::new((
        chart(0.0, Alignment::TopLeading, 200.0),
        chart(0.0, Alignment::TopTrailing, 200.0),
        chart(-60.0, Alignment::TopTrailing, 350.0),
    )).spacing(0.0);

    let mut harness = Harness::new(charts.framed(), Dimension::new(600.0, 750.0));

    harness.snapshot("aligned_by_hand");
}

/// The same product names on a wide chart and a narrow one, with the default rotation. Pins down
/// that labels stay flat while they fit, and turn by 45° on the narrow chart, where they don't,
/// with the band deepened to hold them.
#[test]
fn turned_only_when_they_dont_fit() {
    let wide = Chart::new(BarController::new(products())).frame(900.0, 250.0);
    let narrow = Chart::new(BarController::new(products())).frame(450.0, 250.0);

    let mut harness = Harness::new(VStack::new((wide, narrow)).spacing(0.0).framed(), Dimension::new(900.0, 500.0));

    harness.snapshot("turned_only_when_they_dont_fit");
}

/// Thirty days of bars, too many for every label even turned, with the default rotation, and flat.
/// Pins down that labels too close together are thinned out evenly from the first, turned labels
/// needing less room than flat ones, and that every bar keeps its tick mark and grid line.
#[test]
fn thinned_out_when_turning_isnt_enough() {
    let turned = Chart::new(BarController::new(days())).frame(600.0, 250.0);
    let flat = Chart::new(BarController::new(days()))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).label_rotation(LabelRotation::FLAT))
        .frame(600.0, 250.0);

    let mut harness = Harness::new(VStack::new((turned, flat)).spacing(0.0).framed(), Dimension::new(600.0, 500.0));

    harness.snapshot("thinned_out_when_turning_isnt_enough");
}

/// Thirty days of flat labels with skipping turned off. Pins down that `label_skip(false)` draws
/// every label, overlapping.
#[test]
fn every_label_without_skipping() {
    let chart = Chart::new(BarController::new(days()))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).label_rotation(LabelRotation::FLAT).label_skip(false))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 300.0));

    harness.snapshot("every_label_without_skipping");
}

/// Horizontal bars, their product names turned upright beside the left axis, reading upwards.
/// Pins down that a fixed rotation applies on a vertical axis too, with the label's bottom edge
/// facing the axis, centred on its tick, and the band as wide as the labels are tall. The chart is
/// tall enough for every upright label.
#[test]
fn turned_upright_beside_a_vertical_axis() {
    let chart = Chart::new(BarController::new(products()).horizontal(true))
        .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).label_rotation(-90.0))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 1100.0));

    harness.snapshot("turned_upright_beside_a_vertical_axis");
}

/// A long first category on a narrow chart, its turned label ending at its tick and reaching back
/// past the left axis. Pins down that end labels are left to the chart's padding: the label
/// runs under the left band and into the padding, and the layout doesn't make room for it.
#[test]
fn a_turned_first_label_reaches_past_the_left_axis() {
    let sales = vec![("International shipping", 12.0), ("Pickup", 7.0), ("Courier", 9.0), ("Post", 4.0), ("Locker", 6.0)];

    let chart = Chart::new(BarController::new(sales)).framed();
    let mut harness = Harness::new(chart, Dimension::new(400.0, 400.0));

    harness.snapshot("a_turned_first_label_reaches_past_the_left_axis");
}

/// A rotation and an alignment held in states, changed between renders from flat and automatic
/// to turned by 30° and hanging from their top right corner. Pins down that the label options are
/// states, read on every render.
#[test]
fn rotation_and_alignment_change_between_renders() {
    let mut rotation = LocalState::new(LabelRotation::FLAT);
    let mut alignment = LocalState::new(None);

    let chart = Chart::new(BarController::new(products()))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).label_rotation(rotation.clone()).label_alignment(alignment.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(900.0, 400.0));
    harness.snapshot("rotation_and_alignment_before");

    *rotation.value_mut() = LabelRotation::Fixed(carbide_core::draw::Angle::Degrees(-30.0));
    *alignment.value_mut() = Some(Alignment::TopTrailing);
    harness.snapshot("rotation_and_alignment_after");
}

// ===============================
//  Utility
// ===============================

mod support {
    /// Sales by product, named with names of different lengths, too long to fit flat on a narrow
    /// chart.
    pub fn products() -> Vec<(&'static str, f64)> {
        vec![
            ("Espresso machine", 12.0),
            ("Grinder", 19.0),
            ("Milk frother", 7.0),
            ("Kettle", 15.0),
            ("Pour-over set", 9.0),
            ("Scale", 11.0),
            ("Cold brew maker", 5.0),
        ]
    }

    /// Visitors on each of thirty days, named "Day 1" to "Day 30".
    pub fn days() -> Vec<(String, f64)> {
        (1..=30)
            .map(|day| (format!("Day {day}"), 20.0 + (day as f64 * 1.7).sin() * 8.0 + day as f64 * 0.5))
            .collect()
    }
}
