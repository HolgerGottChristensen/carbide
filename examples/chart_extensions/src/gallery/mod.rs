//! The gallery's frame: the bar of charts on top, the chart shown with its side bar, the bar
//! reporting the pointer at the bottom, and the options and helpers every chart shares. Each chart
//! is in a module of its own.

mod bars;
mod custom;
mod lines;
mod marks;
mod mixed;
mod pie;
mod polar;
mod scales;
mod scatter;

use carbide::automatic_style::AutomaticStyle;
use carbide::chart::{Axis, AxisPosition, CategoryScale, Chart, ChartStyleExt, Interpolation, Sequence, Stepped, LabelRotation, LegendPosition, LinearScale, LogScale, Measure, Pick, PickMode, Scale, SeriesStyle, SnapMode, TimeScale, TooltipAnchor, ZoomWindow, DatasetController};
use carbide::color::Color;
use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::picker::{MenuStyle, Picker};
use carbide::controls::slider::{Slider, SliderStepping};
use carbide::controls::toggle::{CheckboxStyle, Toggle};
use carbide::controls::{ControlsExt, TextInput};
use carbide::draw::color::rgba_bytes;
use carbide::draw::{Alignment, Angle, Scalar};
use carbide::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide::layout::LayoutDirection;
use carbide::state::{IntoReadState, LocalState, Map1, Map3, ReadState, State};
use carbide::widget::*;
use carbide::ui;

/// The charts the gallery shows, one at a time.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Example {
    Bars,
    Lines,
    Scatter,
    Pie,
    Polar,
    Scales,
    Marks,
    Mixed,
    Custom,
}

pub(crate) const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// The gallery: the bar of charts on top, the chart `selected` shows with its side bar, and the
/// bar reporting what the pointer hovers and clicks at the bottom, all in the colours the shared
/// style picker chooses.
pub fn gallery(selected: LocalState<Example>) -> impl Widget {
    let shared = Shared::new();

    let (bars, lines, scatter, pie, polar) = (bars::Options::new(), lines::Options::new(), scatter::Options::new(), pie::Options::new(), polar::Options::new());
    let (scales, marks, mixed, custom) = (scales::Options::new(), marks::Options::new(), mixed::Options::new(), custom::Options::new());
    let s = shared.clone();

    let gallery = ui!(
        match selected {
            Example::Bars => bars::view(bars.clone(), s.clone()),
            Example::Lines => lines::view(lines.clone(), s.clone()),
            Example::Scatter => scatter::view(scatter.clone(), s.clone()),
            Example::Pie => pie::view(pie.clone(), s.clone()),
            Example::Polar => polar::view(polar.clone(), s.clone()),
            Example::Scales => scales::view(scales.clone(), s.clone()),
            Example::Marks => marks::view(marks.clone(), s.clone()),
            Example::Mixed => mixed::view(mixed.clone(), s.clone()),
            Example::Custom => custom::view(custom.clone(), s.clone()),
        }
    );

    let tool_bar = HStack::new((
        example_button("Bars", Example::Bars, &selected, &shared),
        example_button("Lines", Example::Lines, &selected, &shared),
        example_button("Scatter", Example::Scatter, &selected, &shared),
        example_button("Pie", Example::Pie, &selected, &shared),
        example_button("Polar", Example::Polar, &selected, &shared),
        example_button("Scales", Example::Scales, &selected, &shared),
        example_button("Marks", Example::Marks, &selected, &shared),
        example_button("Mixed", Example::Mixed, &selected, &shared),
        example_button("Custom", Example::Custom, &selected, &shared),
        Spacer::new(),
    ))
        .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0))
        .frame_fixed_height(35.0)
        .background(Rectangle::new().fill(EnvironmentColor::SystemFill));

    let status_bar = HStack::new((
        Text::new(Map1::read_map(shared.hovered.clone(), |pick: &Option<Pick>| match pick {
            Some(pick) => format!("Hovering: {}", describe(pick)),
            None => "Hover the chart".to_string(),
        })),
        Spacer::new(),
        Text::new(shared.clicked.clone()),
    ))
        .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0))
        .frame_fixed_height(24.0)
        .background(Rectangle::new().fill(EnvironmentColor::SystemFill));

    VStack::new((tool_bar, gallery, status_bar))
        .spacing(0.0)
        .series_style(GalleryStyle { colors: shared.colors.clone() })
        .button_style(BorderedProminentStyle)
        .layout_direction(LayoutDirection::LeftToRight)
}

/// A button in the top bar that shows `example`, letting go of what the chart before it hovered.
fn example_button(label: &str, example: Example, selected: &LocalState<Example>, shared: &Shared) -> impl Widget + use<> {
    let (selected, hovered) = (selected.clone(), shared.hovered.clone());
    Button::new(label.to_string(), move |_| {
        let (mut selected, mut hovered) = (selected.clone(), hovered.clone());
        *selected.value_mut() = example;
        *hovered.value_mut() = None;
    }).frame(80.0, 26.0)
}

// ===============================
//  Options shared by every chart
// ===============================

/// The colours the datasets take, chosen in the side bar for every chart.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Colors {
    Theme,
    Pastel,
    Vivid,
}

/// The gallery's series style: the theme's palette, or one of two of the gallery's own, as the
/// shared picker chooses. The chart reads its style as it draws, so a change shows at once.
#[derive(Clone, Debug)]
pub(crate) struct GalleryStyle {
    colors: LocalState<Colors>,
}

impl SeriesStyle for GalleryStyle {
    fn palette(&self, env: &Environment) -> Vec<Color> {
        match *self.colors.value() {
            Colors::Theme => AutomaticStyle.palette(env),
            Colors::Pastel => vec![
                rgba_bytes(141, 180, 226, 1.0), rgba_bytes(244, 178, 128, 1.0), rgba_bytes(156, 214, 159, 1.0),
                rgba_bytes(236, 150, 160, 1.0), rgba_bytes(196, 170, 222, 1.0), rgba_bytes(240, 220, 130, 1.0),
            ],
            Colors::Vivid => vec![
                rgba_bytes(230, 25, 75, 1.0), rgba_bytes(60, 180, 75, 1.0), rgba_bytes(0, 130, 200, 1.0),
                rgba_bytes(245, 130, 48, 1.0), rgba_bytes(145, 30, 180, 1.0), rgba_bytes(70, 240, 240, 1.0),
            ],
        }
    }

    fn fill_opacity(&self) -> f32 {
        match *self.colors.value() {
            Colors::Pastel => 0.5,
            _ => AutomaticStyle.fill_opacity(),
        }
    }
}

/// What every chart reports and is set by: the hovered and clicked data, which the bottom bar
/// shows, and the options for the pointer, the legend and the colours.
#[derive(Clone)]
pub(crate) struct Shared {
    hovered: LocalState<Option<Pick>>,
    clicked: LocalState<String>,
    interactive: LocalState<bool>,
    hover_mode: LocalState<PickMode>,
    legend: LocalState<LegendPosition>,
    legend_toggles: LocalState<bool>,
    tooltip: LocalState<bool>,
    tooltip_anchor: LocalState<TooltipAnchor>,
    tooltip_alignment: LocalState<Alignment>,
    colors: LocalState<Colors>,
    horizontal_axis: AxisControls,
    vertical_axis: AxisControls,
}

impl Shared {
    fn new() -> Shared {
        Shared {
            hovered: LocalState::new(None),
            clicked: LocalState::new("Click the chart".to_string()),
            interactive: LocalState::new(true),
            hover_mode: LocalState::new(PickMode::Index),
            legend: LocalState::new(LegendPosition::Bottom),
            legend_toggles: LocalState::new(true),
            tooltip: LocalState::new(true),
            tooltip_anchor: LocalState::new(TooltipAnchor::Data),
            tooltip_alignment: LocalState::new(Alignment::Top),
            colors: LocalState::new(Colors::Theme),
            horizontal_axis: AxisControls::new(Axis::Horizontal),
            vertical_axis: AxisControls::new(Axis::Vertical),
        }
    }

    /// The side and tick label options of the scales along `axis`, shared by every chart.
    pub(crate) fn axis_controls(&self, axis: Axis) -> &AxisControls {
        match axis {
            Axis::Vertical => &self.vertical_axis,
            _ => &self.horizontal_axis,
        }
    }

    /// The side across from the one the main scales along `axis` are on, for a second scale
    /// beside them, as a state that follows the side bar.
    pub(crate) fn other_side(&self, axis: Axis) -> impl ReadState<T=AxisPosition> + Clone + use<> {
        Map1::read_map(self.axis_controls(axis).side.clone(), |side: &AxisPosition| match side {
            AxisPosition::Left => AxisPosition::Right,
            AxisPosition::Right => AxisPosition::Left,
            AxisPosition::Leading => AxisPosition::Trailing,
            AxisPosition::Trailing => AxisPosition::Leading,
            AxisPosition::Top => AxisPosition::Bottom,
            AxisPosition::Bottom => AxisPosition::Top,
        })
    }

    /// The side bar's sections for the horizontal and the vertical scales, in the charts with
    /// scales along the sides: their side and tick labels. Polar charts place their labels around
    /// the circle, and leave these out.
    pub(crate) fn axis_options(&self) -> impl Widget + use<> {
        VStack::new((
            self.horizontal_axis.options("x axis", Axis::Horizontal),
            self.vertical_axis.options("y axis", Axis::Vertical),
        )).cross_axis_alignment(CrossAxisAlignment::Start)
    }

    /// `chart` with the shared options: the legend, the pointer's options, and its hover and click
    /// reported to the bottom bar.
    pub(crate) fn apply<C: Sequence<dyn DatasetController>>(&self, chart: Chart<C>) -> Chart<C> {
        let clicked = self.clicked.clone();
        chart
            .legend(self.legend.clone())
            .legend_toggles(self.legend_toggles.clone())
            .tooltip(self.tooltip.clone())
            .tooltip_anchor(self.tooltip_anchor.clone())
            .tooltip_alignment(self.tooltip_alignment.clone())
            .interactive(self.interactive.clone())
            .hover(self.hovered.clone())
            .hover_mode(self.hover_mode.clone())
            .on_click(move |click| {
                let mut clicked = clicked.clone();
                *clicked.value_mut() = format!("Clicked: {}", describe(click.pick));
            })
    }

    /// The side bar's section for the shared options.
    fn options(&self) -> impl Widget + use<> {
        VStack::new((
            section("Pointer"),
            Toggle::new("Interactive", self.interactive.clone()),
            Toggle::new("Tooltips", self.tooltip.clone()),
            Picker::new("Tooltip at", self.tooltip_anchor.clone(), (
                Text::new("The data").tag(TooltipAnchor::Data),
                Text::new("The pointer").tag(TooltipAnchor::Pointer),
            )),
            Picker::new("Tooltip side", self.tooltip_alignment.clone(), (
                Text::new("Above").tag(Alignment::Top),
                Text::new("Below").tag(Alignment::Bottom),
                Text::new("Before").tag(Alignment::Leading),
                Text::new("After").tag(Alignment::Trailing),
                Text::new("Above, before").tag(Alignment::TopLeading),
                Text::new("Above, after").tag(Alignment::TopTrailing),
                Text::new("Below, before").tag(Alignment::BottomLeading),
                Text::new("Below, after").tag(Alignment::BottomTrailing),
                Text::new("Centred").tag(Alignment::Center),
            )),
            Picker::new("Hover finds", self.hover_mode.clone(), (
                Text::new("The category").tag(PickMode::Index),
                Text::new("What is under it").tag(PickMode::Point),
                Text::new("The nearest").tag(PickMode::Nearest(Measure::Both)),
                Text::new("The nearest along x").tag(PickMode::Nearest(Measure::X)),
                Text::new("The nearest along y").tag(PickMode::Nearest(Measure::Y)),
                Text::new("The nearest around a circle").tag(PickMode::Nearest(Measure::Angle)),
                Text::new("The nearest out from its centre").tag(PickMode::Nearest(Measure::Distance)),
                Text::new("The whole dataset").tag(PickMode::Dataset),
            )),
            section("Look"),
            Picker::new("Legend", self.legend.clone(), (
                Text::new("Hidden").tag(LegendPosition::Hidden),
                Text::new("Top").tag(LegendPosition::Top),
                Text::new("Bottom").tag(LegendPosition::Bottom),
                Text::new("Left").tag(LegendPosition::Left),
                Text::new("Right").tag(LegendPosition::Right),
            )),
            // A click on a legend entry hides or shows its dataset, a double-click shows only it.
            Toggle::new("Legend toggles datasets", self.legend_toggles.clone()),
            Picker::new("Colours", self.colors.clone(), (
                Text::new("The theme's").tag(Colors::Theme),
                Text::new("Pastel").tag(Colors::Pastel),
                Text::new("Vivid").tag(Colors::Vivid),
            )),
        )).cross_axis_alignment(CrossAxisAlignment::Start)
    }
}

// ===============================
//  Tick labels
// ===============================

/// The kind of curve through a line's points, picked in the side bar: smooth and cardinal curves
/// take the tension slider's tension, Catmull-Rom curves the alpha slider's alpha, and with none
/// only the points are drawn. Each page offers the curves that suit its charts.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Curve {
    None,
    Straight,
    Smooth,
    Cardinal,
    CatmullRom,
    Monotone,
    StepsBefore,
    StepsMiddle,
    StepsMiddleVertical,
    StepsAfter,
}

/// A curve picked in the side bar, with the tension and the alpha the curves that have one take.
#[derive(Clone)]
pub(crate) struct CurveControls {
    pub(crate) curve: LocalState<Curve>,
    tension: LocalState<Scalar>,
    alpha: LocalState<Scalar>,
}

impl CurveControls {
    /// `curve` to start with, a tension of 0.4 and the centripetal alpha of 0.5.
    pub(crate) fn new(curve: Curve) -> CurveControls {
        CurveControls {
            curve: LocalState::new(curve),
            tension: LocalState::new(0.4),
            alpha: LocalState::new(0.5),
        }
    }

    /// The interpolation of the picked curve, as a state that follows the side bar. No curve is
    /// drawn straight, for the points the line would join.
    pub(crate) fn interpolation(&self) -> impl ReadState<T=Interpolation> + Clone + use<> {
        Map3::read_map(self.curve.clone(), self.tension.clone(), self.alpha.clone(), |curve: &Curve, tension: &Scalar, alpha: &Scalar| match curve {
            Curve::None | Curve::Straight => Interpolation::Linear,
            Curve::Smooth => Interpolation::Smooth { tension: *tension },
            Curve::Cardinal => Interpolation::Cardinal { tension: *tension },
            Curve::CatmullRom => Interpolation::CatmullRom { alpha: *alpha },
            Curve::Monotone => Interpolation::Monotone,
            Curve::StepsBefore => Interpolation::Stepped(Stepped::Before),
            Curve::StepsMiddle => Interpolation::Stepped(Stepped::Middle),
            Curve::StepsMiddleVertical => Interpolation::Stepped(Stepped::MiddleVertical),
            Curve::StepsAfter => Interpolation::Stepped(Stepped::After),
        })
    }

    /// Whether a line is drawn: with no curve, only the points are.
    pub(crate) fn shows_line(&self) -> impl ReadState<T=bool> + Clone + use<> {
        Map1::read_map(self.curve.clone(), |curve: &Curve| *curve != Curve::None)
    }

    /// The sliders of the picked curve: the tension for smooth and cardinal curves, the alpha for
    /// Catmull-Rom curves, and none for the others.
    pub(crate) fn sliders(&self) -> impl Widget + use<> {
        VStack::new((
            shown_when(
                Map1::read_map(self.curve.clone(), |curve: &Curve| matches!(curve, Curve::Smooth | Curve::Cardinal)),
                slider("Tension", &self.tension, 0.0, 1.0, 0.05, |tension| format!("{tension:.2}")),
            ),
            shown_when(
                Map1::read_map(self.curve.clone(), |curve: &Curve| *curve == Curve::CatmullRom),
                slider("Alpha", &self.alpha, 0.0, 1.0, 0.05, |alpha| format!("{alpha:.2}")),
            ),
        )).cross_axis_alignment(CrossAxisAlignment::Start)
    }
}

/// How the side bar turns tick labels: only when they don't fit, or always by an angle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Turning {
    Auto,
    Fixed,
}

/// The options of the scales along one direction, set in the side bar: the side of the chart they
/// are on, and how their tick labels turn, by how much, where they hang off their ticks, and
/// whether those too close together are left out.
#[derive(Clone)]
pub(crate) struct AxisControls {
    side: LocalState<AxisPosition>,
    turning: LocalState<Turning>,
    /// The angle of a fixed rotation, in degrees.
    angle: LocalState<Scalar>,
    /// The angle automatic rotation turns to, in degrees.
    max_angle: LocalState<Scalar>,
    alignment: LocalState<Option<Alignment>>,
    skip: LocalState<bool>,
}

impl AxisControls {
    /// The options for the scales along `axis`, on its default side.
    fn new(axis: Axis) -> AxisControls {
        AxisControls {
            side: LocalState::new(axis.default_position()),
            turning: LocalState::new(Turning::Auto),
            angle: LocalState::new(-45.0),
            max_angle: LocalState::new(-45.0),
            alignment: LocalState::new(None),
            skip: LocalState::new(true),
        }
    }

    /// The rotation the options choose, as a state of a `LabelRotation`.
    fn rotation(&self) -> impl ReadState<T=LabelRotation> + Clone + use<> {
        Map3::read_map(self.turning.clone(), self.angle.clone(), self.max_angle.clone(), |turning: &Turning, angle: &Scalar, max: &Scalar| match turning {
            Turning::Auto => LabelRotation::Auto { max: Angle::Degrees(*max) },
            Turning::Fixed => LabelRotation::Fixed(Angle::Degrees(*angle)),
        })
    }

    /// The side bar's section for these options, headed `title`, for the scales along `axis`:
    /// the sides the axis can take, the angle slider of the chosen way of turning, and only
    /// where labels turn automatically along a horizontal axis does the automatic angle apply.
    fn options(&self, title: &str, axis: Axis) -> impl Widget + use<> {
        let horizontal = axis == Axis::Horizontal;
        // A vertical axis takes its side by name or by reading direction.
        let side = IfElse::new(horizontal)
            .when_true(Picker::new("Side", self.side.clone(), (
                Text::new("Bottom").tag(AxisPosition::Bottom),
                Text::new("Top").tag(AxisPosition::Top),
            )))
            .when_false(Picker::new("Side", self.side.clone(), (
                Text::new("Left").tag(AxisPosition::Left),
                Text::new("Right").tag(AxisPosition::Right),
                Text::new("Leading").tag(AxisPosition::Leading),
                Text::new("Trailing").tag(AxisPosition::Trailing),
            )));
        let fixed = Map1::read_map(self.turning.clone(), |turning: &Turning| *turning == Turning::Fixed);
        let auto_along = Map1::read_map(self.turning.clone(), move |turning: &Turning| *turning == Turning::Auto && horizontal);
        let auto_beside = Map1::read_map(self.turning.clone(), move |turning: &Turning| *turning == Turning::Auto && !horizontal);

        VStack::new((
            section(title),
            side,
            Picker::new("Turn labels", self.turning.clone(), (
                Text::new("When they don't fit").tag(Turning::Auto),
                Text::new("Always").tag(Turning::Fixed),
            )),
            shown_when(fixed, slider("Angle", &self.angle, -90.0, 90.0, 5.0, degrees)),
            shown_when(auto_along, slider("Up to", &self.max_angle, -90.0, 90.0, 5.0, degrees)),
            shown_when(auto_beside, note("Labels beside a vertical axis stay flat: turning them makes no room between them.")),
            Picker::new("Hang from", self.alignment.clone(), (
                Text::new("Pointing at the tick").tag(None),
                Text::new("Top").tag(Some(Alignment::Top)),
                Text::new("Bottom").tag(Some(Alignment::Bottom)),
                Text::new("Start").tag(Some(Alignment::Leading)),
                Text::new("End").tag(Some(Alignment::Trailing)),
                Text::new("Top, start").tag(Some(Alignment::TopLeading)),
                Text::new("Top, end").tag(Some(Alignment::TopTrailing)),
                Text::new("Bottom, start").tag(Some(Alignment::BottomLeading)),
                Text::new("Bottom, end").tag(Some(Alignment::BottomTrailing)),
                Text::new("Centre").tag(Some(Alignment::Center)),
            )),
            Toggle::new("Leave out labels that don't fit", self.skip.clone()),
        )).cross_axis_alignment(CrossAxisAlignment::Start)
    }
}

/// A scale that follows the side bar's options for the direction it runs in.
pub(crate) trait Labelled: Scale + Sized {
    /// The scale with `rotation`, `alignment` and `skip` set.
    fn with_labels(self, rotation: impl IntoReadState<LabelRotation>, alignment: LocalState<Option<Alignment>>, skip: LocalState<bool>) -> Self;

    /// The scale on `side`.
    fn with_side(self, side: impl IntoReadState<AxisPosition>) -> Self;

    /// The scale with the tick label options `shared` has for its axis, on a side of its own: a
    /// second scale beside the main one.
    fn labelled(self, shared: &Shared) -> Self {
        let options = shared.axis_controls(self.axis()).clone();
        self.with_labels(options.rotation(), options.alignment, options.skip)
    }

    /// The scale on the side `shared` has for its axis, with its tick label options: the main
    /// scale along that axis.
    fn placed(self, shared: &Shared) -> Self {
        let side = shared.axis_controls(self.axis()).side.clone();
        self.with_side(side).labelled(shared)
    }
}

macro_rules! labelled {
    ($($scale:ty),*) => {
        $(
            impl Labelled for $scale {
                fn with_labels(self, rotation: impl IntoReadState<LabelRotation>, alignment: LocalState<Option<Alignment>>, skip: LocalState<bool>) -> Self {
                    self.label_rotation(rotation).label_alignment(alignment).label_skip(skip)
                }

                fn with_side(self, side: impl IntoReadState<AxisPosition>) -> Self {
                    self.position(side)
                }
            }
        )*
    };
}

labelled!(LinearScale, CategoryScale, TimeScale, LogScale, chart_extensions::LogScale);

// ===============================
//  Helpers for the side bars
// ===============================

/// A chart beside its side bar, the shared options at the bottom of it.
pub(crate) fn with_side_bar(chart: impl Widget, options: impl Widget, shared: &Shared) -> Box<dyn AnyWidget> {
    HSplit::new(
        chart.padding(10.0),
        ZStack::new((
            Rectangle::new().fill(EnvironmentColor::TertiarySystemFill),
            VStack::new((
                options,
                shared.options(),
                Spacer::new(),
            ))
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .padding(10.0)
                .toggle_style(CheckboxStyle)
                .picker_style(MenuStyle)
                .scroll()
                .with_scroll_direction(ScrollDirection::Vertical)
                .clip(),
        )),
    ).relative_to_end(300.0).boxed()
}

/// A chart with scales along its sides beside its side bar: the chart's own options, then the
/// options of its x and y axes, then the shared ones.
pub(crate) fn with_labelled_side_bar(chart: impl Widget, options: impl Widget, shared: &Shared) -> Box<dyn AnyWidget> {
    let options = VStack::new((options, shared.axis_options())).cross_axis_alignment(CrossAxisAlignment::Start);
    with_side_bar(chart, options, shared)
}

/// A heading in the side bar, set apart from the section above it.
pub(crate) fn section(title: &str) -> impl Widget + use<> {
    first_section(title).padding(EdgeInsets::single(12.0, 0.0, 0.0, 0.0))
}

/// The heading at the top of a side bar, with nothing above it to be set apart from.
pub(crate) fn first_section(title: &str) -> impl Widget + use<> {
    Text::new(title.to_string()).font_size(EnvironmentFontSize::Title3)
}

/// A line of explanation in the side bar.
pub(crate) fn note(text: &str) -> impl Widget + use<> {
    Text::new(text.to_string()).frame_fixed_width(270.0).fit_height()
}

/// A text option, edited in the side bar.
pub(crate) fn text_input(label: &str, text: &LocalState<String>) -> impl Widget + use<> {
    HStack::new((Text::new(label.to_string()), TextInput::new(text.clone())))
}

/// A zoom window's controls: what it shows, and a button that shows everything again.
pub(crate) fn zoom_controls(label: &str, zoom: &LocalState<ZoomWindow>) -> impl Widget + use<> {
    let reset = zoom.clone();
    let label = label.to_string();
    VStack::new((
        Text::new(Map1::read_map(zoom.clone(), move |window: &ZoomWindow| format!("{} shows {:.0}% to {:.0}%", label, window.start * 100.0, window.end * 100.0))),
        Button::new("Show everything", move |_| {
            let mut reset = reset.clone();
            *reset.value_mut() = ZoomWindow::WHOLE;
        }).frame(140.0, 26.0),
    )).cross_axis_alignment(CrossAxisAlignment::Start)
}

/// How a category scale's zoom lines up with its categories: snapped to whole ones, or
/// scrolling freely.
pub(crate) fn snap_picker(snap_mode: &LocalState<SnapMode>) -> impl Widget + use<> {
    Picker::new("Snap", snap_mode.clone(), (
        Text::new("To whole categories").tag(SnapMode::Categories),
        Text::new("Scroll freely").tag(SnapMode::Free),
    ))
}

/// A slider for a number, with its value beside it. `step` is a step size to snap to, or
/// `SliderStepping::Smooth` for any value in between.
pub(crate) fn slider<S: IntoReadState<SliderStepping<Scalar>>>(label: &str, value: &LocalState<Scalar>, min: Scalar, max: Scalar, step: S, format: fn(&Scalar) -> String) -> impl Widget + use<S> {
    HStack::new((
        Text::new(label.to_string()),
        Slider::new(value.clone(), min, max).step(step),
        Text::new(Map1::read_map(value.clone(), format)).frame_fixed_width(50.0),
    )).frame_fixed_height(26.0)
}

/// `widget`, shown only while `condition` holds: an option that can't apply while other options
/// are as they are is left out of the side bar, taking no room.
pub(crate) fn shown_when(condition: impl ReadState<T=bool> + Clone + 'static, widget: impl Widget) -> impl Widget {
    IfElse::new(condition).when_true(widget)
}

/// A share as a percentage, for sliders.
pub(crate) fn percent(share: &Scalar) -> String {
    format!("{:.0}%", share * 100.0)
}

/// Degrees, for sliders.
pub(crate) fn degrees(degrees: &Scalar) -> String {
    format!("{degrees:.0}°")
}

/// Pixels, for sliders.
pub(crate) fn pixels(pixels: &Scalar) -> String {
    format!("{pixels:.0} px")
}

/// What a pick found, for the bottom bar: each datum by its dataset and index, and the value under
/// the pointer on each scale, or its category.
fn describe(pick: &Pick) -> String {
    // The data found, as their controllers describe them, or by their keys when they can't.
    let data = if pick.data.is_empty() {
        "nothing".to_string()
    } else if !pick.items.is_empty() {
        pick.items.iter().map(|item| item.row_text()).collect::<Vec<_>>().join(", ")
    } else {
        pick.data.iter().map(|datum| format!("dataset {} #{}", datum.dataset, datum.index)).collect::<Vec<_>>().join(", ")
    };

    let values = pick.values.iter()
        .map(|(id, value)| match (&value.category, &value.label) {
            (Some(category), _) => format!("{} {}", id.name(), category.name.clone().unwrap_or_else(|| format!("{}", category.index))),
            (None, Some(label)) => format!("{} {}", id.name(), label),
            (None, None) => format!("{} {:.1}", id.name(), value.value),
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("{data} at {values}")
}


