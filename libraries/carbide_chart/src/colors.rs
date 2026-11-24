//! The colours datasets are drawn in: the chart's `SeriesStyle`, read once per draw, and the rules
//! that turn a dataset's or point's `DataColor` into a colour. The built-in controllers, the mark
//! canvases and controllers of your own all colour their datasets through these types, so a
//! dataset is coloured the same way whatever draws it.

use carbide::color::Color;
use carbide::draw::color::{rgba, Rgba};
use carbide::environment::{Environment, EnvironmentKey, Shared};
use crate::DataColor;
use crate::style::{axis_style, series_style};

/// The colours a controller draws its datasets in: the chart's `SeriesStyle` (its palette, fill
/// opacity, and rising, falling and outline colours) and the `AxisStyle`'s label colour, for marks
/// drawn over the data, read once from the environment, with the controller's palette offset.
///
/// Build one where the controller draws, reports its legend entries or describes a datum, from the
/// environment there (`ctx.env()` in `DatasetController::draw`), and don't keep it: the theme and
/// the styles can change between renders. It holds no borrow of the environment, so it can be kept
/// across drawing calls that need the canvas mutably.
///
/// ```
/// use carbide_chart::{DataColor, SeriesColors};
/// use carbide_core::environment::Environment;
///
/// let env = Environment::new();
/// // A controller starting one colour into the palette.
/// let colors = SeriesColors::new(&env, 1);
///
/// // Its first dataset, coloured by dataset, takes the palette's second colour.
/// assert_eq!(colors.dataset(DataColor::ByDataSet, 0).color(), colors.palette(1));
/// ```
#[derive(Clone, Debug)]
pub struct SeriesColors {
    palette: Vec<Color>,
    offset: usize,
    fill_opacity: f32,
    rising: Color,
    falling: Color,
    outline: Color,
    ink: Color,
    /// The indices of the controller's datasets drawn dimmed, while the chart shows which
    /// datasets are on a hovered scale.
    dimmed: Vec<usize>,
    dimmed_opacity: f32,
}

impl SeriesColors {
    /// The colours of the styles in `env`, for a controller whose datasets start `palette_offset`
    /// colours into the palette (see `BarController::palette_offset`).
    pub fn new(env: &Environment, palette_offset: usize) -> SeriesColors {
        let series = series_style(env);

        SeriesColors {
            palette: series.palette(env),
            offset: palette_offset,
            fill_opacity: series.fill_opacity(),
            rising: series.rising_color(env),
            falling: series.falling_color(env),
            outline: series.outline_color(env),
            ink: axis_style(env).label_color(env),
            dimmed: env.get::<DimmedDatasetsKey>().cloned().unwrap_or_default(),
            dimmed_opacity: series.dimmed_opacity(),
        }
    }

    /// The colours of the dataset at `dataset_index` in the controller, coloured `color` (its
    /// `DataSetOptions::color`). Count every dataset of the controller, shown or hidden, so a
    /// dataset keeps its colour when another is hidden from the legend.
    ///
    /// While the chart shows which datasets are on a hovered scale, the colours of the datasets
    /// on other scales come at the series style's dimmed opacity (`SeriesStyle::dimmed_opacity`),
    /// so a controller that colours its datasets here dims them without doing anything else.
    pub fn dataset(&self, color: DataColor, dataset_index: usize) -> DatasetColors<'_> {
        DatasetColors { series: self, color, dataset_index }
    }

    /// The palette's colour at `index`, wrapping around at its end, whatever the palette offset:
    /// the colour `DataColor::Palette(index)` picks. An empty palette gives the default colour.
    pub fn palette(&self, index: usize) -> Color {
        if self.palette.is_empty() {
            Color::default()
        } else {
            self.palette[index % self.palette.len()]
        }
    }

    /// `color` at the `SeriesStyle`'s fill opacity, for shapes the grid and the data behind them
    /// should show through, such as filled areas, bubbles and box plots' boxes.
    pub fn translucent(&self, color: Color) -> Color {
        let Rgba(r, g, b, a) = Rgba::from(color);
        rgba(r, g, b, a * self.fill_opacity)
    }

    /// The `SeriesStyle`'s fill opacity, from 0 to 1 (see `translucent`).
    pub fn fill_opacity(&self) -> f32 {
        self.fill_opacity
    }

    /// The `SeriesStyle`'s colour for rising candlesticks and OHLC bars.
    pub fn rising(&self) -> Color {
        self.rising
    }

    /// The `SeriesStyle`'s colour for falling candlesticks and OHLC bars.
    pub fn falling(&self) -> Color {
        self.falling
    }

    /// The `SeriesStyle`'s colour for outlines between neighbouring shapes, such as a pie's slices.
    pub fn outline(&self) -> Color {
        self.outline
    }

    /// The colour for marks drawn over the datasets' colours, such as error bars: the
    /// `AxisStyle`'s label colour, so they stand out on any dataset.
    pub fn ink(&self) -> Color {
        self.ink
    }

    /// The palette colour for a dataset or point by its `index` in its controller or dataset,
    /// shifted by the palette offset.
    fn by_index(&self, index: usize) -> Color {
        self.palette(self.offset + index)
    }
}

/// One dataset's colours (see `SeriesColors::dataset`), by the same rules for every controller:
///
/// - a fixed colour (`DataColor::Color`) is itself, and a picked one (`DataColor::Palette`) the
///   palette's colour at its index;
/// - a dataset coloured by dataset takes the palette colour for its index in the controller,
///   shifted by the controller's palette offset;
/// - a dataset coloured by point gives each point the palette colour for the point's index in the
///   dataset, shifted the same way, while what is drawn once for the whole dataset, such as a line,
///   its area and its legend entry, keeps the colour by dataset;
/// - a point's own colour (`DataPoint::color`) wins over its dataset's.
#[derive(Copy, Clone, Debug)]
pub struct DatasetColors<'a> {
    series: &'a SeriesColors,
    color: DataColor,
    dataset_index: usize,
}

impl<'a> DatasetColors<'a> {
    /// The colour of what is drawn once for the whole dataset, such as a line, its area, or its
    /// legend entry.
    pub fn color(&self) -> Color {
        self.dim(match self.color {
            DataColor::ByDataSet | DataColor::ByPoint => self.series.by_index(self.dataset_index),
            DataColor::Palette(index) => self.series.palette(index),
            DataColor::Color(color) => color,
        })
    }

    /// The colour of one bar or point at `point_index` in the dataset, whose own colour is
    /// `color` (`DataPoint::color`).
    pub fn point(&self, point_index: usize, color: DataColor) -> Color {
        match (color, self.color) {
            (DataColor::Color(color), _) => self.dim(color),
            (DataColor::Palette(index), _) => self.dim(self.series.palette(index)),
            (DataColor::ByPoint, _) | (DataColor::ByDataSet, DataColor::ByPoint) => self.dim(self.series.by_index(point_index)),
            (DataColor::ByDataSet, _) => self.color(),
        }
    }

    /// What `color` stands for in this dataset, without a point: a fixed colour, a picked palette
    /// colour, or, for a colour by dataset or by point, the dataset's colour (`color`). Marks
    /// resolve their colour options with it, such as a range's inverted colour.
    pub fn resolve(&self, color: DataColor) -> Color {
        match color {
            DataColor::Color(color) => self.dim(color),
            DataColor::Palette(index) => self.dim(self.series.palette(index)),
            DataColor::ByDataSet | DataColor::ByPoint => self.color(),
        }
    }

    /// `color`, at the dimmed opacity when this dataset is dimmed.
    fn dim(&self, color: Color) -> Color {
        if !self.series.dimmed.contains(&self.dataset_index) {
            return color;
        }

        let Rgba(r, g, b, a) = Rgba::from(color);
        rgba(r, g, b, a * self.series.dimmed_opacity)
    }

    /// The colours of the whole controller, for the styles' other colours.
    pub fn series(&self) -> &'a SeriesColors {
        self.series
    }
}

/// The indices of the datasets of the controller being drawn that the chart dims, while it shows
/// which datasets are on a hovered scale. The chart scopes it around each controller's draw.
#[derive(Debug, Copy, Clone)]
pub(crate) struct DimmedDatasetsKey;

impl EnvironmentKey for DimmedDatasetsKey {
    type Value = Vec<usize>;
    type Access = Shared;
}

#[cfg(test)]
mod tests {
    use carbide::color::{Color, BLUE, GREEN, RED, WHITE};
    use carbide::draw::color::Rgba;
    use carbide::environment::Environment;
    use crate::DataColor;
    use crate::style::{axis_style, series_style};
    use super::SeriesColors;

    /// Colours with a palette of white, green and red, starting `offset` colours in.
    fn colors(offset: usize) -> SeriesColors {
        SeriesColors { palette: vec![WHITE, GREEN, RED], offset, fill_opacity: 0.5, rising: GREEN, falling: RED, outline: WHITE, ink: BLUE, dimmed: vec![], dimmed_opacity: 0.25 }
    }

    /// A dataset coloured `ByPoint` gives each point the palette colour for its index in the
    /// dataset, wrapping at the palette's end, while what is drawn once for the whole dataset
    /// keeps the palette colour for the dataset's index. A point's own colour always wins.
    #[test]
    fn points_coloured_by_point_cycle_through_the_palette() {
        let colors = colors(0);
        let by_point = colors.dataset(DataColor::ByPoint, 1);

        let colours: Vec<_> = (0..4).map(|point_index| by_point.point(point_index, DataColor::ByDataSet)).collect();
        assert_eq!(colours, vec![WHITE, GREEN, RED, WHITE], "by the point's index, not the dataset's");
        assert_eq!(by_point.color(), GREEN, "the line and its area keep the dataset's palette colour");

        assert_eq!(by_point.point(2, DataColor::Color(BLUE)), BLUE, "a point's own colour wins");
        assert_eq!(colors.dataset(DataColor::ByDataSet, 0).point(2, DataColor::ByPoint), RED, "a point can ask for its palette colour itself");
        assert_eq!(colors.dataset(DataColor::ByDataSet, 1).point(2, DataColor::ByDataSet), GREEN, "otherwise points take the dataset's colour");
        assert_eq!(colors.dataset(DataColor::Color(BLUE), 1).point(2, DataColor::ByDataSet), BLUE);
    }

    /// A controller's palette offset shifts the colours its datasets and points take by their
    /// index; a dataset or point that picks a palette colour itself keeps it, and a fixed colour
    /// stays fixed. Palette indices wrap around at the end.
    #[test]
    fn palette_offsets_shift_colours_by_index_but_not_picked_ones() {
        let colors = colors(2);

        assert_eq!(colors.dataset(DataColor::ByDataSet, 0).color(), RED, "dataset 0 starts two colours in");
        assert_eq!(colors.dataset(DataColor::ByDataSet, 1).color(), WHITE, "and wraps around");
        assert_eq!(colors.dataset(DataColor::ByPoint, 0).point(1, DataColor::ByDataSet), WHITE, "points by index are shifted too");

        assert_eq!(colors.dataset(DataColor::Palette(1), 0).color(), GREEN, "a picked colour isn't shifted");
        assert_eq!(colors.dataset(DataColor::Palette(4), 0).color(), GREEN, "picked indices wrap around");
        assert_eq!(colors.palette(4), GREEN, "nor is one read from the palette directly");
        assert_eq!(colors.dataset(DataColor::ByDataSet, 1).point(1, DataColor::Palette(0)), WHITE, "a point can pick a colour itself");
        assert_eq!(colors.dataset(DataColor::Palette(1), 1).point(1, DataColor::ByDataSet), GREEN, "its points take the dataset's picked colour");
        assert_eq!(colors.dataset(DataColor::Color(BLUE), 0).color(), BLUE);
    }

    /// Resolving a mark's colour option gives a fixed or picked colour as it is, and the dataset's
    /// colour for a colour by dataset or by point, which has no point to go by.
    #[test]
    fn colours_resolve_without_a_point_to_the_dataset_colour() {
        let colors = colors(1);
        let dataset = colors.dataset(DataColor::ByPoint, 0);

        assert_eq!(dataset.resolve(DataColor::Color(BLUE)), BLUE);
        assert_eq!(dataset.resolve(DataColor::Palette(0)), WHITE, "a picked colour isn't shifted");
        assert_eq!(dataset.resolve(DataColor::ByDataSet), GREEN, "the dataset's colour, shifted by the offset");
        assert_eq!(dataset.resolve(DataColor::ByPoint), GREEN, "no point to go by, so the dataset's colour");
    }

    /// The colours are read from the styles in the environment: the series style's palette, fill
    /// opacity and other colours, and the axis style's label colour as ink. An empty palette gives
    /// the default colour.
    #[test]
    fn colours_come_from_the_styles_in_the_environment() {
        let env = Environment::new();
        let colors = SeriesColors::new(&env, 0);
        let (series, axis) = (series_style(&env), axis_style(&env));

        assert_eq!(colors.palette, series.palette(&env));
        assert_eq!(colors.rising(), series.rising_color(&env));
        assert_eq!(colors.falling(), series.falling_color(&env));
        assert_eq!(colors.outline(), series.outline_color(&env));
        assert_eq!(colors.ink(), axis.label_color(&env));

        let Rgba(_, _, _, alpha) = Rgba::from(colors.translucent(RED));
        assert!((alpha - series.fill_opacity()).abs() < 1e-6, "translucent at the fill opacity: {alpha}");

        let empty = SeriesColors { palette: vec![], ..colors };
        assert_eq!(empty.dataset(DataColor::ByDataSet, 3).color(), Color::default());
    }

    /// The datasets the chart dims are coloured at the dimmed opacity, whatever their colour; the
    /// others keep theirs.
    #[test]
    fn dimmed_datasets_are_faded() {
        let colors = SeriesColors { dimmed: vec![1], ..colors(0) };
        let alpha = |color: Color| Rgba::from(color).3;

        assert_eq!(alpha(colors.dataset(DataColor::ByDataSet, 0).color()), 1.0, "not dimmed");
        assert_eq!(alpha(colors.dataset(DataColor::ByDataSet, 1).color()), 0.25);
        assert_eq!(alpha(colors.dataset(DataColor::Color(BLUE), 1).point(0, DataColor::ByPoint)), 0.25, "by point too");
        assert_eq!(alpha(colors.dataset(DataColor::ByDataSet, 1).resolve(DataColor::Color(RED))), 0.25, "and fixed colours");
    }
}
