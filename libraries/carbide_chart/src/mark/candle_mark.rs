use carbide::color::Color;
use carbide::draw::{Position, Scalar};
use carbide::environment::Environment;
use carbide::state::{IntoReadState, ReadState};
use crate::DataColor;
use crate::dataset::OhlcPoint;
use crate::legend::LegendMarker;
use crate::mark::{BarCanvas, BarMark, BarSlots, LegendContext, Mark, Marked, Primary};
use crate::scale::Coordinates;
use crate::sequence::Sequence;

/// How a `CandleMark` draws each period.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CandleGlyph {
    /// A filled body from the open to the close, and a wick from the low to the high through it.
    Candlestick,
    /// A line from the low to the high, with a tick to one side at the open and to the other at the
    /// close: to the left and to the right for standing bars.
    Ohlc,
}

/// Draws each point's open, high, low and close in place of a bar, as a candlestick or an OHLC
/// bar (see `OhlcDataSetExt`). A period rises when its close is above its open, and falls
/// otherwise, a flat one included, and is drawn in the rising or the falling colour: the
/// `SeriesStyle`'s (`SeriesStyle::rising_color`, `falling_color`), unless the dataset sets its own
/// with `rising_color` and `falling_color`, each a value or a state. A point missing any of its
/// values isn't drawn.
#[derive(Clone, Debug)]
pub struct CandleMark<R: ReadState<T=DataColor> = DataColor, F: ReadState<T=DataColor> = DataColor> {
    glyph: CandleGlyph,
    rising_color: Option<R>,
    falling_color: Option<F>,
    /// How wide the lines are, in pixels: the wicks, and an OHLC bar's lines.
    line_width: Scalar,
}

impl CandleMark {
    pub(crate) fn new(glyph: CandleGlyph) -> CandleMark {
        CandleMark { glyph, rising_color: None, falling_color: None, line_width: 2.0 }
    }
}

impl<R: ReadState<T=DataColor>, F: ReadState<T=DataColor>> CandleMark<R, F> {
    /// Whether it draws candlesticks or OHLC bars.
    pub fn glyph(&self) -> CandleGlyph {
        self.glyph
    }

    /// The dataset's own colour for rising periods as it was last synced, or `None` for the
    /// `SeriesStyle`'s.
    pub fn rising_color(&self) -> Option<DataColor> {
        self.rising_color.as_ref().map(|color| *color.value())
    }

    /// The dataset's own colour for falling periods as it was last synced, or `None` for the
    /// `SeriesStyle`'s.
    pub fn falling_color(&self) -> Option<DataColor> {
        self.falling_color.as_ref().map(|color| *color.value())
    }

    /// The colour the period at `point_index` is drawn in.
    fn color(&self, rising: bool, point_index: usize, canvas: &mut BarCanvas) -> Color {
        match (rising, self.rising_color(), self.falling_color()) {
            (true, Some(color), _) | (false, _, Some(color)) => canvas.point_color(point_index, color),
            (true, None, _) => canvas.rising_color(),
            (false, _, None) => canvas.falling_color(),
        }
    }
}

impl<R: ReadState<T=DataColor>, F: ReadState<T=DataColor>> Mark for CandleMark<R, F> {
    type Role = Primary;

    fn sync(&mut self, env: &mut Environment) -> bool {
        let rising = self.rising_color.as_mut().is_some_and(|color| color.sync(env));
        let falling = self.falling_color.as_mut().is_some_and(|color| color.sync(env));
        rising | falling
    }
}

impl<D, R: ReadState<T=DataColor>, F: ReadState<T=DataColor>> Marked<D, CandleMark<R, F>> {
    /// Draws the rising periods in `color`, a value or a state, instead of the `SeriesStyle`'s
    /// rising colour. Set it right after `.candles()` or `.ohlc()`, before other options.
    pub fn rising_color<R2: IntoReadState<DataColor>>(self, color: R2) -> Marked<D, CandleMark<R2::Output, F>> {
        self.map_mark(|mark| CandleMark { glyph: mark.glyph, rising_color: Some(color.into_read_state()), falling_color: mark.falling_color, line_width: mark.line_width })
    }

    /// Draws the falling periods in `color`, a value or a state, instead of the `SeriesStyle`'s
    /// falling colour. Set it right after `.candles()` or `.ohlc()`, before other options.
    pub fn falling_color<F2: IntoReadState<DataColor>>(self, color: F2) -> Marked<D, CandleMark<R, F2::Output>> {
        self.map_mark(|mark| CandleMark { glyph: mark.glyph, rising_color: mark.rising_color, falling_color: Some(color.into_read_state()), line_width: mark.line_width })
    }
}

/// A point's x, open, high, low and close, when none is missing.
fn prices(coordinates: &Coordinates, point: &dyn OhlcPoint) -> Option<(Scalar, Scalar, Scalar, Scalar, Scalar)> {
    Some((coordinates.x(point)?, point.open()?, point.high()?, point.low()?, point.close()?))
}

impl<D: Sequence<dyn OhlcPoint>, R: ReadState<T=DataColor>, F: ReadState<T=DataColor>> BarMark<D> for CandleMark<R, F> {
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.foreach(&mut |_index, point| {
            if let Some((x, open, high, low, close)) = prices(&coordinates, point) {
                f(x, low.min(high).min(open).min(close), high.max(low).max(open).max(close));
            }
        });
    }

    /// A square split into the rising colour on the left and the falling colour on the right, as
    /// the dataset's own palette colour isn't drawn.
    fn legend(&self, _data: &D, legend: &LegendContext) -> Option<(Color, LegendMarker)> {
        let rising = self.rising_color().map_or_else(|| legend.rising_color(), |color| legend.resolve(color));
        let falling = self.falling_color().map_or_else(|| legend.falling_color(), |color| legend.resolve(color));
        Some((rising, LegendMarker::Split(falling)))
    }

    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
        let coordinates = canvas.coordinates;
        data.foreach(&mut |point_index, point| {
            let Some((x, open, high, low, close)) = prices(&coordinates, point) else {
                return;
            };

            let color = self.color(close > open, point_index, canvas);
            let slot = slots.slot(x);
            let middle = slot.middle();
            let [open, high, low, close] = [open, high, low, close].map(|value| canvas.value_pixel(value));
            // Picked anywhere from its low to its high, wicks included.
            canvas.add_rect(point_index, x, &slot, low, high, color);

            match self.glyph {
                CandleGlyph::Candlestick => {
                    canvas.stroke_lines(&[(Position::new(middle, low), Position::new(middle, high))], self.line_width, color);

                    // A flat body would vanish, so it is drawn as a line across the slot.
                    if (open - close).abs() < 1.0 {
                        canvas.stroke_lines(&[(Position::new(slot.side, open), Position::new(slot.side + slot.thickness, open))], self.line_width, color);
                    } else {
                        canvas.fill_rect(slot.side, slot.thickness, open, close, color);
                    }
                }
                CandleGlyph::Ohlc => {
                    canvas.stroke_lines(&[
                        (Position::new(middle, low), Position::new(middle, high)),
                        (Position::new(slot.side, open), Position::new(middle, open)),
                        (Position::new(middle, close), Position::new(slot.side + slot.thickness, close)),
                    ], self.line_width, color);
                }
            }
        });
    }
}
