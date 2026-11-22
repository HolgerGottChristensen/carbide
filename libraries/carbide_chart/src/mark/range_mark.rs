use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::{IntoReadState, ReadState};
use crate::DataColor;
use crate::dataset::RangePoint;
use crate::mark::{BarCanvas, BarMark, BarSlots, LineCanvas, LineMark, Mark, Marked, Primary};
use crate::scale::Coordinates;
use crate::sequence::Sequence;

/// Draws a dataset's ranges, from each point's low to its high value (see `RangeDataSetExt::range`):
/// in a bar controller, each point as a floating bar in place of a bar growing from zero; in a
/// line controller, a band between the line through the lows and the line through the highs, in
/// place of the line. A point missing its x, low or high isn't drawn, and leaves a gap in a band.
///
/// Where a range is inverted, its high below its low, it is drawn like any other by default. With
/// `inverted_color`, those parts take another colour: the floating bars whose high is below their
/// low, and the parts of a band where the line through the highs runs below the line through the
/// lows, so a crossing shows. The colour is a value or a state, synced with the dataset.
#[derive(Clone, Debug)]
pub struct RangeMark<C: ReadState<T=DataColor> = DataColor> {
    /// The colour of the inverted parts, or `None` to draw them as the rest.
    inverted_color: Option<C>,
}

impl Default for RangeMark {
    fn default() -> RangeMark {
        RangeMark { inverted_color: None }
    }
}

impl<C: ReadState<T=DataColor>> RangeMark<C> {
    /// The colour of the inverted parts as it was last synced, or `None` to draw them as the rest.
    pub fn inverted_color(&self) -> Option<DataColor> {
        self.inverted_color.as_ref().map(|color| *color.value())
    }
}

impl<C: ReadState<T=DataColor>> Mark for RangeMark<C> {
    type Role = Primary;

    fn sync(&mut self, env: &mut Environment) -> bool {
        match &mut self.inverted_color {
            Some(color) => color.sync(env),
            None => false,
        }
    }
}

impl<D, C: ReadState<T=DataColor>> Marked<D, RangeMark<C>> {
    /// Draws the inverted parts of the ranges in `color`, a value or a state: floating bars whose
    /// high is below their low, and the parts of a band where the highs run below the lows (see
    /// `RangeMark`). A palette colour (`DataColor::Palette`) follows the chart's `SeriesStyle`. Set
    /// it right after `.range()`, before other options.
    pub fn inverted_color<C2: IntoReadState<DataColor>>(self, color: C2) -> Marked<D, RangeMark<C2::Output>> {
        self.map_mark(|_| RangeMark { inverted_color: Some(color.into_read_state()) })
    }
}

/// Floating bars, each in its point's colour, or in the inverted colour where its high is below its
/// low and one is set.
impl<D: Sequence<dyn RangePoint>, C: ReadState<T=DataColor>> BarMark<D> for RangeMark<C> {
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(low), Some(high)) = (coordinates.x(point), point.low(), point.high()) {
                f(x, low.min(high), low.max(high));
            }
        });
    }

    /// Floating bars are wedges around a circle, from the low's ring to the high's.
    fn draws_around_a_circle(&self) -> bool {
        true
    }

    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
        let coordinates = canvas.coordinates;
        data.foreach(&mut |point_index, point| {
            let (Some(x), Some(low), Some(high)) = (coordinates.x(point), point.low(), point.high()) else {
                return;
            };

            let slot = slots.slot(x);
            let color = match self.inverted_color() {
                Some(inverted) if high < low => canvas.point_color(point_index, inverted),
                _ => canvas.point_color(point_index, point.color()),
            };
            let (from, to) = (canvas.value_pixel(low), canvas.value_pixel(high));
            canvas.add_rect(point_index, x, &slot, from, to, color);
            canvas.fill_rect(slot.side, slot.thickness, from, to, color);
        });
    }
}

/// A band in place of the line: filled between the lows and the highs, translucent, with both
/// edges stroked in the dataset's colour, and the parts where the highs run below the lows filled
/// in the inverted colour, when one is set. The edges follow the dataset's `interpolation`, and the
/// band breaks at missing points as the dataset's `span_gaps` says, as a line does.
impl<D: Sequence<dyn RangePoint>, C: ReadState<T=DataColor>> LineMark<D> for RangeMark<C> {
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(low), Some(high)) = (coordinates.x(point), point.low(), point.high()) {
                f(x, low.min(high), low.max(high));
            }
        });
    }

    fn draw(&self, data: &D, canvas: &mut LineCanvas) {
        let coordinates = canvas.coordinates;
        let mut xs = vec![];
        let mut highs = vec![];
        let mut lows = vec![];

        data.foreach(&mut |_index, point| {
            if let (Some(x), Some(low), Some(high)) = (coordinates.x(point), point.low(), point.high()) {
                xs.push(Some(x));
                highs.push(canvas.position(x, high));
                lows.push(canvas.position(x, low));
            } else {
                xs.push(None);
            }
        });

        let color = canvas.color();
        let fill = canvas.translucent(color);
        let inverted = self.inverted_color().map(|inverted| {
            let inverted = canvas.resolve(inverted);
            canvas.translucent(inverted)
        });
        let runs = canvas.runs(&xs);
        // Around a full circle, a band in one run is a ring between two loops.
        let closed = canvas.closes(&runs, &xs);

        for run in &runs {
            canvas.fill_band(&highs[run.clone()], &lows[run.clone()], fill, inverted, closed);
        }

        for run in &runs {
            canvas.stroke_line(&highs[run.clone()], color, closed);
            canvas.stroke_line(&lows[run.clone()], color, closed);
        }
    }
}
