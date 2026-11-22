use carbide::draw::{Position, Scalar};
use carbide::environment::Environment;
use carbide::state::{IntoReadState, ReadState};
use crate::DataColor;
use crate::dataset::RangePoint;
use crate::mark::{BarCanvas, BarMark, BarSlots, LineCanvas, LineMark, Mark, Marked, Overlay, ScatterCanvas, ScatterMark};
use crate::scale::Coordinates;
use crate::sequence::Sequence;

/// Draws a whisker from each point's low to its high value, with a cap at each end, on top of the
/// dataset's glyph (see `RangeDataSetExt::error_bars`): through the middle of its bar in a bar
/// controller, and through its point in a line or scatter controller. On scatter points it also
/// draws a level whisker from the point's x low to its x high, where the point has them
/// (`RangePoint::x_low`, `x_high`); bar and line charts, whose x is the category, leave them out. On a stacked bar or line the
/// whisker moves with it: it keeps its distance from the point's value, around where the bar ends
/// or the line is drawn. A point missing its x, y, low or high has no whisker, as a missing value
/// has no bar either.
///
/// The whiskers are drawn in the chart's label colour, so they show on bars of any colour, unless
/// the dataset sets its own with `whisker_color`, a value or a state, synced with the dataset.
#[derive(Clone, Debug)]
pub struct ErrorBars<C: ReadState<T=DataColor> = DataColor> {
    /// How wide the caps are on bars, as a share of the bar's thickness.
    pub cap_percentage: Scalar,
    /// How wide the caps are on lines and scatter points, in pixels, about as wide as a point.
    pub line_cap_length: Scalar,
    /// How wide the lines are, in pixels.
    pub line_width: Scalar,
    /// The whiskers' own colour, or `None` for the chart's label colour.
    whisker_color: Option<C>,
}

impl Default for ErrorBars {
    fn default() -> ErrorBars {
        ErrorBars { cap_percentage: 0.4, line_cap_length: 8.0, line_width: 2.0, whisker_color: None }
    }
}

impl<C: ReadState<T=DataColor>> ErrorBars<C> {
    /// The whiskers' own colour as it was last synced, or `None` for the chart's label colour.
    pub fn whisker_color(&self) -> Option<DataColor> {
        self.whisker_color.as_ref().map(|color| *color.value())
    }
}

impl<C: ReadState<T=DataColor>> Mark for ErrorBars<C> {
    type Role = Overlay;

    fn sync(&mut self, env: &mut Environment) -> bool {
        match &mut self.whisker_color {
            Some(color) => color.sync(env),
            None => false,
        }
    }
}

impl<D, C: ReadState<T=DataColor>> Marked<D, ErrorBars<C>> {
    /// Draws the whiskers in `color`, a value or a state, instead of the chart's label colour: a
    /// fixed colour, a palette colour (`DataColor::Palette`), which follows the chart's
    /// `SeriesStyle`, or the dataset's colour (`DataColor::ByDataSet`). The whiskers of a dataset
    /// are drawn in one colour, so a colour by point is the dataset's colour too. Set it right
    /// after `.error_bars()`, before other options.
    ///
    /// ```
    /// use carbide_chart::{BarController, DataColor, Estimate, RangeDataSetExt};
    ///
    /// let measured = vec![Estimate { x: 0.0, y: 6.0, low: 5.0, high: 7.0 }];
    /// BarController::new(measured.error_bars().whisker_color(DataColor::Palette(3)));
    /// ```
    pub fn whisker_color<C2: IntoReadState<DataColor>>(self, color: C2) -> Marked<D, ErrorBars<C2::Output>> {
        self.map_mark(|mark| ErrorBars {
            cap_percentage: mark.cap_percentage,
            line_cap_length: mark.line_cap_length,
            line_width: mark.line_width,
            whisker_color: Some(color.into_read_state()),
        })
    }
}

/// Calls `f` with each point's x, y, low and high, for the points that have a whisker: those with
/// all four.
fn whiskers<D: Sequence<dyn RangePoint>>(data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar, Scalar)) {
    data.foreach(&mut |_index, point| {
        if let (Some(x), Some(y), Some(low), Some(high)) = (coordinates.x(point), coordinates.y(point), point.low(), point.high()) {
            f(x, y, low, high);
        }
    });
}

impl<D: Sequence<dyn RangePoint>, C: ReadState<T=DataColor>> BarMark<D> for ErrorBars<C> {
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        whiskers(data, &coordinates, &mut |x, _, low, high| f(x, low.min(high), low.max(high)));
    }

    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
        let coordinates = canvas.coordinates;
        canvas.begin_lines();

        whiskers(data, &coordinates, &mut |x, _, low, high| {
            let slot = slots.slot(x);
            let middle = slot.middle();
            let cap = slot.thickness * self.cap_percentage / 2.0;
            let (low, high) = (canvas.value_pixel(low + slot.stack_offset), canvas.value_pixel(high + slot.stack_offset));

            canvas.add_line(Position::new(middle, low), Position::new(middle, high));
            canvas.add_line(Position::new(middle - cap, low), Position::new(middle + cap, low));
            canvas.add_line(Position::new(middle - cap, high), Position::new(middle + cap, high));
        });

        let color = match self.whisker_color() {
            Some(color) => canvas.resolve(color),
            None => canvas.ink(),
        };
        canvas.stroke_lines_added(self.line_width, color);
    }
}

/// Whiskers through a line's points, each the same length across, on top of every dataset's lines
/// and points. Around a circle they run along each point's spoke, or around its ring on a lying
/// radar, whose values run around the circle (see `LineCanvas::add_whisker`).
impl<D: Sequence<dyn RangePoint>, C: ReadState<T=DataColor>> LineMark<D> for ErrorBars<C> {
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        whiskers(data, &coordinates, &mut |x, _, low, high| f(x, low.min(high), low.max(high)));
    }

    fn draw(&self, data: &D, canvas: &mut LineCanvas) {
        let coordinates = canvas.coordinates;
        canvas.begin_lines();

        whiskers(data, &coordinates, &mut |x, y, low, high| {
            let offset = canvas.stack_offset(x);
            canvas.add_whisker(x, y + offset, low + offset, high + offset, self.line_cap_length);
        });

        let color = match self.whisker_color() {
            Some(color) => canvas.resolve(color),
            None => canvas.ink(),
        };
        canvas.stroke_lines_added(self.line_width, color);
    }
}

/// Whiskers through scatter points along both axes, each the same length across, on top of every
/// dataset's points: upright from the point's low to its high y, and level from its x low to its x
/// high (`RangePoint::x_low`, `x_high`). Each is drawn only where both its ends are there, so a
/// point has either whisker, both or neither. Around a circle, the y whisker runs along the point's
/// spoke and the x whisker around its ring (see `ScatterCanvas::add_whisker`).
impl<D: Sequence<dyn RangePoint>, C: ReadState<T=DataColor>> ScatterMark<D> for ErrorBars<C> {
    /// Both ends of each whisker, so the scales cover them as well as the points.
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar)) {
        scatter_whiskers(data, &coordinates, &mut |x, y, along_y, along_x| {
            if let Some((low, high)) = along_y {
                f(x, low);
                f(x, high);
            }
            if let Some((low, high)) = along_x {
                f(low, y);
                f(high, y);
            }
        });
    }

    fn draw(&self, data: &D, canvas: &mut ScatterCanvas) {
        let coordinates = canvas.coordinates;
        canvas.begin_lines();

        scatter_whiskers(data, &coordinates, &mut |x, y, along_y, along_x| {
            if let Some((low, high)) = along_y {
                canvas.add_whisker((x, y), (x, low), (x, high), self.line_cap_length);
            }

            if let Some((low, high)) = along_x {
                canvas.add_whisker((x, y), (low, y), (high, y), self.line_cap_length);
            }
        });

        let color = match self.whisker_color() {
            Some(color) => canvas.resolve(color),
            None => canvas.ink(),
        };
        canvas.stroke_lines_added(self.line_width, color);
    }
}

/// Calls `f` with each point's x and y, and its interval along y and along x, each `None` when one
/// of its ends is missing, for the points that have an x and a y.
fn scatter_whiskers<D: Sequence<dyn RangePoint>>(data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Option<(Scalar, Scalar)>, Option<(Scalar, Scalar)>)) {
    data.foreach(&mut |_index, point| {
        if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
            f(x, y, point.low().zip(point.high()), point.x_low().zip(point.x_high()));
        }
    });
}
