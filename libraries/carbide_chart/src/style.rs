//! How a chart looks. Its axes take their colours from an `AxisStyle`, its datasets from a
//! `SeriesStyle`, and its tooltip from a `TooltipStyle`. Each is a style behind an environment key, like carbide's other widget styles,
//! so it is set on any ancestor of the charts it applies to (`ChartStyleExt`) and defaults to
//! `AutomaticStyle`, which follows the environment's theme.
//!
//! `DebugStyle` keeps the loud colours charts were first drawn with: a yellow grid, red and green
//! borders and a white, green and red palette, which make every part easy to tell apart.

use std::fmt::Debug;
use carbide::color::{Color, BLACK, GREEN, RED, WHITE, YELLOW};
use carbide::draw::{AutomaticStyle, DebugStyle, Scalar};
use carbide::draw::color::{rgba, Rgba};
use carbide::environment::{Environment, EnvironmentColor, EnvironmentKey, Shared};
use carbide::widget::{EnvUpdatingNew, Widget, WidgetStyle};
use dyn_clone::clone_trait_object;
use crate::scale::Axis;

/// The colours of a chart's axes: grid lines across the chart area, and the tick marks, labels
/// and border in each scale's band. Set it with `ChartStyleExt::axis_style`.
pub trait AxisStyle: WidgetStyle {
    /// The colour of grid lines across the chart area.
    fn grid_color(&self, env: &Environment) -> Color;

    /// The colour of grid lines at minor ticks (see `Scale::minor_ticks`), fainter than the
    /// others. The grid colour at half its opacity by default.
    fn minor_grid_color(&self, env: &Environment) -> Color {
        let Rgba(r, g, b, a) = Rgba::from(self.grid_color(env));
        rgba(r, g, b, a / 2.0)
    }

    /// The colour of tick marks, between the chart area and the labels.
    fn tick_color(&self, env: &Environment) -> Color;

    /// The colour of tick labels.
    fn label_color(&self, env: &Environment) -> Color;

    /// The colour of the border along the inner edge of a scale's band, for a scale along `axis`.
    fn border_color(&self, axis: Axis, env: &Environment) -> Color;

    /// The width of the border, in pixels.
    fn border_width(&self) -> Scalar {
        1.0
    }

    /// The colour a zoomable scale's band is filled with while the pointer is over it, showing that
    /// gestures there move only that scale. The environment's tertiary fill by default, a faint
    /// tint over the background.
    fn hovered_band_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::TertiarySystemFill)
    }
}

/// The colours of a chart's datasets. Set it with `ChartStyleExt::series_style`.
pub trait SeriesStyle: WidgetStyle {
    /// The colours datasets take in turn when they don't set one: dataset `i` of a controller
    /// takes colour `i`, wrapping around at the end, and so do the points of a dataset coloured
    /// by point (`DataColor::ByPoint`).
    fn palette(&self, env: &Environment) -> Vec<Color>;

    /// How opaque a filled area is (see `Fill`), from 0 to 1, so the grid and the areas behind
    /// it show through.
    fn fill_opacity(&self) -> f32 {
        0.3
    }

    /// The colour of a rising candlestick or OHLC bar, whose close is above its open (see
    /// `OhlcDataSetExt`). The environment's green by default.
    fn rising_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::Green)
    }

    /// The colour of a falling candlestick or OHLC bar, whose close isn't above its open. The
    /// environment's red by default.
    fn falling_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::Red)
    }

    /// The colour of the outlines between neighbouring shapes that would otherwise run into each
    /// other, such as a pie's slices (see `PieController::outline_width`): the environment's
    /// background by default, so a pie whose slices share a colour still shows them apart.
    fn outline_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::SystemBackground)
    }
}

/// The colours of a chart's tooltip (see `Chart::tooltip`): its background and border, its title
/// and its rows. Set it with `ChartStyleExt::tooltip_style`.
pub trait TooltipStyle: WidgetStyle {
    /// The colour the tooltip is filled with.
    fn background_color(&self, env: &Environment) -> Color;

    /// The colour of the tooltip's outline.
    fn border_color(&self, env: &Environment) -> Color;

    /// The colour of the tooltip's title.
    fn title_color(&self, env: &Environment) -> Color;

    /// The colour of the tooltip's rows.
    fn text_color(&self, env: &Environment) -> Color;

    /// The radius of the tooltip's corners, in pixels.
    fn corner_radius(&self) -> Scalar {
        6.0
    }
}

clone_trait_object!(AxisStyle);
clone_trait_object!(SeriesStyle);
clone_trait_object!(TooltipStyle);

/// Follows the environment's theme: grid lines and tick marks in its separator colours, labels in
/// its secondary label colour, and the series in its system colours.
impl AxisStyle for AutomaticStyle {
    fn grid_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::OpaqueSeparator)
    }

    fn tick_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::TertiaryLabel)
    }

    fn label_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::SecondaryLabel)
    }

    fn border_color(&self, _axis: Axis, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::TertiaryLabel)
    }
}

impl SeriesStyle for AutomaticStyle {
    fn palette(&self, env: &Environment) -> Vec<Color> {
        [
            EnvironmentColor::Blue,
            EnvironmentColor::Green,
            EnvironmentColor::Orange,
            EnvironmentColor::Red,
            EnvironmentColor::Purple,
            EnvironmentColor::Teal,
            EnvironmentColor::Yellow,
            EnvironmentColor::Pink,
            EnvironmentColor::Indigo,
        ]
            .into_iter()
            .map(|color| theme_color(env, color))
            .collect()
    }
}

/// A background a step from the system background's, outlined in the separator colour, with the
/// title in the label colour and the rows in the secondary label colour.
impl TooltipStyle for AutomaticStyle {
    fn background_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::SecondarySystemBackground)
    }

    fn border_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::OpaqueSeparator)
    }

    fn title_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::Label)
    }

    fn text_color(&self, env: &Environment) -> Color {
        theme_color(env, EnvironmentColor::SecondaryLabel)
    }
}

/// A black tooltip with a yellow outline and white text.
impl TooltipStyle for DebugStyle {
    fn background_color(&self, _env: &Environment) -> Color {
        BLACK
    }

    fn border_color(&self, _env: &Environment) -> Color {
        YELLOW
    }

    fn title_color(&self, _env: &Environment) -> Color {
        WHITE
    }

    fn text_color(&self, _env: &Environment) -> Color {
        WHITE
    }
}

/// A yellow grid, tick marks and labels, and thick borders, red along horizontal scales and green
/// along vertical ones, so every part of an axis stands out.
impl AxisStyle for DebugStyle {
    fn grid_color(&self, _env: &Environment) -> Color {
        YELLOW
    }

    fn tick_color(&self, _env: &Environment) -> Color {
        YELLOW
    }

    fn label_color(&self, _env: &Environment) -> Color {
        YELLOW
    }

    fn border_color(&self, axis: Axis, _env: &Environment) -> Color {
        match axis {
            Axis::Horizontal => RED,
            Axis::Vertical | Axis::Radial => GREEN,
        }
    }

    fn border_width(&self) -> Scalar {
        3.0
    }
}

/// White, green and red, with green rising and red falling candles.
impl SeriesStyle for DebugStyle {
    fn palette(&self, _env: &Environment) -> Vec<Color> {
        vec![WHITE, GREEN, RED]
    }

    fn rising_color(&self, _env: &Environment) -> Color {
        GREEN
    }

    fn falling_color(&self, _env: &Environment) -> Color {
        RED
    }
}

/// Sets how charts in a widget look. Any widget can set it, and it applies to every chart inside.
pub trait ChartStyleExt: Widget + Sized {
    /// Draws the axes of every chart inside with `style`.
    fn axis_style(self, style: impl AxisStyle) -> impl Widget {
        EnvUpdatingNew::<Self, AxisStyleKey>::new(Box::new(style) as Box<dyn AxisStyle>, self)
    }

    /// Colours the datasets of every chart inside with `style`.
    fn series_style(self, style: impl SeriesStyle) -> impl Widget {
        EnvUpdatingNew::<Self, SeriesStyleKey>::new(Box::new(style) as Box<dyn SeriesStyle>, self)
    }

    /// Draws the tooltips of every chart inside with `style`.
    fn tooltip_style(self, style: impl TooltipStyle) -> impl Widget {
        EnvUpdatingNew::<Self, TooltipStyleKey>::new(Box::new(style) as Box<dyn TooltipStyle>, self)
    }
}

impl<W: Widget> ChartStyleExt for W {}

#[derive(Debug, Copy, Clone)]
pub(crate) struct AxisStyleKey;

impl EnvironmentKey for AxisStyleKey {
    type Value = Box<dyn AxisStyle>;
    type Access = Shared;
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct SeriesStyleKey;

impl EnvironmentKey for SeriesStyleKey {
    type Value = Box<dyn SeriesStyle>;
    type Access = Shared;
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct TooltipStyleKey;

impl EnvironmentKey for TooltipStyleKey {
    type Value = Box<dyn TooltipStyle>;
    type Access = Shared;
}

/// The tooltip style set on the chart or one of its ancestors, or `AutomaticStyle`.
pub(crate) fn tooltip_style(env: &Environment) -> Box<dyn TooltipStyle> {
    env.get::<TooltipStyleKey>().cloned().unwrap_or_else(|| Box::new(AutomaticStyle))
}

/// The axis style set on the chart or one of its ancestors, or `AutomaticStyle`.
pub(crate) fn axis_style(env: &Environment) -> Box<dyn AxisStyle> {
    env.get::<AxisStyleKey>().cloned().unwrap_or_else(|| Box::new(AutomaticStyle))
}

/// The series style set on the chart or one of its ancestors, or `AutomaticStyle`.
pub(crate) fn series_style(env: &Environment) -> Box<dyn SeriesStyle> {
    env.get::<SeriesStyleKey>().cloned().unwrap_or_else(|| Box::new(AutomaticStyle))
}

/// The theme's value for `color`.
fn theme_color(env: &Environment, color: EnvironmentColor) -> Color {
    env.value(&color).unwrap_or_default()
}
