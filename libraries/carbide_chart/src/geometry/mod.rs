//! The chart's laid-out geometry: the shape every datum is drawn as, laid out by the controllers
//! on every render and kept on the chart until the next. Drawing reads the shapes, picking finds
//! data by them, and, later, animation will move them from one update to the next.
//!
//! A datum is known by its controller, its dataset and its index in the dataset (see `DatumKey`).

mod shape;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use carbide::color::Color;
use carbide::draw::{Dimension, Position, Rect, Scalar};
use indexmap::IndexSet;
pub(crate) use shape::{Along, Hit, Polyline, Shape};
use shape::distance;
use crate::scale::{Circle, ScaleId};

/// Which datum something is: its controller's index in the chart, its dataset's index in the
/// controller, and its index in the dataset, as the controllers count them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct DatumKey {
    pub(crate) controller: usize,
    pub(crate) dataset: usize,
    pub(crate) index: usize,
}

/// Which dataset something belongs to: its controller's index in the chart and its index in the
/// controller.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct DatasetKey {
    pub(crate) controller: usize,
    pub(crate) dataset: usize,
}

impl DatasetKey {
    fn datum(self, index: usize) -> DatumKey {
        DatumKey { controller: self.controller, dataset: self.dataset, index }
    }
}

/// The datum a shape, or a vertex of a polyline, shows: its index in its dataset, and where it
/// sits along its index scale, when it has one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Datum {
    pub(crate) index: usize,
    pub(crate) place: Option<IndexPlace>,
}

/// Where a datum sits along its index scale: the scale, by its slot in `Shapes` (see
/// `Shapes::place`), its value there, and where that is on the screen.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IndexPlace {
    pub(crate) scale: usize,
    pub(crate) value: Scalar,
    pub(crate) along: Along,
}

/// One shape a controller drew, in the colour it filled it with, for one of its datasets.
#[derive(Clone, Debug)]
pub(crate) struct Item {
    pub(crate) dataset: DatasetKey,
    pub(crate) color: Color,
    pub(crate) shape: Shape,
    /// The datum the shape shows, or `None` for a polyline, whose vertices each show their own.
    pub(crate) datum: Option<Datum>,
}

impl Item {
    /// The datum of the shape, or of the polyline's vertex `vertex`.
    fn datum(&self, vertex: usize) -> Option<&Datum> {
        match (&self.shape, &self.datum) {
            (Shape::Polyline(line), _) => line.vertices.get(vertex).map(|(_, datum)| datum),
            (_, datum) => datum.as_ref(),
        }
    }
}

/// A point a datum is measured from when looking for the nearest one: a shape's centre, or a
/// polyline's vertex.
#[derive(Clone, Debug)]
struct Anchor {
    key: DatumKey,
    center: Position,
    place: Option<IndexPlace>,
}

/// One dataset's anchors, a range of `Shapes::anchors`, and whether they lie in order along a side
/// of the chart area, so the ones at a place along it can be found with a binary search.
#[derive(Clone, Debug)]
struct Group {
    anchors: std::ops::Range<usize>,
    /// Along x (`true`) or y, and whether the coordinate grows (`1.0`) or shrinks (`-1.0`) from
    /// one anchor to the next; `None` when they aren't in order along a side.
    sorted: Option<(bool, Scalar)>,
}

/// How a pick finds data at a position (see `Shapes::pick`).
#[cfg_attr(not(test), allow(dead_code, reason = "constructed by pointer input, Interaction 3 in docs/multiple-controllers-and-scales.md, which isn't built yet"))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PickMode {
    /// The data whose shapes are under the position, the one drawn on top first: inside a bar, a
    /// wedge or a filled area, or within `LINE_TOLERANCE` of a marker or a line.
    Point,
    /// The datum nearest the position, measured as `Measure` says, from the centres of shapes and
    /// the points of lines; all of them when several are equally near.
    Nearest(Measure),
    /// The data at the index (the category, along the index axis) nearest the position, in every
    /// dataset that has one there, as for a shared tooltip.
    Index,
    /// Every datum of the dataset under the position, or else of the nearest datum's dataset.
    Dataset,
}

/// What `PickMode::Nearest` measures the distance along.
#[allow(dead_code, reason = "constructed by pointer input, Interaction 3 in docs/multiple-controllers-and-scales.md, which isn't built yet")]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Measure {
    /// Straight across the screen.
    Both,
    /// Along x only.
    X,
    /// Along y only.
    Y,
    /// Around the chart's circle: the angle between them, at the position's distance from the
    /// centre. Without a circle, straight across the screen.
    Angle,
    /// Out from the chart's circle's centre: the difference in distance. Without a circle,
    /// straight across the screen.
    Distance,
}

/// The shapes a chart's controllers drew on its last render, with the chart area and the circle
/// they were drawn in. Controllers add a shape for each datum as they draw it (see
/// `DatasetController::draw`); the chart finds the data under or near a position through them.
#[derive(Clone, Debug)]
pub struct Shapes {
    area: Rect,
    circle: Option<Circle>,
    /// The controller whose shapes are being added.
    controller: usize,
    /// The index scales the data's places are on (see `IndexPlace`), so each place holds a slot
    /// instead of an id.
    scales: IndexSet<ScaleId>,
    items: Vec<Item>,
    /// Every datum's anchors, by dataset, built by `finish`.
    anchors: Vec<Anchor>,
    groups: Vec<Group>,
}

impl Default for Shapes {
    fn default() -> Shapes {
        Shapes::new(Rect::new(Position::new(0.0, 0.0), Dimension::new(0.0, 0.0)), None)
    }
}

impl Shapes {
    /// No shapes yet, for a render into `area`, around `circle` when the chart has polar scales.
    pub(crate) fn new(area: Rect, circle: Option<Circle>) -> Shapes {
        Shapes { area, circle, controller: 0, scales: IndexSet::new(), items: vec![], anchors: vec![], groups: vec![] }
    }

    /// The chart area the shapes were drawn in.
    pub(crate) fn area(&self) -> Rect {
        self.area
    }

    /// Makes the shapes added from now on the controller at `controller`'s.
    pub(crate) fn set_controller(&mut self, controller: usize) {
        self.controller = controller;
    }

    /// Where a datum at `value` on the index scale `scale` sits, `along` the index axis on the
    /// screen.
    pub(crate) fn place(&mut self, scale: &ScaleId, value: Scalar, along: Along) -> IndexPlace {
        let (scale, _) = self.scales.insert_full(scale.clone());
        IndexPlace { scale, value, along }
    }

    /// Adds `shape`, filled with `color`, for the datum `datum` of the current controller's
    /// dataset at `dataset`, or for a polyline (whose vertices name their data, so `datum` is
    /// `None`), and returns it, for the controller to draw.
    pub(crate) fn add(&mut self, dataset: usize, datum: Option<Datum>, color: Color, shape: Shape) -> &Item {
        let dataset = DatasetKey { controller: self.controller, dataset };
        self.items.push(Item { dataset, color, shape, datum });
        self.items.last().unwrap()
    }

    /// Where the datum `key` is drawn: the centre of its shape, or its point on its line, or
    /// `None` when it wasn't drawn.
    #[cfg_attr(not(test), allow(dead_code, reason = "called by pointer input, Interaction 3 in docs/multiple-controllers-and-scales.md, which isn't built yet"))]
    pub(crate) fn anchor(&self, key: DatumKey) -> Option<Position> {
        self.anchors.iter().find(|anchor| anchor.key == key).map(|anchor| anchor.center)
    }

    /// Every shape, in the order they were drawn.
    #[cfg_attr(not(test), allow(dead_code, reason = "read by the tests, and by animation later"))]
    pub(crate) fn items(&self) -> &[Item] {
        &self.items
    }

    /// Collects each datum's anchors, by dataset, once every shape is added, for `pick`.
    pub(crate) fn finish(&mut self) {
        let mut anchors: Vec<(DatasetKey, Anchor)> = vec![];

        for item in &self.items {
            match &item.shape {
                Shape::Polyline(line) => {
                    for (position, datum) in &line.vertices {
                        anchors.push((item.dataset, Anchor { key: item.dataset.datum(datum.index), center: *position, place: datum.place.clone() }));
                    }
                }
                shape => {
                    if let (Some(datum), Some(center)) = (&item.datum, shape.center()) {
                        anchors.push((item.dataset, Anchor { key: item.dataset.datum(datum.index), center, place: datum.place.clone() }));
                    }
                }
            }
        }

        // By dataset, in the order each dataset's anchors were added: a line's points, then its
        // markers, which sit on the same places.
        anchors.sort_by_key(|(dataset, _)| *dataset);

        self.anchors.clear();
        self.groups.clear();
        let mut start = 0;
        while start < anchors.len() {
            let dataset = anchors[start].0;
            let end = start + anchors[start..].iter().take_while(|(other, _)| *other == dataset).count();
            self.groups.push(Group { anchors: start..end, sorted: sorted_along(&anchors[start..end]) });
            start = end;
        }
        self.anchors = anchors.into_iter().map(|(_, anchor)| anchor).collect();
    }

    /// The data at `position`, as `mode` finds them, each once. Nothing is found from outside the
    /// chart area, nor, but for `Point`, data whose anchors lie outside it.
    pub(crate) fn pick(&self, position: Position, mode: PickMode) -> Vec<DatumKey> {
        if !inside(self.area, position) {
            return vec![];
        }

        match mode {
            PickMode::Point => self.under(position),
            PickMode::Nearest(measure) => self.nearest(position, measure),
            PickMode::Index => self.at_index(position),
            PickMode::Dataset => {
                let found = self.under(position).first().copied().or_else(|| self.nearest(position, Measure::Both).first().copied());
                let Some(found) = found else {
                    return vec![];
                };

                let mut keys: Vec<DatumKey> = self.anchors.iter()
                    .filter(|anchor| anchor.key.controller == found.controller && anchor.key.dataset == found.dataset)
                    .map(|anchor| anchor.key)
                    .collect();
                keys.sort();
                keys.dedup();
                keys
            }
        }
    }

    /// The data whose shapes are under `position`, the one drawn last first.
    fn under(&self, position: Position) -> Vec<DatumKey> {
        let mut found = vec![];
        for item in self.items.iter().rev() {
            if let Some(Hit { vertex, .. }) = item.shape.hit(position)
                && let Some(datum) = item.datum(vertex) {
                    found.push(item.dataset.datum(datum.index));
                }
        }
        unique(found)
    }

    /// The data whose anchors are nearest `position`, as `measure` measures, inside the chart area.
    fn nearest(&self, position: Position, measure: Measure) -> Vec<DatumKey> {
        let center = self.circle.map(|circle| circle.center);
        let measured = |anchor: &Anchor| -> Scalar {
            match (measure, center) {
                (Measure::X, _) => (position.x - anchor.center.x).abs(),
                (Measure::Y, _) => (position.y - anchor.center.y).abs(),
                (Measure::Angle, Some(center)) => {
                    let (dx, dy) = (anchor.center.x - center.x, anchor.center.y - center.y);
                    Along::Angle { center, angle: dx.atan2(-dy) }.gap(position)
                }
                (Measure::Distance, Some(center)) => Along::Distance { center, distance: distance(anchor.center, center) }.gap(position),
                _ => distance(position, anchor.center),
            }
        };

        let mut best = Scalar::INFINITY;
        let mut found = vec![];
        for anchor in self.anchors.iter().filter(|anchor| inside(self.area, anchor.center)) {
            let distance = measured(anchor);
            if distance < best - 1e-9 {
                best = distance;
                found.clear();
            }
            if distance <= best + 1e-9 {
                found.push(anchor.key);
            }
        }
        unique(found)
    }

    /// The data at the index nearest `position` along the index axis, in every dataset, ordered
    /// by controller, dataset and index.
    fn at_index(&self, position: Position) -> Vec<DatumKey> {
        // The nearest place along the index axis: a binary search in datasets in order along a
        // side, a scan in the others.
        let mut best: Option<(Scalar, &IndexPlace)> = None;
        for group in &self.groups {
            for anchor in self.candidates(group, position) {
                let Some(place) = &anchor.place else {
                    continue;
                };
                if !inside(self.area, anchor.center) {
                    continue;
                }
                let gap = place.along.gap(position);
                if best.is_none_or(|(best, _)| gap < best) {
                    best = Some((gap, place));
                }
            }
        }

        let Some((_, target)) = best else {
            return vec![];
        };

        let mut found: Vec<DatumKey> = vec![];
        for group in &self.groups {
            for anchor in self.at_place(group, target) {
                found.push(anchor.key);
            }
        }
        found.sort();
        found.dedup();
        found
    }

    /// The anchors of `group` that may be nearest `position` along the index axis: the two either
    /// side of it when the group is in order along a side, else all of them.
    fn candidates<'a>(&'a self, group: &Group, position: Position) -> &'a [Anchor] {
        let anchors = &self.anchors[group.anchors.clone()];
        let Some((along_x, sign)) = group.sorted else {
            return anchors;
        };

        let target = sign * if along_x { position.x } else { position.y };
        let split = anchors.partition_point(|anchor| sign * coordinate(anchor) < target);
        &anchors[split.saturating_sub(1)..(split + 1).min(anchors.len())]
    }

    /// The anchors of `group` at the same place as `target`: on the same scale, at the same value.
    fn at_place<'a>(&'a self, group: &Group, target: &'a IndexPlace) -> impl Iterator<Item=&'a Anchor> + 'a {
        let mut anchors = &self.anchors[group.anchors.clone()];

        // In order along the side the target is on, the anchors at its coordinate are together.
        if let (Some((along_x, sign)), Some((target_x, at))) = (group.sorted, target.along.coordinate())
            && along_x == target_x {
                let start = anchors.partition_point(|anchor| sign * coordinate(anchor) < sign * at - 0.5);
                let end = anchors.partition_point(|anchor| sign * coordinate(anchor) <= sign * at + 0.5);
                anchors = &anchors[start..end.max(start)];
            }

        anchors.iter().filter(move |anchor| {
            anchor.place.as_ref().is_some_and(|place| place.scale == target.scale && (place.value - target.value).abs() < 1e-9)
        })
    }
}

/// The screen coordinate of an anchor's place along its side, or NaN without one.
fn coordinate(anchor: &Anchor) -> Scalar {
    anchor.place.as_ref().and_then(|place| place.along.coordinate()).map_or(Scalar::NAN, |(_, at)| at)
}

/// Whether one dataset's anchors lie in order along one side: all placed along the same side, and
/// never turning back.
fn sorted_along(anchors: &[(DatasetKey, Anchor)]) -> Option<(bool, Scalar)> {
    let coordinates: Vec<(bool, Scalar)> = anchors.iter()
        .map(|(_, anchor)| anchor.place.as_ref().and_then(|place| place.along.coordinate()))
        .collect::<Option<Vec<_>>>()?;

    let along_x = coordinates.first()?.0;
    if coordinates.iter().any(|(x, _)| *x != along_x) {
        return None;
    }

    let values: Vec<Scalar> = coordinates.iter().map(|(_, at)| *at).collect();
    if values.windows(2).all(|pair| pair[0] <= pair[1]) {
        Some((along_x, 1.0))
    } else if values.windows(2).all(|pair| pair[0] >= pair[1]) {
        Some((along_x, -1.0))
    } else {
        None
    }
}

/// Whether `position` is inside `area`, edges included.
fn inside(area: Rect, position: Position) -> bool {
    let (top, bottom) = (area.bottom().min(area.top()), area.bottom().max(area.top()));
    area.left() <= position.x && position.x <= area.right() && top <= position.y && position.y <= bottom
}

/// `keys` without repeats, keeping the first of each.
fn unique(keys: Vec<DatumKey>) -> Vec<DatumKey> {
    let mut seen = HashSet::new();
    keys.into_iter().filter(|key| seen.insert(*key)).collect()
}
