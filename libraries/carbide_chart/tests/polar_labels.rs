//! Labels around a polar chart: the rim labels turned flat, along the circle or out from its
//! centre, and the rim and radius labels thinned out when they are too close together.

use carbide_chart::{Axis, BarController, CategoryScale, Chart, LineController, LinearScale, RimOrientation, ScaleId};
use carbide_core::draw::Dimension;
use carbide_core::widget::{HStack, WidgetExt};
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::{months, SKILLS};

/// A radar with long skill names, its rim labels flat, along the circle and out from its centre.
/// Pins down that each orientation turns the labels as it says without any reading upside down:
/// along the circle the lower labels are turned half a turn, out from the centre the left ones
/// end at the rim; and that the circle shrinks to fit the labels as turned.
#[test]
fn turned_three_ways() {
    let radar = |orientation: RimOrientation| {
        Chart::new(LineController::new(vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0]))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS).rim_orientation(orientation))
            .frame(500.0, 500.0)
    };

    let charts = HStack::new((radar(RimOrientation::Flat), radar(RimOrientation::Along), radar(RimOrientation::Outward))).spacing(0.0);
    let mut harness = Harness::new(charts.framed(), Dimension::new(1500.0, 500.0));

    harness.snapshot("turned_three_ways");
}

/// Radial bars for thirty-six months on a small circle, their labels flat and along the circle.
/// Pins down that rim labels too close together are thinned out evenly, counted both ways from
/// the first around the full circle, so the first is always shown and the two ends meet evenly;
/// and that labels along the circle, which need less room, are thinned out less.
#[test]
fn thinned_out_around_the_circle() {
    let bars = |orientation: RimOrientation| {
        Chart::new(BarController::new((0..36).map(|month| 20.0 + (month as f64 * 0.7).sin() * 10.0).collect::<Vec<_>>()))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(months()).rim_orientation(orientation))
            .frame(400.0, 400.0)
    };

    let charts = HStack::new((bars(RimOrientation::Flat), bars(RimOrientation::Along))).spacing(0.0);
    let mut harness = Harness::new(charts.framed(), Dimension::new(800.0, 400.0));

    harness.snapshot("thinned_out_around_the_circle");
}

/// A small radar whose radius scale has a tick every 5 up to 100, with its labels thinned out and
/// with skipping turned off. Pins down that the radius labels are thinned out along their spoke
/// from the first, and that `label_skip(false)` keeps every one, overlapping.
#[test]
fn radius_labels_thinned_out_along_the_spoke() {
    let radar = |skip: bool| {
        Chart::new(LineController::new(vec![80.0, 60.0, 70.0, 90.0, 50.0, 40.0]))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["Speed", "Strength", "Stamina", "Passing", "Shooting", "Defence"]))
            .scale(ScaleId::Y, LinearScale::new(Axis::Radial).min(0.0).max(100.0).step(5.0).label_skip(skip))
            .frame(450.0, 350.0)
    };

    let charts = HStack::new((radar(true), radar(false))).spacing(0.0);
    let mut harness = Harness::new(charts.framed(), Dimension::new(900.0, 350.0));

    harness.snapshot("radius_labels_thinned_out_along_the_spoke");
}

/// The same rim labels without skipping, flat and along the circle. Pins down that
/// `label_skip(false)` on the angle scale keeps every rim label, overlapping.
#[test]
fn every_rim_label_without_skipping() {
    let bars = |orientation: RimOrientation| {
        Chart::new(BarController::new((0..36).map(|month| 20.0 + (month as f64 * 0.7).sin() * 10.0).collect::<Vec<_>>()))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(months()).rim_orientation(orientation).label_skip(false))
            .frame(400.0, 400.0)
    };

    let charts = HStack::new((bars(RimOrientation::Flat), bars(RimOrientation::Along))).spacing(0.0);
    let mut harness = Harness::new(charts.framed(), Dimension::new(800.0, 400.0));

    harness.snapshot("every_rim_label_without_skipping");
}

// ===============================
//  Utility
// ===============================

mod support {
    /// The skills a radar rates, some of them long.
    pub const SKILLS: [&str; 6] = ["Speed and agility", "Strength", "Stamina over ninety minutes", "Passing", "Shooting from range", "Defence"];

    /// Thirty-six months, from January 2024, as "Jan 24".
    pub fn months() -> Vec<String> {
        const NAMES: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        (0..36).map(|month| format!("{} {}", NAMES[month % 12], 24 + month / 12)).collect()
    }
}
