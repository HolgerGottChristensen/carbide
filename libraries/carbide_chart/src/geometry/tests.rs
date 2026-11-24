//! Picking on whole charts: each test renders a chart, picks at positions found from its shapes,
//! and snapshots the chart with what each pick found outlined in white and the position marked
//! with a red cross (see `support::Probe`).

use crate::{Axis, BarController, CategoryScale, Chart, DatasetExt, Fill, LinearScale, LineController, PieController, ScaleId, ScatterController, SizedDataSetExt, StackId, ZoomWindow};
use super::{Measure, PickMode};
use support::{key, probe, At, Drawn, Kind};
use crate::Orientation;

const QUARTERS: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];

/// Two datasets of bars side by side. Pins down that a point inside a bar finds that bar alone,
/// and one in the empty space above the bars finds nothing; and that nothing at all is found
/// from outside the chart area, in any mode.
#[test]
fn point_finds_the_bar_under_it() {
    let chart = Chart::new(BarController::new((vec![10.0, 20.0, 30.0, 25.0], vec![15.0, 25.0, 5.0, 20.0])))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(QUARTERS));

    let found = probe(chart, "point_finds_the_bar_under_it", vec![
        (At::Datum(key(0, 1, 1)), PickMode::Point),
        (At::Area(0.05, 0.05), PickMode::Point),
        (At::Area(-0.05, 0.5), PickMode::Nearest(Measure::Both)),
        (At::Area(0.5, 1.05), PickMode::Index),
    ]);

    assert_eq!(found[0], vec![key(0, 1, 1)]);
    assert_eq!(found[1], vec![], "above the bars, nothing is under the position");
    assert_eq!(found[2], vec![], "left of the chart area");
    assert_eq!(found[3], vec![], "below the chart area");
}

/// Vertical grouped bars, and horizontal stacked bars. Pins down that `Index` finds every dataset's
/// bar at the category nearest the position along the index axis, from above the bars along x,
/// and from beside them along y, where the stack's parts sit at the same category.
#[test]
fn index_finds_every_dataset_at_the_nearest_category() {
    let chart = Chart::new(BarController::new((vec![10.0, 20.0, 30.0, 25.0], vec![15.0, 25.0, 5.0, 20.0])))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(QUARTERS));

    let found = probe(chart, "grouped", vec![(At::Offset(Box::new(At::Datum(key(0, 1, 2))), 0.0, -120.0), PickMode::Index)]);
    assert_eq!(found[0], vec![key(0, 0, 2), key(0, 1, 2)]);

    const COSTS: StackId = StackId::new("costs");
    let chart = Chart::new(BarController::new((vec![10.0, 20.0, 30.0, 25.0].stack(COSTS), vec![15.0, 25.0, 5.0, 20.0].stack(COSTS))).horizontal(true))
        .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).labels(QUARTERS));

    // In the empty space right of Q3's stack, a little below its middle.
    let found = probe(chart, "horizontal_stacked", vec![(At::Offset(Box::new(At::Datum(key(0, 1, 2))), 110.0, 15.0), PickMode::Index)]);
    assert_eq!(found[0], vec![key(0, 0, 2), key(0, 1, 2)]);
}

/// A filled line. Pins down that a position on the line between two points finds the nearer of
/// them, either side of the middle, and one inside the area under the line finds the point above
/// it.
#[test]
fn lines_pick_their_nearer_point_and_areas_the_point_above() {
    let chart = Chart::new(LineController::new(vec![5.0, 9.0, 4.0, 7.0, 6.0].fill(Fill::Origin)));

    let found = probe(chart, "lines_pick_their_nearer_point_and_areas_the_point_above", vec![
        (At::Between(key(0, 0, 0), key(0, 0, 1), 0.3), PickMode::Point),
        (At::Between(key(0, 0, 0), key(0, 0, 1), 0.8), PickMode::Point),
        (At::Offset(Box::new(At::Datum(key(0, 0, 2))), 6.0, 60.0), PickMode::Point),
    ]);

    assert_eq!(found[0], vec![key(0, 0, 0)]);
    assert_eq!(found[1], vec![key(0, 0, 1)]);
    assert_eq!(found[2], vec![key(0, 0, 2)], "inside the area, under the third point");
}

/// Scattered points in two datasets. Pins down that `Nearest` measured straight across finds the
/// point nearest the position, while measured along x it finds the one nearest in x, however far
/// away in y; and that `Dataset` finds every point of the nearest point's dataset.
#[test]
fn nearest_measures_straight_or_along_x() {
    let near = vec![(1.0, 2.0), (3.0, 8.0), (5.0, 3.0)];
    let far = vec![(2.0, 9.0), (4.0, 1.0)];
    let chart = Chart::new(ScatterController::new((near, far)));

    // Just above and right of (3, 8), on the way to (4, 1) in x.
    let at = At::Between(key(0, 0, 1), key(0, 1, 1), 0.3);
    let found = probe(chart, "nearest_measures_straight_or_along_x", vec![
        (at.clone(), PickMode::Nearest(Measure::Both)),
        (At::Offset(Box::new(At::Datum(key(0, 1, 1))), -10.0, -150.0), PickMode::Nearest(Measure::X)),
        (at, PickMode::Dataset),
    ]);

    assert_eq!(found[0], vec![key(0, 0, 1)]);
    assert_eq!(found[1], vec![key(0, 1, 1)], "along x, (4, 1) is nearest");
    assert_eq!(found[2], vec![key(0, 0, 0), key(0, 0, 1), key(0, 0, 2)]);
}

/// Two radars, picked 105° round from the top, nearer the Strength spoke at 72° than the Stamina
/// spoke at 144°, but nearest, straight across, to a point on the Stamina spoke. Pins down that
/// `Index` finds both datasets' points on the Strength spoke, and so does `Nearest` around the
/// circle, whatever their distance from the centre, while `Nearest` straight across finds the
/// point on the Stamina spoke.
#[test]
fn radars_pick_by_spoke() {
    let skills = ["Speed", "Strength", "Stamina", "Passing", "Shooting"];
    let chart = Chart::new(LineController::new((vec![8.0, 3.0, 6.0, 9.0, 4.0], vec![2.0, 7.0, 5.0, 3.0, 8.0])))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(skills));

    let at = At::Around(105.0, 75.0);
    let found = probe(chart, "radars_pick_by_spoke", vec![
        (at.clone(), PickMode::Index),
        (at.clone(), PickMode::Nearest(Measure::Angle)),
        (at, PickMode::Nearest(Measure::Both)),
    ]);

    assert_eq!(found[0], vec![key(0, 0, 1), key(0, 1, 1)]);
    assert_eq!(found[1], vec![key(0, 0, 1), key(0, 1, 1)]);
    assert_eq!(found[2], vec![key(0, 1, 2)], "straight across, the second radar's Stamina point is nearest");
}

/// A doughnut of two rings. Pins down that a position in a slice finds that slice, one in the
/// hole finds nothing, and `Dataset` on the inner ring finds every part of the inner ring.
#[test]
fn pies_pick_slices_and_rings() {
    let rings = vec![vec![("A", 1.0), ("B", 2.0), ("C", 3.0)], vec![("A", 3.0), ("B", 1.0), ("C", 1.0)]];
    let chart = Chart::new(PieController::new(rings).labels(false))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).inner_radius(0.4));

    let found = probe(chart, "pies_pick_slices_and_rings", vec![
        (At::Datum(key(0, 1, 2)), PickMode::Point),
        (At::Area(0.5, 0.5), PickMode::Point),
        (At::Datum(key(0, 0, 1)), PickMode::Dataset),
    ]);

    assert_eq!(found[0], vec![key(0, 1, 2)]);
    assert_eq!(found[1], vec![], "in the hole");
    assert_eq!(found[2], vec![key(0, 0, 0), key(0, 0, 1), key(0, 0, 2)]);
}

/// Two overlapping bubbles in different datasets. Pins down that `Point` finds every bubble under
/// the position, the one drawn on top first.
#[test]
fn bubbles_under_each_other_are_found_top_first() {
    let lower = vec![(3.0, 3.0, 40.0), (6.0, 6.0, 10.0)];
    let upper = vec![(3.3, 3.3, 30.0), (8.0, 2.0, 5.0)];
    let chart = Chart::new(ScatterController::new((lower.bubbles(), upper.bubbles())));

    let found = probe(chart, "bubbles_under_each_other_are_found_top_first", vec![
        (At::Between(key(0, 0, 0), key(0, 1, 0), 0.5), PickMode::Point),
    ]);

    assert_eq!(found[0], vec![key(0, 1, 0), key(0, 0, 0)]);
}

/// Bubbles zoomed along x so the largest one's centre is just past the chart area's right edge,
/// part of it still showing, the pointer on that part. Pins down that a datum shown only partly
/// counts for `Nearest`, and that a bubble is measured from its edge, so the pointer on it finds it
/// rather than a bubble whose centre is inside the area but further away.
#[test]
fn nearest_finds_a_bubble_partly_past_the_edge() {
    let bubbles = vec![(2.0, 3.0, 10.0), (4.0, 5.0, 10.0), (5.0, 5.0, 40.0), (8.0, 7.0, 20.0)];
    let chart = Chart::new(ScatterController::new(bubbles.bubbles()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(ZoomWindow::new(0.0, 0.48)));

    // 25 pixels left of the big bubble's centre, which lies about 20 pixels past the edge.
    let found = probe(chart, "nearest_finds_a_bubble_partly_past_the_edge", vec![
        (At::Offset(Box::new(At::Datum(key(0, 0, 2))), -25.0, 0.0), PickMode::Nearest(Measure::Both)),
    ]);

    assert_eq!(found[0], vec![key(0, 0, 2)]);
}

/// A line of 400 points, about a pixel and a third apart, picked by the nearest along x. Pins down
/// that points close together don't tie because the pointer is within their radius of several of
/// them: among those, the one whose centre is nearest is found, alone.
#[test]
fn nearest_finds_one_of_many_points_close_together() {
    let points: Vec<(f64, f64)> = (0..400).map(|i| (i as f64, 50.0 + 30.0 * (i as f64 / 9.0).sin())).collect();
    let chart = Chart::new(LineController::new(points)).scale(ScaleId::X, LinearScale::new(Axis::Horizontal));

    let found = probe(chart, "nearest_finds_one_of_many_points_close_together", vec![
        (At::Offset(Box::new(At::Datum(key(0, 0, 200))), 0.3, -40.0), PickMode::Nearest(Measure::X)),
    ]);

    assert_eq!(found[0], vec![key(0, 0, 200)]);
}

/// Radial bars: wedges around a circle. Pins down that a position in a wedge finds it, and that
/// `Index` finds both datasets' wedges at the category nearest around the circle.
#[test]
fn wedges_are_picked_around_the_circle() {
    let chart = Chart::new(BarController::new((vec![10.0, 20.0, 30.0, 25.0], vec![15.0, 25.0, 5.0, 20.0])))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(QUARTERS));

    let found = probe(chart, "wedges_are_picked_around_the_circle", vec![
        (At::Datum(key(0, 0, 2)), PickMode::Point),
        (At::Offset(Box::new(At::Datum(key(0, 1, 3))), 0.0, -30.0), PickMode::Index),
    ]);

    assert_eq!(found[0], vec![key(0, 0, 2)]);
    assert_eq!(found[1], vec![key(0, 0, 3), key(0, 1, 3)]);
}

/// Lollipops drawn by a controller of the test's own, each a stem and a dot added through the
/// public `Shapes::add`, two datasets side by side. Pins down that a datum of two shapes is found
/// once, by its dot or its stem, and that `on_index` places it, so `Index` finds both datasets'
/// lollipops at the category nearest the position.
#[test]
fn shapes_added_by_a_controller_of_its_own_are_found() {
    let chart = Chart::new(Drawn::new(Kind::Lollipops(Orientation::Vertical), (vec![10.0, 20.0, 30.0, 25.0], vec![15.0, 25.0, 5.0, 20.0])))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(QUARTERS));

    let found = probe(chart, "shapes_added_by_a_controller_of_its_own_are_found", vec![
        (At::Dot(key(0, 0, 1)), PickMode::Point),
        (At::Datum(key(0, 1, 2)), PickMode::Point),
        (At::Offset(Box::new(At::Datum(key(0, 1, 3))), 0.0, -150.0), PickMode::Index),
        (At::Offset(Box::new(At::Dot(key(0, 0, 0))), 3.0, -14.0), PickMode::Nearest(Measure::Both)),
    ]);

    assert_eq!(found[0], vec![key(0, 0, 1)], "the dot");
    assert_eq!(found[1], vec![key(0, 1, 2)], "the stem, found once");
    assert_eq!(found[2], vec![key(0, 0, 3), key(0, 1, 3)]);
    assert_eq!(found[3], vec![key(0, 0, 0)]);
}

/// The lollipops of `shapes_added_by_a_controller_of_its_own_are_found` laid along x, and
/// wedges around a circle, both from the controller of the test's own. Pins down that `on_index`
/// takes the datum's place from the way its index scale runs: along y for horizontal lollipops,
/// and around the circle for wedges on a radial category scale.
#[test]
fn shapes_are_placed_on_the_way_their_index_scale_runs() {
    let chart = Chart::new(Drawn::new(Kind::Lollipops(Orientation::Horizontal), (vec![10.0, 20.0, 30.0, 25.0], vec![15.0, 25.0, 5.0, 20.0])))
        .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).labels(QUARTERS));

    let found = probe(chart, "horizontal_shapes_of_a_controller_of_its_own", vec![
        (At::Offset(Box::new(At::Datum(key(0, 1, 2))), 150.0, 4.0), PickMode::Index),
    ]);
    assert_eq!(found[0], vec![key(0, 0, 2), key(0, 1, 2)]);

    let chart = Chart::new(Drawn::new(Kind::Wedges, vec![10.0, 20.0, 30.0, 25.0]))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(QUARTERS));

    let found = probe(chart, "wedges_of_a_controller_of_its_own", vec![
        (At::Datum(key(0, 0, 2)), PickMode::Point),
        (At::Around(100.0, 20.0), PickMode::Index),
    ]);
    assert_eq!(found[0], vec![key(0, 0, 2)]);
    assert_eq!(found[1], vec![key(0, 0, 1)], "Q2 runs from 90° to 180°");
}

/// A line drawn by the controller of the test's own through `Shapes::add_line`, with a gap at a
/// missing point and its area down to zero. Pins down that a position on the line finds the
/// nearer vertex, one across the gap finds nothing, and one in the area the vertex above it.
#[test]
fn lines_added_by_a_controller_of_its_own_are_found() {
    let chart = Chart::new(Drawn::new(Kind::Line, vec![Some(5.0), Some(9.0), None, Some(4.0), Some(7.0)]));

    let found = probe(chart, "lines_added_by_a_controller_of_its_own_are_found", vec![
        (At::Between(key(0, 0, 0), key(0, 0, 1), 0.3), PickMode::Point),
        (At::Between(key(0, 0, 1), key(0, 0, 3), 0.5), PickMode::Point),
        (At::Offset(Box::new(At::Datum(key(0, 0, 4))), -6.0, 60.0), PickMode::Point),
        (At::Offset(Box::new(At::Datum(key(0, 0, 3))), 4.0, -100.0), PickMode::Index),
    ]);

    assert_eq!(found[0], vec![key(0, 0, 0)]);
    assert_eq!(found[1], vec![], "across the gap");
    assert_eq!(found[2], vec![key(0, 0, 4)], "in the area, under the last point");
    assert_eq!(found[3], vec![key(0, 0, 3)]);
}

// ===============================
//  Utility
// ===============================

mod support {
    use std::cell::RefCell;
    use std::rc::Rc;
    use carbide::color::{BLUE, RED, WHITE};
    use carbide::draw::{Dimension, Position, Scalar};
    use carbide::render::{Render, RenderContext};
    use carbide::state::ValueState;
    use carbide::widget::{CommonWidget, Widget, WidgetExt, WidgetId, WidgetSync};
    use carbide_core::CommonWidgetImpl;
    use carbide_core::widget::canvas::CanvasContext;
    use carbide_fluent::{locale, LocaleExt};
    use carbide_testing::Harness;
    use crate::Chart;
    use crate::ControllerSequence;
    use carbide::draw::Rect;
    use carbide::environment::Environment;
    use crate::{DataSet, DatasetController, IntoSequence, Orientation, PolarRole, ScaleBounds, Scales, Sequence, Shape};
    use crate::geometry::{DatumKey, PickMode, Shapes};

    pub(super) fn key(controller: usize, dataset: usize, index: usize) -> DatumKey {
        DatumKey { controller, dataset, index }
    }

    /// Where to pick, from the chart's shapes on the render, so tests need not know the layout.
    #[derive(Clone, Debug)]
    pub(super) enum At {
        /// Where the datum is drawn (see `Shapes::anchor`).
        Datum(DatumKey),
        /// The way from where one datum is drawn to where another is, from 0 to 1.
        Between(DatumKey, DatumKey, Scalar),
        /// A share of the way across and down the chart area, outside it below 0 or above 1.
        Area(Scalar, Scalar),
        /// Another position, moved by pixels.
        Offset(Box<At>, Scalar, Scalar),
        /// Around the chart's circle: degrees clockwise from the top, and pixels from the centre.
        Around(Scalar, Scalar),
        /// The dot of a lollipop drawn by `Drawn`: the middle of its datum's second shape.
        Dot(DatumKey),
    }

    impl At {
        fn resolve(&self, shapes: &Shapes) -> Position {
            match self {
                At::Datum(key) => shapes.anchor(*key).unwrap_or_else(|| panic!("{key:?} wasn't drawn")),
                At::Between(from, to, share) => {
                    let (from, to) = (At::Datum(*from).resolve(shapes), At::Datum(*to).resolve(shapes));
                    Position::new(from.x + (to.x - from.x) * share, from.y + (to.y - from.y) * share)
                }
                At::Area(across, down) => {
                    let area = shapes.area();
                    Position::new(area.left() + across * area.width(), area.bottom() + down * area.height())
                }
                At::Offset(at, dx, dy) => {
                    let at = at.resolve(shapes);
                    Position::new(at.x + dx, at.y + dy)
                }
                At::Dot(key) => shapes.anchors.iter()
                    .filter(|anchor| anchor.key == *key)
                    .nth(1)
                    .map(|anchor| anchor.center)
                    .unwrap_or_else(|| panic!("{key:?} has no dot")),
                At::Around(degrees, distance) => {
                    let center = shapes.circle.expect("a polar chart").center;
                    let (sin, cos) = degrees.to_radians().sin_cos();
                    Position::new(center.x + distance * sin, center.y - distance * cos)
                }
            }
        }
    }

    /// Renders `chart` and picks at each position as its mode says, then snapshots it as `name`,
    /// with what each pick found outlined, and returns what each found.
    pub(super) fn probe<C: ControllerSequence>(chart: Chart<C>, name: &str, picks: Vec<(At, PickMode)>) -> Vec<Vec<DatumKey>> {
        let found = Rc::new(RefCell::new(vec![]));
        let probe = Probe { id: WidgetId::new(), position: Position::new(0.0, 0.0), dimension: Dimension::new(100.0, 100.0), child: chart, picks, found: found.clone() };

        // Framed as the snapshots in `tests/` are, with Danish tick labels.
        let framed = probe.border().color(BLUE).locale(ValueState::new(locale!("da")));
        let mut harness = Harness::new(framed, Dimension::new(600.0, 500.0));
        harness.snapshot(name);

        found.borrow().clone()
    }

    /// A chart that picks after it renders, and draws what it found on top of itself.
    #[derive(Debug, Clone, Widget)]
    #[carbide_exclude(Render)]
    struct Probe<C> where C: ControllerSequence {
        #[id] id: WidgetId,
        position: Position,
        dimension: Dimension,
        child: Chart<C>,
        picks: Vec<(At, PickMode)>,
        found: Rc<RefCell<Vec<Vec<DatumKey>>>>,
    }

    impl<C: ControllerSequence> CommonWidget for Probe<C> {
        CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
    }

    impl<C: ControllerSequence> Render for Probe<C> {
        fn render(&mut self, ctx: &mut RenderContext) {
            self.sync(ctx.env);
            self.child.render(ctx);

            let shapes = self.child.shapes();
            let positions: Vec<Position> = self.picks.iter().map(|(at, _)| at.resolve(shapes)).collect();
            let found: Vec<Vec<DatumKey>> = positions.iter().zip(&self.picks).map(|(at, (_, mode))| self.child.pick(*at, *mode)).collect();

            let mut canvas = CanvasContext::new(self.child.position(), self.child.dimension(), ctx);
            canvas.set_line_width(2.0);
            canvas.set_stroke_style(WHITE);
            for key in found.iter().flatten() {
                shapes.outline(&mut canvas, *key);
            }

            canvas.set_stroke_style(RED);
            canvas.begin_path();
            for at in &positions {
                canvas.move_to(at.x - 6.0, at.y);
                canvas.line_to(at.x + 6.0, at.y);
                canvas.move_to(at.x, at.y - 6.0);
                canvas.line_to(at.x, at.y + 6.0);
            }
            canvas.stroke();

            *self.found.borrow_mut() = found;
        }
    }

    /// What `Drawn` draws each dataset as.
    #[derive(Copy, Clone, Debug)]
    pub(super) enum Kind {
        /// A lollipop for each point: a stem from zero and a dot, two shapes, upright or along x,
        /// each dataset's a little further along the index axis than the one before.
        Lollipops(Orientation),
        /// A wedge for each point around the circle, on a radial x scale, added but not drawn.
        Wedges,
        /// A line through the points, with a gap at each missing one, filled down to zero.
        Line,
    }

    /// A controller of the test's own, outside the built-in ones, that draws and adds its shapes
    /// through the public API only, as a controller in another crate would.
    #[derive(Clone, Debug)]
    pub(super) struct Drawn<D> {
        kind: Kind,
        datasets: D,
    }

    impl<D: Sequence<dyn DataSet>> Drawn<D> {
        pub(super) fn new<I: IntoSequence<dyn DataSet, M, Sequence=D>, M>(kind: Kind, datasets: I) -> Drawn<D> {
            Drawn { kind, datasets: datasets.into_sequence() }
        }

        fn orientation(&self) -> Orientation {
            match self.kind {
                Kind::Lollipops(orientation) => orientation,
                _ => Orientation::Vertical,
            }
        }
    }

    impl<D: Sequence<dyn DataSet>> DatasetController for Drawn<D> {
        fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
            self.datasets.foreach(&mut |_, dataset| {
                let options = dataset.options();
                let (index_id, value_id) = self.orientation().index_and_value_scales(&options);
                bounds.register_values(dataset, index_id, value_id);
                let (placement, requests) = bounds.split();

                if placement.is_radial(index_id) {
                    requests.request_polar_role(index_id, PolarRole::Angle);
                    requests.request_polar_role(value_id, PolarRole::Radius);
                }
                if !placement.categories(index_id).is_empty() {
                    requests.request_offset(index_id);
                }
                requests.include(value_id, 0.0, 0.0);

                let coordinates = placement.coordinates(index_id, value_id);
                dataset.points(&mut |_, point| {
                    if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
                        requests.include_point(index_id, value_id, x, y);
                    }
                });
            });
        }

        fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect, shapes: &mut Shapes) {
            let orientation = self.orientation();
            self.datasets.foreach(&mut |dataset_index, dataset| {
                let options = dataset.options();
                let (index_id, value_id) = orientation.index_and_value_scales(&options);
                let coordinates = scales.coordinates(index_id, value_id);
                let color = if dataset_index == 0 { BLUE } else { RED };

                match self.kind {
                    Kind::Lollipops(_) => dataset.points(&mut |point_index, point| {
                        let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) else {
                            return;
                        };
                        let index = scales.pixel(index_id, x, area).unwrap() + 12.0 * dataset_index as Scalar;
                        let (base, value) = (scales.pixel(value_id, 0.0, area).unwrap(), scales.pixel(value_id, y, area).unwrap());
                        let stem = Shape::rect(orientation, index - 1.0, 2.0, base, value);
                        let dot = orientation.swap(Position::new(index, value));

                        ctx.begin_path();
                        ctx.circle(dot.x, dot.y, 10.0);
                        ctx.set_fill_style(color);
                        ctx.fill();
                        shapes.add(dataset_index, point_index, color, stem).on_index(index_id, x);
                        shapes.add(dataset_index, point_index, color, Shape::marker(dot, 5.0)).on_index(index_id, x);
                    }),
                    Kind::Wedges => {
                        let polar = scales.polar(index_id, value_id).unwrap();
                        let (index_scale, value_scale) = (scales.get(index_id).unwrap(), scales.get(value_id).unwrap());
                        dataset.points(&mut |point_index, point| {
                            let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) else {
                                return;
                            };
                            let angles = (polar.angle(index_scale.normalize(x - 0.5)), polar.angle(index_scale.normalize(x + 0.5)));
                            let distances = (polar.distance(0.0), polar.distance(value_scale.normalize(y)));

                            // Not drawn: only the outlines of the wedges found show.
                            shapes.add(dataset_index, point_index, color, Shape::wedge(polar, angles, distances)).on_index(index_id, x);
                        });
                    }
                    Kind::Line => {
                        let base = scales.pixel(value_id, 0.0, area).unwrap();
                        let mut points = vec![];
                        dataset.points(&mut |point_index, point| {
                            let at = coordinates.x(point).zip(coordinates.y(point))
                                .map(|(x, y)| (x, Position::new(scales.pixel(index_id, x, area).unwrap(), scales.pixel(value_id, y, area).unwrap())));
                            points.push((point_index, at));
                        });
                        let runs: Vec<Vec<Position>> = points.split(|(_, at)| at.is_none())
                            .filter(|run| !run.is_empty())
                            .map(|run| run.iter().filter_map(|(_, at)| at.map(|(_, at)| at)).collect())
                            .collect();

                        ctx.begin_path();
                        for run in &runs {
                            ctx.move_to(run[0].x, run[0].y);
                            for at in &run[1..] {
                                ctx.line_to(at.x, at.y);
                            }
                        }
                        ctx.set_stroke_style(color);
                        ctx.stroke();

                        let mut line = shapes.add_line(dataset_index, color);
                        for (point_index, at) in points {
                            line = match at {
                                Some((x, at)) => line.vertex(point_index, at).on_index(index_id, x),
                                None => line.gap(),
                            };
                        }
                        for run in runs {
                            let (first, last) = (run[0], run[run.len() - 1]);
                            line = line.area(run.into_iter().chain([Position::new(last.x, base), Position::new(first.x, base)]));
                        }
                    }
                }
            });
        }
    }
}
