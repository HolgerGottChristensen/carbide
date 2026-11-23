use std::collections::HashSet;
use carbide::environment::{Environment, EnvironmentFontSize};
use carbide::lifecycle::{Update, UpdateContext};
use carbide::widget::EdgeInsets;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Alignment, Dimension, Position, Rect, Scalar};
use carbide::state::{AnyReadState, AnyState, IntoReadState, IntoState, ReadStateExt, StateExtNew};
use carbide::event::{ModifierKey, MouseButton, MouseEvent, MouseEventContext, MouseEventHandler};
use carbide::draw::color::{rgba, Rgba};
use indexmap::{IndexMap, IndexSet};
use carbide_core::render::{Render, RenderContext};
use carbide_core::widget::{CommonWidget, WidgetId, WidgetSync};
use carbide_core::text::{FontSize, FontWeight};
use carbide_core::widget::canvas::CanvasContext;
use carbide_derive::Widget;
use crate::controller::{ControllerSequence, DatasetController};
use crate::legend::{self, LegendPosition};
use crate::sequence::IntoSequence;
use crate::style::axis_style;
use crate::scale::{axis, Axis, AxisPosition, PolarRole, Scale, ScaleBounds, ScaleId, ScaleKind, Scales};
use crate::scale::polar::RimLabel;
use crate::geometry::{DatumKey, PickMode, Shapes};
use crate::interaction::{pan, share_at, zoom_about, ChartClick, ClickAction, Gesture, Pick, WHEEL_ZOOM_PER_LINE};

/// The gap between the bands of two scales stacked on the same side of the chart area.
const SCALE_SPACING: Scalar = 8.0;

/// The gap between the chart title or the legend and what is inside it.
const DECORATION_SPACING: Scalar = 12.0;

/// A container that owns the scales and draws a sequence of controllers on them. Each dataset
/// is drawn on the scales it is bound to (`ScaleId::X` and `ScaleId::Y` by default), so datasets
/// in different controllers share a scale when they are bound to the same id.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Update, Sync, MouseEvent)]
pub struct Chart<C> where C: ControllerSequence {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    controllers: C,
    scales: Scales,
    padding: Box<dyn AnyReadState<T=EdgeInsets>>,
    title: Option<Box<dyn AnyReadState<T=String>>>,
    legend: Box<dyn AnyReadState<T=LegendPosition>>,
    /// Unknown scale ids already warned about, so each is logged once, not every update.
    warned_scale_ids: HashSet<ScaleId>,
    /// The default scales not replaced with `Chart::scale`, and the kind each is now. Each fit
    /// rebuilds one when the controllers ask for another kind.
    default_kinds: IndexMap<ScaleId, ScaleKind>,
    /// The shapes the controllers drew on the last render, with the chart area, which the data
    /// under a position is found by.
    shapes: Shapes,
    /// Whether the chart responds to the pointer at all (see `interactive`).
    interactive: Box<dyn AnyReadState<T=bool>>,
    /// The state the data under the pointer is written to, when the chart reports hovering.
    hover: Option<Box<dyn AnyState<T=Option<Pick>>>>,
    /// How hovering and clicks find the data under the pointer.
    hover_mode: Box<dyn AnyReadState<T=PickMode>>,
    /// What a click in the chart area does.
    on_click: Option<ClickAction>,
    /// The drag in progress: a pan, or a box to zoom to.
    gesture: Option<Gesture>,
    /// The last press of the primary button, kept until the next, so a drag only starts from a
    /// press inside the chart, and a press that moved isn't a click.
    press: Option<Press>,
    /// The band of each scale along a side on the last render, in the chart's own coordinates,
    /// where gestures move only that scale.
    bands: Vec<(ScaleId, Rect)>,
}

impl<C: ControllerSequence> Chart<C> {
    /// A chart drawing `controllers` on the default scales, `ScaleId::X` and `ScaleId::Y`. Each
    /// default is a linear scale unless a controller asks for another kind (see
    /// `ScaleBounds::request_kind`); the first to ask wins. A bar or line chart's x scale is a
    /// `CategoryScale` this way. The kind is decided on every fit, so it follows controllers
    /// added or removed after the chart is built.
    ///
    /// `controllers` is a sequence of controllers, or a single controller.
    pub fn new<I: IntoSequence<dyn DatasetController, M, Sequence=C>, M>(controllers: I) -> Chart<C> {
        Chart {
            id: Default::default(),
            position: Default::default(),
            dimension: Default::default(),
            controllers: controllers.into_sequence(),
            scales: Scales::new(),
            padding: IntoReadState::<EdgeInsets>::into_read_state(EdgeInsets::all(30.0)).as_dyn_read(),
            title: None,
            legend: IntoReadState::<LegendPosition>::into_read_state(LegendPosition::Hidden).as_dyn_read(),
            warned_scale_ids: HashSet::new(),
            default_kinds: [(ScaleId::X, ScaleKind::Linear), (ScaleId::Y, ScaleKind::Linear)].into_iter().collect(),
            shapes: Shapes::default(),
            interactive: IntoReadState::<bool>::into_read_state(true).as_dyn_read(),
            hover: None,
            hover_mode: IntoReadState::<PickMode>::into_read_state(PickMode::Index).as_dyn_read(),
            on_click: None,
            gesture: None,
            press: None,
            bands: vec![],
        }
    }

    /// Adds `scale` to the chart under `id`, for datasets bound to it with `.x_scale(id)` or
    /// `.y_scale(id)`. Passing `ScaleId::X` or `ScaleId::Y` replaces that default scale.
    ///
    /// Scales on the same side of the chart area are stacked outwards, each in its own band:
    /// by `Scale::weight`, then in the order they were added, the first closest to the area.
    ///
    /// `id` is a `ScaleId`, or a `String` for a scale named at runtime.
    /// A default scale replaced this way is never rebuilt, whatever kind controllers ask for;
    /// it still gets their offset and grid offset requests.
    pub fn scale(mut self, id: impl Into<ScaleId>, scale: impl Scale) -> Chart<C> {
        let id = id.into();
        self.default_kinds.shift_remove(&id);
        self.scales.insert(id, scale);
        self
    }

    /// Gives the chart a title, a text or a state of one, drawn centred above everything else.
    pub fn title(mut self, title: impl IntoReadState<String>) -> Chart<C> {
        self.title = Some(title.into_read_state().as_dyn_read());
        self
    }

    /// The space between the chart's edges and what it draws, an `EdgeInsets` or a number for
    /// every side, or a state of one. Defaults to 30 on every side.
    pub fn padding(mut self, padding: impl IntoReadState<EdgeInsets>) -> Chart<C> {
        self.padding = padding.into_read_state().as_dyn_read();
        self
    }

    /// Shows a legend on the given side of the chart, a `LegendPosition` or a state of one:
    /// each dataset with a name (`DatasetExt::label`), next to a marker in its colour. Defaults to
    /// `LegendPosition::Hidden`.
    pub fn legend(mut self, position: impl IntoReadState<LegendPosition>) -> Chart<C> {
        self.legend = position.into_read_state().as_dyn_read();
        self
    }

    /// Whether the chart responds to the pointer, a value or a state of one: `true` by default.
    /// While it is `false`, the chart ignores the pointer, so nothing is hovered, outlined or
    /// clicked and no gesture zooms or pans it; a hover state is set to `None`, and a drag or box
    /// in progress is dropped. Zoom windows keep what they show, and can still be set from code.
    pub fn interactive(mut self, interactive: impl IntoReadState<bool>) -> Chart<C> {
        self.interactive = interactive.into_read_state().as_dyn_read();
        self
    }

    /// Whether the chart responds to the pointer, as last synced.
    fn is_interactive(&self) -> bool {
        *self.interactive.value_dyn()
    }

    /// Lets go of everything the pointer started, when the chart stops responding to it: the
    /// hovered data, and any drag or box in progress.
    fn release_pointer(&mut self) {
        self.gesture = None;
        self.press = None;
        if let Some(hover) = &mut self.hover
            && hover.value_dyn().is_some() {
                hover.set_value_dyn(None);
            }
    }

    /// Reports what is under the pointer to `state`, an `Option<Pick>` or a state of one: the
    /// data found there, as `hover_mode` finds them, and the value under the pointer on every
    /// scale, while the pointer is over the chart area, and `None` when it isn't. The chart
    /// outlines the data found.
    pub fn hover(mut self, state: impl IntoState<Option<Pick>>) -> Chart<C> {
        self.hover = Some(state.into_state().as_dyn());
        self
    }

    /// How hovering and clicks find the data under the pointer, a `PickMode` or a state of one.
    /// `PickMode::Index` by default: every dataset's datum at the category nearest the pointer, as
    /// a shared tooltip shows.
    pub fn hover_mode(mut self, mode: impl IntoReadState<PickMode>) -> Chart<C> {
        self.hover_mode = mode.into_read_state().as_dyn_read();
        self
    }

    /// Calls `action` when the chart area is clicked with the primary button, with what was found
    /// there, as for `hover`.
    pub fn on_click(mut self, action: impl Fn(ChartClick) + 'static) -> Chart<C> {
        self.on_click = Some(ClickAction(std::rc::Rc::new(action)));
        self
    }

    /// What is under `position`, in the chart's own coordinates: `None` outside the chart area.
    fn pick_at(&self, position: Position, env: &mut Environment) -> Option<Pick> {
        if !inside(self.shapes.area(), position) {
            return None;
        }

        let data = self.shapes.pick(position, *self.hover_mode.value_dyn());
        let ids: Vec<ScaleId> = self.scales.iter().map(|(id, _)| id.clone()).collect();
        let values = ids.into_iter()
            .filter_map(|id| self.scales.value_at(&id, position, self.shapes.area(), env).map(|value| (id, value)))
            .collect();

        Some(Pick { position, data, values })
    }

    /// Writes what is under `position` to the hover state, when the chart has one and it changed.
    fn hover_at(&mut self, position: Option<Position>, env: &mut Environment) {
        if self.hover.is_none() {
            return;
        }

        let pick = position.and_then(|position| self.pick_at(position, env));
        if let Some(hover) = &mut self.hover
            && *hover.value_dyn() != pick {
                hover.set_value_dyn(pick);
            }
    }

    /// The zoomable scales a gesture at `position` moves: every one over the chart area, or the one
    /// whose band it is over.
    fn gesture_scales(&self, position: Position) -> Vec<ScaleId> {
        let zoomable = |id: &ScaleId| self.scales.get(id).is_some_and(|scale| scale.axis_options().is_zoomable());

        if inside(self.shapes.area(), position) {
            return self.scales.iter().map(|(id, _)| id.clone()).filter(|id| zoomable(id)).collect();
        }

        self.bands.iter()
            .find(|(id, band)| zoomable(id) && inside(*band, position))
            .map(|(id, _)| vec![id.clone()])
            .unwrap_or_default()
    }

    /// Calls the click action with what is at `position`, when the chart has one and `position`
    /// is in the chart area.
    fn click(&self, position: Position, modifier_key: ModifierKey, env: &mut Environment) {
        let Some(action) = self.on_click.clone() else {
            return;
        };
        if let Some(pick) = self.pick_at(position, env) {
            (action.0)(ChartClick { pick: &pick, env, modifier_key });
        }
    }

    /// Changes the zoom of each of `scales` with `change`, from the window it asks for and its
    /// smallest span. Returns whether any of them changed.
    fn change_zoom(&mut self, scales: &[ScaleId], mut change: impl FnMut(&dyn Scale, crate::scale::ZoomWindow, f64) -> crate::scale::ZoomWindow) -> bool {
        let mut changed_any = false;
        for id in scales {
            let Some(scale) = self.scales.get_mut(id) else {
                continue;
            };

            let min_span = scale.axis_options().min_span();
            let window = scale.axis_options().zoom().limited(min_span);
            let changed = change(&*scale, window, min_span);
            if changed != window && let Some(options) = scale.axis_options_mut() {
                options.set_zoom(changed);
                changed_any = true;
            }
        }
        changed_any
    }

    /// Zooms `scales` by `factor` about `position`: a factor below 1 zooms in. Returns whether
    /// any of them changed.
    fn zoom_scales(&mut self, scales: &[ScaleId], position: Position, factor: f64) -> bool {
        let area = self.shapes.area();
        self.change_zoom(scales, |scale, window, min_span| zoom_about(window, share_at(scale, window, position, area), factor, min_span))
    }

    /// Pans `scales` so the data at `from` moves to `to`, as a drag from one to the other does,
    /// stopping at the data's ends. Returns whether any of them moved.
    fn pan_scales(&mut self, scales: &[ScaleId], from: Position, to: Position) -> bool {
        let area = self.shapes.area();
        self.change_zoom(scales, |scale, window, min_span| {
            pan(window, share_at(scale, window, from, area) - share_at(scale, window, to, area), min_span)
        })
    }


    /// The shapes the controllers drew on the last render.
    #[cfg_attr(not(test), allow(dead_code, reason = "read by the tests"))]
    pub(crate) fn shapes(&self) -> &Shapes {
        &self.shapes
    }

    /// The data at `position`, as `mode` finds them, among the shapes of the last render (see
    /// `Shapes::pick`). Positions are in the chart's own coordinates, from its top-left corner.
    #[cfg_attr(not(test), allow(dead_code, reason = "read by the tests"))]
    pub(crate) fn pick(&self, position: Position, mode: PickMode) -> Vec<DatumKey> {
        self.shapes.pick(position, mode)
    }

    /// Has every controller fill in `bounds`, after each scale's own categories, which come first
    /// so data naming them is placed on them.
    fn collect_bounds(&mut self, bounds: ScaleBounds, env: &mut Environment) -> ScaleBounds {
        for (id, scale) in self.scales.iter() {
            scale.categories(&mut |text| {
                bounds.register_category(id, text);
            });
        }

        let mut bounds = bounds;
        self.controllers.foreach(&mut |_, controller| {
            controller.collect_bounds(&mut bounds, env);
        });
        bounds
    }

    /// Collects what every controller asks of each scale id, rebuilds a default scale whose
    /// requested kind changed, then fits each scale to its requests. Data bound to an id the chart
    /// has no scale for is not drawn; it is logged once per id.
    fn fit_scales(&mut self, env: &mut Environment) {
        // A default scale across from a radial scale is radial too, before the controllers
        // collect, so they read polar datasets as polar. Its kind is kept until the requests say.
        let defaults: Vec<ScaleId> = self.default_kinds.keys().cloned().collect();
        let mut first_prefers_polar = None;
        self.controllers.foreach(&mut |_, controller| {
            first_prefers_polar.get_or_insert(controller.prefers_polar());
        });
        let first_prefers_polar = first_prefers_polar.unwrap_or(false);

        for id in &defaults {
            let axis = self.default_axis(id, first_prefers_polar);
            if let Some(kind) = self.default_kinds.get(id).copied()
                && self.scales.get(id).map(|scale| scale.axis()) != Some(axis) {
                self.scales.insert_boxed(id.clone(), kind.build(axis));
            }
        }

        let bounds = self.collect_bounds(ScaleBounds::with_radial(self.scales.radial_ids()), env);

        for id in &bounds.ids() {
            if !self.scales.contains(id) && !self.warned_scale_ids.contains(id) {
                self.warned_scale_ids.insert(id.clone());
                log::warn!("Chart has no scale with id {:?}; datasets bound to it are not drawn", id.name());
            }

            let request = bounds.get(id);
            if request.polar_role_conflict() {
                self.scales.warn_once("role conflict", id, id, || format!("The radial scale {:?} is the angle for some datasets and the radius for others; it is drawn as the {:?}, and datasets that need the other role are not drawn", id.name(), request.polar_role().unwrap_or(PolarRole::Angle)));
            }
        }

        for id in defaults {
            let requested = bounds.get(&id).default_kind();

            if Some(requested) != self.default_kinds.get(&id).copied()
                && let Some(axis) = self.scales.get(&id).map(|scale| scale.axis()) {
                self.scales.insert_boxed(id.clone(), requested.build(axis));
                self.default_kinds.insert(id, requested);
            }
        }

        self.scales.fit(&bounds);

        // Scales that fit to the visible data fit again, to the data inside the zoomed scales'
        // windows. The zoomed scales include every x either way, so they keep their range.
        let fitting_visible: IndexSet<ScaleId> = self.scales.iter()
            .filter(|(_, scale)| scale.axis() != Axis::Radial && scale.axis_options().fit_visible())
            .map(|(id, _)| id.clone())
            .collect();
        let windows: IndexMap<ScaleId, (Scalar, Scalar)> = self.scales.iter()
            .filter(|(_, scale)| !scale.axis_options().window().is_whole())
            .map(|(id, scale)| {
                let (start, end) = (scale.denormalize(0.0), scale.denormalize(1.0));
                (id.clone(), (start.min(end), start.max(end)))
            })
            .collect();
        if !fitting_visible.is_empty() && !windows.is_empty() {
            let visible = self.collect_bounds(ScaleBounds::with_windows(self.scales.radial_ids(), windows, fitting_visible), env);
            self.scales.fit(&visible);
        }

        // A scale gives up its decoration when a controller asks, unless its grid lines were set.
        let hidden = bounds.ids().into_iter()
            .filter(|id| bounds.get(id).hidden_decoration())
            .filter(|id| self.scales.get(id).is_some_and(|scale| !scale.axis_options().grid_lines_set()))
            .collect();
        self.scales.set_decoration_hidden(hidden);
    }

    /// The axis the default scale `id` is built on: across from the other default, as
    /// `ScaleId::X` is horizontal and `ScaleId::Y` vertical, unless the other was set to a radial
    /// scale with `Chart::scale`. Then it is radial too, so a radial x scale is all a polar chart
    /// needs. When both are defaults, they are radial if the first controller prefers it
    /// (`DatasetController::prefers_polar`), as a pie does.
    fn default_axis(&self, id: &ScaleId, first_prefers_polar: bool) -> Axis {
        let (own, other) = if *id == ScaleId::Y {
            (Axis::Vertical, ScaleId::X)
        } else {
            (Axis::Horizontal, ScaleId::Y)
        };

        let other_is_radial = if self.default_kinds.contains_key(&other) {
            first_prefers_polar
        } else {
            self.scales.get(&other).is_some_and(|scale| scale.axis() == Axis::Radial)
        };

        if other_is_radial { Axis::Radial } else { own }
    }

    /// Lays out everything around the chart area, from the outside in: the padding, the chart
    /// title along the top, the legend on its side, and the scales' bands, which hold their
    /// titles. On each side, the scales are sorted by weight, ties kept
    /// in the order they were added, and stacked outwards from the chart area, each taking a band
    /// as wide as its measured `sizes` entry (see `axis::size`), with `SCALE_SPACING` between
    /// neighbouring bands. The chart area is what is left over.
    fn layout(&self, sizes: &IndexMap<ScaleId, Scalar>, decorations: &Decorations) -> ChartLayout {
        let padding = *self.padding.value_dyn();

        // The space left, as screen edges: `top` is the smaller y.
        let mut top = padding.top;
        let mut bottom = self.dimension.height - padding.bottom;
        let mut left = padding.left;
        let mut right = self.dimension.width - padding.right;

        let title = decorations.title.map(|height| {
            let position = Position::new((left + right) / 2.0, top);
            top += height + DECORATION_SPACING;
            position
        });

        let legend = decorations.legend.map(|(position, size)| {
            let area = match position {
                LegendPosition::Top | LegendPosition::Hidden => {
                    let area = Rect::new(Position::new(left, top), Dimension::new(right - left, size.height));
                    top += size.height + DECORATION_SPACING;
                    area
                }
                LegendPosition::Bottom => {
                    let area = Rect::new(Position::new(left, bottom - size.height), Dimension::new(right - left, size.height));
                    bottom -= size.height + DECORATION_SPACING;
                    area
                }
                LegendPosition::Left => {
                    let area = Rect::new(Position::new(left, top), Dimension::new(size.width, bottom - top));
                    left += size.width + DECORATION_SPACING;
                    area
                }
                LegendPosition::Right => {
                    let area = Rect::new(Position::new(right - size.width, top), Dimension::new(size.width, bottom - top));
                    right -= size.width + DECORATION_SPACING;
                    area
                }
            };

            (position, area)
        });

        let mut offsets = IndexMap::new();

        let mut stack_side = |side: AxisPosition| -> Scalar {
            let mut on_side: Vec<(i32, usize, &ScaleId)> = self.scales.iter()
                .enumerate()
                .filter(|(_, (_, scale))| scale.axis_options().position() == side)
                .map(|(index, (id, scale))| (scale.axis_options().weight(), index, id))
                .collect();

            on_side.sort_by_key(|(weight, index, _)| (*weight, *index));

            let mut depth = 0.0;
            for (_, _, id) in on_side {
                let size = sizes.get(id).copied().unwrap_or(0.0);

                // Spacing goes only between two bands: never against the chart area, and not
                // next to a scale that takes no space.
                if depth > 0.0 && size > 0.0 {
                    depth += SCALE_SPACING;
                }

                offsets.insert(id.clone(), depth);
                depth += size;
            }

            depth
        };

        let left_bands = stack_side(AxisPosition::Left);
        let right_bands = stack_side(AxisPosition::Right);
        let top_bands = stack_side(AxisPosition::Top);
        let bottom_bands = stack_side(AxisPosition::Bottom);

        let chart_area = Rect::new(
            Position::new(left + left_bands, top + top_bands),
            Dimension::new(
                right - left - left_bands - right_bands,
                bottom - top - top_bands - bottom_bands,
            )
        );

        ChartLayout { chart_area, offsets, title, legend }
    }
}

/// The sizes of what surrounds the scales, measured for one render.
#[derive(Debug, Default)]
struct Decorations {
    /// The chart title's height, when it has one.
    title: Option<Scalar>,
    /// The legend's side and size, when it is shown and has items.
    legend: Option<(LegendPosition, Dimension)>,
}

/// Where everything goes for one render: the area the datasets are drawn in, for each scale the
/// distance from that area to the inner edge of its band, the top centre of the chart title, and
/// the legend's side and area.
#[derive(Debug)]
struct ChartLayout {
    chart_area: Rect,
    offsets: IndexMap<ScaleId, Scalar>,
    title: Option<Position>,
    legend: Option<(LegendPosition, Rect)>,
}

/// The font size of the chart title when the environment has no `Title2` size.
const TITLE_FONT_SIZE: FontSize = 20;

/// Sets the canvas's text style to the chart title's: bold, at the environment's `Title2` size, so
/// it stands out above the labels and follows the font sizes set around the chart.
fn set_title_font(canvas: &mut CanvasContext) {
    let size = canvas.env().value(&EnvironmentFontSize::Title2).unwrap_or(TITLE_FONT_SIZE);
    canvas.set_font_size(size);
    canvas.set_font_weight(FontWeight::Bold);
}

impl ChartLayout {
    fn offset(&self, id: &ScaleId) -> Scalar {
        self.offsets.get(id).copied().unwrap_or(0.0)
    }
}

impl<C: ControllerSequence> CommonWidget for Chart<C> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

/// Syncs every state of the chart: its title, padding and legend position, its scales, and its
/// controllers and their datasets. carbide calls it before every update and event, and `render`
/// calls it first, as carbide's default `render` does, so the scales are fitted and everything is
/// drawn from synced values.
impl<C: ControllerSequence> WidgetSync for Chart<C> {
    fn sync(&mut self, env: &mut Environment) {
        self.padding.sync(env);
        self.legend.sync(env);
        if let Some(title) = &mut self.title {
            title.sync(env);
        }

        self.interactive.sync(env);
        self.hover_mode.sync(env);
        if let Some(hover) = &mut self.hover {
            hover.sync(env);
        }

        self.scales.sync(env);
        self.controllers.sync(env);
    }
}

/// The pointer's gestures: hovering reports what is under it (see `Chart::hover`), a click calls
/// the click action (see `Chart::on_click`), and on zoomable scales (see `LinearScale::zoom`) a
/// drag and the wheel pan, a pinch, or the wheel with Ctrl or Cmd held, zooms about the pointer, a
/// drag with Shift held zooms to the box it draws, a double-click shows the whole range again, and
/// a trackpad's smart zoom toggles between zooming in and the whole range. Over the chart area these move every
/// zoomable scale; over a scale's band, only that one. A chart without these options ignores the
/// pointer.
impl<C: ControllerSequence> MouseEventHandler for Chart<C> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !self.is_interactive() {
            self.release_pointer();
            return;
        }

        let local = |position: Position| Position::new(position.x - self.position.x, position.y - self.position.y);

        match event {
            MouseEvent::Press { button: MouseButton::Left, position, .. } => {
                self.gesture = None;
                let after_drag = self.press.is_some_and(|press| press.moved);
                self.press = Some(Press { inside: self.is_inside(*position), moved: false, after_drag });
            }
            MouseEvent::Move { to, .. } => {
                let at = local(*to);
                self.hover_at(Some(at), ctx.env);
            }
            MouseEvent::Left => self.hover_at(None, ctx.env),
            MouseEvent::Scroll { x, y, mouse_position, modifiers } => {
                let at = local(*mouse_position);
                let scales = self.gesture_scales(at);
                if scales.is_empty() {
                    return;
                }

                let changed = if modifiers.intersects(ModifierKey::CONTROL | ModifierKey::SUPER) {
                    // With Ctrl or Cmd held the wheel zooms, as a pinch does, so a mouse can zoom
                    // too. Ten scroll units are a line of the wheel; scrolling up zooms in.
                    self.zoom_scales(&scales, at, WHEEL_ZOOM_PER_LINE.powf(*y / 10.0))
                } else {
                    // Otherwise it pans, as a drag does, the data moving the way a scroll view's
                    // content would: with Shift, a vertical wheel scrolls sideways.
                    let moved = if modifiers.contains(ModifierKey::SHIFT) { Position::new(-*y, -*x) } else { Position::new(*x, -*y) };
                    self.pan_scales(&scales, at, Position::new(at.x + moved.x, at.y + moved.y))
                };

                // Only a wheel that moved something is taken, so over a chart that can't move
                // that way any further, a scroll view around it scrolls instead.
                if changed {
                    *ctx.consumed = true;
                }
            }
            MouseEvent::SmartScale(position) => {
                // A trackpad's smart zoom toggles: zoomed in, it shows the whole range again, as a
                // double-click does; showing everything, it zooms in by half about the pointer.
                let at = local(*position);
                let scales = self.gesture_scales(at);
                if scales.is_empty() {
                    return;
                }

                let zoomed = scales.iter().any(|id| self.scales.get(id).is_some_and(|scale| !scale.axis_options().zoom().limited(scale.axis_options().min_span()).is_whole()));
                if zoomed {
                    self.change_zoom(&scales, |_, _, _| crate::scale::ZoomWindow::WHOLE);
                } else {
                    self.zoom_scales(&scales, at, SMART_ZOOM);
                }
                *ctx.consumed = true;
            }
            MouseEvent::Scale(delta, position, _) => {
                let at = local(*position);
                let scales = self.gesture_scales(at);
                if scales.is_empty() {
                    return;
                }

                self.zoom_scales(&scales, at, 1.0 / (1.0 + delta).max(0.1));
                *ctx.consumed = true;
            }
            // A press that moved isn't a click, even let go where it started.
            MouseEvent::NClick(MouseButton::Left, _, _, _) | MouseEvent::Click(MouseButton::Left, _, _) if self.press.is_some_and(|press| press.moved) => {}
            MouseEvent::Click(MouseButton::Left, position, modifier_key) => self.click(local(*position), *modifier_key, ctx.env),
            MouseEvent::NClick(MouseButton::Left, position, modifier_key, count) => {
                // Each click of a double-click is a click. A double-click shows the whole range,
                // unless its first press was a drag, which carbide counts as a click when it ends
                // where it started.
                let at = local(*position);
                self.click(at, *modifier_key, ctx.env);
                if *count == 2 && !self.press.is_some_and(|press| press.after_drag) {
                    let scales = self.gesture_scales(at);
                    self.change_zoom(&scales, |_, _, _| crate::scale::ZoomWindow::WHOLE);
                }
            }
            MouseEvent::Drag { button: MouseButton::Left, origin, to, total_delta_xy, modifiers, .. } => {
                // Only a press inside the chart drags it, wherever the pointer goes after.
                let Some(press) = &mut self.press else {
                    return;
                };
                if !press.inside {
                    return;
                }
                if total_delta_xy.x.hypot(total_delta_xy.y) > CLICK_SLOP {
                    press.moved = true;
                }

                let (origin, to) = (local(*origin), local(*to));

                if self.gesture.is_none() {
                    let scales = self.gesture_scales(origin);
                    if scales.is_empty() {
                        return;
                    }

                    self.gesture = Some(if modifiers.contains(ModifierKey::SHIFT) {
                        Gesture::Box { origin, to: origin, scales }
                    } else {
                        Gesture::Pan { last: origin, scales }
                    });
                }

                match self.gesture.clone() {
                    Some(Gesture::Pan { last, scales }) => {
                        self.pan_scales(&scales, last, to);
                        self.gesture = Some(Gesture::Pan { last: to, scales });
                    }
                    Some(Gesture::Box { origin, scales, .. }) => {
                        self.gesture = Some(Gesture::Box { origin, to, scales });
                    }
                    None => {}
                }
                *ctx.consumed = true;
            }
            MouseEvent::Release { button: MouseButton::Left, .. } => {
                if let Some(Gesture::Box { origin, to, scales }) = self.gesture.take() {
                    let area = self.shapes.area();
                    self.change_zoom(&scales, |scale, window, min_span| {
                        // A box too thin along a scale leaves that scale as it is.
                        let extent = match scale.axis() {
                            Axis::Vertical => (to.y - origin.y).abs(),
                            _ => (to.x - origin.x).abs(),
                        };
                        if extent < MIN_BOX {
                            return window;
                        }

                        let (a, b) = (share_at(scale, window, origin, area), share_at(scale, window, to, area));
                        crate::scale::ZoomWindow::new(a.min(b), a.max(b)).limited(min_span)
                    });
                }
            }
            _ => {}
        }
    }
}

/// The last press of the primary button: whether it was inside the chart, whether the pointer
/// has moved further than `CLICK_SLOP` from it since, and whether the press before it had.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Press {
    inside: bool,
    moved: bool,
    after_drag: bool,
}

/// How far the pointer can move from a press, in pixels, for it to still be a click: as far as
/// carbide allows between a press and its release.
const CLICK_SLOP: Scalar = 3.0;

/// How far a trackpad's smart zoom zooms in: to half the range shown.
const SMART_ZOOM: Scalar = 0.5;

/// How far a box must reach along a scale, in pixels, to zoom it.
const MIN_BOX: Scalar = 5.0;

/// Whether `position` is inside `area`, edges included.
fn inside(area: Rect, position: Position) -> bool {
    area.left() <= position.x && position.x <= area.right() && area.bottom() <= position.y && position.y <= area.top()
}

/// The band of a scale on `position`'s side of `chart_area`, `offset` from it and `size` deep, in
/// the chart's own coordinates. `Rect::bottom` is the screen's top edge.
fn band(position: AxisPosition, chart_area: Rect, offset: Scalar, size: Scalar) -> Rect {
    match position {
        AxisPosition::Left => Rect::new(Position::new(chart_area.left() - offset - size, chart_area.bottom()), Dimension::new(size, chart_area.height())),
        AxisPosition::Right => Rect::new(Position::new(chart_area.right() + offset, chart_area.bottom()), Dimension::new(size, chart_area.height())),
        AxisPosition::Top => Rect::new(Position::new(chart_area.left(), chart_area.bottom() - offset - size), Dimension::new(chart_area.width(), size)),
        AxisPosition::Bottom => Rect::new(Position::new(chart_area.left(), chart_area.top() + offset), Dimension::new(chart_area.width(), size)),
    }
}

impl<C: ControllerSequence> Update for Chart<C> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        // Turned off while hovered or dragged, the chart lets go before the next render.
        if !self.is_interactive() {
            self.release_pointer();
        }

        self.fit_scales(ctx.env);
    }
}

impl<C: ControllerSequence> Render for Chart<C> {
    fn render(&mut self, context: &mut RenderContext) {
        self.sync(context.env);

        let mut canvas = CanvasContext::new(self.position, self.dimension, context);

        let sizes: IndexMap<ScaleId, Scalar> = self.scales.iter()
            .map(|(id, scale)| (id.clone(), axis::size(scale, &mut canvas)))
            .collect();

        // The legend's items, from every controller, in order.
        let legend_position = *self.legend.value_dyn();
        let mut legend_items = vec![];
        if legend_position != LegendPosition::Hidden {
            let env = canvas.env();
            self.controllers.foreach(&mut |_, controller| controller.legend_items(env, &mut legend_items));
        }

        let padding = *self.padding.value_dyn();
        let legend_width = self.dimension.width - padding.left - padding.right;
        let legend_layout = legend::layout(&legend_items, legend_position, legend_width, |text| canvas.measure_text(text));

        let title = self.title.as_ref().map(|title| title.value_dyn().clone());

        let decorations = Decorations {
            title: title.as_ref().map(|title| {
                canvas.save();
                set_title_font(&mut canvas);
                let size = canvas.measure_text(title);
                canvas.restore();
                size.height
            }),
            legend: (!legend_items.is_empty()).then_some((legend_position, legend_layout.size)),
        };

        let layout = self.layout(&sizes, &decorations);
        let chart_area = layout.chart_area;

        self.bands = self.scales.iter()
            .filter(|(_, scale)| scale.axis() != Axis::Radial)
            .map(|(id, scale)| (id.clone(), band(scale.axis_options().position(), chart_area, layout.offset(id), sizes[id])))
            .collect();

        // Polar scales share a circle inside the chart area, with room for the surrounding labels.
        // Labels the controllers draw outside the circle need room too.
        let mut outside = vec![];
        let scales = &self.scales;
        self.controllers.foreach(&mut |_, controller| {
            controller.polar_labels(scales, &mut |text| canvas.measure_text(text), &mut |angle, size, gap| {
                outside.push(RimLabel { angle, size, gap });
            });
        });

        let circle = axis::polar_circle(&self.scales, &mut canvas, chart_area, &outside);
        self.scales.set_circle(circle);

        for (id, scale) in self.scales.iter() {
            axis::draw_grid(scale, &mut canvas, chart_area, layout.offset(id));
        }
        axis::draw_polar_grid(&self.scales, &mut canvas);

        for (id, scale) in self.scales.iter() {
            axis::draw_border(scale, &mut canvas, chart_area, layout.offset(id));
            axis::draw_title(scale, &mut canvas, chart_area, layout.offset(id), sizes[id]);
        }
        axis::draw_polar_border(&self.scales, &mut canvas);

        // Draw the title of the chart
        if let (Some(title), Some(position)) = (&title, layout.title) {
            let color = axis_style(canvas.env()).label_color(canvas.env());
            canvas.save();
            canvas.set_fill_style(color);
            canvas.set_text_align(Alignment::Top);
            set_title_font(&mut canvas);
            canvas.fill_text(title, position.x, position.y);
            canvas.restore();
        }

        // Draw the legends of the chart
        if let Some((position, area)) = layout.legend {
            legend::draw(&legend_items, &legend_layout, position, &mut canvas, area);
        }

        // Every datum's shape, laid out and drawn by its controller, then kept for picking.
        // A zoomed scale shows only part of its data: the rest is clipped to the chart area. Without
        // a zoom nothing is clipped, so points on the area's edges are drawn whole.
        let zoomed = self.scales.iter().any(|(_, scale)| !scale.axis_options().window().is_whole());
        if zoomed {
            let on_screen = Rect::new(Position::new(self.position.x + chart_area.left(), self.position.y + chart_area.bottom()), chart_area.dimension);
            canvas.render_context().render.clip(on_screen);
        }

        let mut shapes = Shapes::new(chart_area, circle);
        let scales = &self.scales;
        self.controllers.foreach(&mut |index, controller| {
            shapes.set_controller(index);
            controller.draw(&mut canvas, scales, chart_area, &mut shapes);
        });

        shapes.finish();

        // The hovered data outlined, and the box a Shift-drag zooms to, in the label colour, which
        // stands out on the background.
        let ink = axis_style(canvas.env()).label_color(canvas.env());
        if let Some(hover) = &self.hover && self.is_interactive() && let Some(pick) = &*hover.value_dyn() {
            canvas.save();
            canvas.set_line_width(2.0);
            canvas.set_stroke_style(ink);
            for key in &pick.data {
                shapes.outline(&mut canvas, *key);
            }
            canvas.restore();
        }
        if let Some(Gesture::Box { origin, to, .. }) = &self.gesture {
            let Rgba(r, g, b, a) = Rgba::from(ink);
            canvas.save();
            canvas.begin_path();
            canvas.move_to(origin.x, origin.y);
            canvas.line_to(to.x, origin.y);
            canvas.line_to(to.x, to.y);
            canvas.line_to(origin.x, to.y);
            canvas.close_path();
            canvas.set_fill_style(rgba(r, g, b, a * 0.15));
            canvas.fill();
            canvas.set_line_width(1.0);
            canvas.set_stroke_style(ink);
            canvas.stroke();
            canvas.restore();
        }

        if zoomed {
            canvas.render_context().render.pop_clip();
        }
        self.shapes = shapes;
    }
}

#[cfg(test)]
mod tests {
    use carbide::color::RED;
    use carbide::draw::{Dimension, Position, Rect, Scalar};
    use carbide::environment::Environment;
    use carbide::state::{LocalState, Map1, ReadStateExt, State};
    use carbide::widget::WidgetSync;
    use indexmap::IndexMap;
    use crate::controller::{BarController, ControllerSequence, DatasetController, LineController, ScatterController};
    use crate::dataset::DatasetExt;
    use crate::sequence::Sequence;
    use crate::{DataColor, ScatterDataSet, StackId};
    use crate::scale::{Axis, AxisPosition, CategoryScale, GridShape, LinearScale, PolarRole, ScaleId, ScaleKind, Scales, System};
    use super::{Chart, Decorations, SCALE_SPACING};

    /// Datasets spread over several controllers share the chart's scales: they fit exactly as
    /// they would with every dataset in one controller.
    #[test]
    fn scales_fit_the_bounds_of_every_controller() {
        let first = vec![75.0, 70.0, 80.0, -60.0];
        let second = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        let split = fitted(Chart::new((
            ScatterController::new(first.clone()),
            ScatterController::new(second.clone()),
        )));
        let combined = fitted(Chart::new(ScatterController::new((first, second))));

        assert_eq!(scale_range(&split, ScaleId::X), scale_range(&combined, ScaleId::X));
        assert_eq!(scale_range(&split, ScaleId::Y), scale_range(&combined, ScaleId::Y));
        assert_eq!(split.scales.get(&ScaleId::Y).unwrap().ticks(), combined.scales.get(&ScaleId::Y).unwrap().ticks());

        // Sanity check that the shared range really spans both controllers' data: the y range
        // must cover the first controller's -60 and the second controller's 92.
        let (y_min, y_max) = scale_range(&split, ScaleId::Y);
        assert!(y_min <= -60.0 && y_max >= 92.0, "y range {y_min}..{y_max} should span both controllers' data");
    }

    /// An empty dataset contributes nothing to the bounds, and a chart with no data at all
    /// doesn't panic: its scales fall back to `0..1`.
    #[test]
    fn empty_datasets_do_not_change_the_scales() {
        let empty: Vec<Scalar> = vec![];
        let data = vec![100.0, 150.0, 200.0];

        let with_empty = fitted(Chart::new(ScatterController::new((empty.clone(), data.clone()))));
        let without = fitted(Chart::new(ScatterController::new(data)));

        assert_eq!(scale_range(&with_empty, ScaleId::X), scale_range(&without, ScaleId::X));
        assert_eq!(scale_range(&with_empty, ScaleId::Y), scale_range(&without, ScaleId::Y));

        let only_empty = fitted(Chart::new(ScatterController::new(empty)));
        assert_eq!(scale_range(&only_empty, ScaleId::Y), (0.0, 1.0));
    }

    /// A dataset's scale binding is a state: changing it moves the dataset's bounds to the newly
    /// bound scale on the next fit.
    #[test]
    fn scale_binding_is_reactive() {
        let mut binding = LocalState::new(ScaleId::new("elsewhere"));
        let small = vec![1.0, 2.0];
        let large = vec![100.0, 200.0].y_scale(binding.clone());

        let mut chart = Chart::new(ScatterController::new((small, large)));
        let mut env = Environment::new();

        // Bound to a scale the chart doesn't have: the large values don't reach Y.
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 100.0, "Y should only cover the small dataset, but reaches {y_max}");

        // Rebound to Y: the next fit includes the large values.
        *binding.value_mut() = ScaleId::Y;
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 200.0, "Y should cover the large dataset after rebinding, but ends at {y_max}");
    }

    /// A dataset's stack is a state: moving a dataset into another's stack makes its bars add up
    /// with that dataset's on the next fit, and moving it out again separates them.
    #[test]
    fn stack_is_reactive() {
        let mut stack = LocalState::new(StackId::new("alone"));
        let first = vec![10.0, 20.0].stack(StackId::new("costs"));
        let second = vec![5.0, 8.0].stack(stack.clone());

        let mut chart = Chart::new(BarController::new((first, second)));
        let mut env = Environment::new();

        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 28.0, "in separate stacks the bars don't add up, but Y reaches {y_max}");

        *stack.value_mut() = StackId::new("costs");
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 28.0, "in one stack Y should reach 20 + 8, but ends at {y_max}");

        *stack.value_mut() = StackId::new("alone");
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 28.0, "moved out again, the bars should no longer add up, but Y reaches {y_max}");
    }

    /// A dataset held in a state can change after the chart is built: the next fit follows it.
    /// Options still work on it.
    #[test]
    fn a_dataset_in_a_state_can_change() {
        let mut points = LocalState::new(vec![Position::new(0.0, 1.0), Position::new(1.0, 2.0)]);

        let mut chart = Chart::new(ScatterController::new(points.clone().color(DataColor::Color(RED))));
        let mut env = Environment::new();

        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 10.0, "Y should fit the first points, but reaches {y_max}");

        *points.value_mut() = vec![Position::new(0.0, 1.0), Position::new(1.0, 50.0)];
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 50.0, "Y should follow the changed points, but ends at {y_max}");
    }

    /// The datasets of a controller can be a state of datasets that are states themselves, so
    /// datasets can be added at runtime, and each one can change on its own.
    #[test]
    fn datasets_in_a_state_can_be_added() {
        let small = LocalState::new(vec![Position::new(0.0, 1.0), Position::new(1.0, 2.0)]);
        let mut large = LocalState::new(vec![Position::new(4.0, 50.0), Position::new(5.0, 100.0)]);
        let mut datasets: LocalState<Vec<LocalState<Vec<Position>>>> = LocalState::new(vec![small.clone()]);

        let mut chart = Chart::new(ScatterController::new(datasets.clone()));
        let mut env = Environment::new();

        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 10.0, "Y should fit the one dataset, but reaches {y_max}");

        *datasets.value_mut() = vec![small.clone(), large.clone()];
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 100.0, "Y should include the added dataset, but ends at {y_max}");

        *large.value_mut() = vec![Position::new(4.0, 50.0), Position::new(5.0, 300.0)];
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 300.0, "Y should follow a change inside an added dataset, but ends at {y_max}");
    }

    /// The controllers of a chart can be a state too, so controllers can be added at runtime.
    #[test]
    fn controllers_in_a_state_can_be_added() {
        let small = ScatterController::new(vec![(0.0, 1.0), (1.0, 2.0)]);
        let large = ScatterController::new(vec![(0.0, 50.0), (1.0, 100.0)]);
        let mut controllers = LocalState::new(vec![small.clone()]);

        let mut chart = Chart::new(controllers.clone());
        let mut env = Environment::new();

        chart.fit_scales(&mut env);
        assert_eq!(chart.controllers.count(), 1);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 10.0, "Y should fit the one controller, but reaches {y_max}");

        *controllers.value_mut() = vec![small, large];
        chart.fit_scales(&mut env);
        assert_eq!(chart.controllers.count(), 2);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 100.0, "Y should include the added controller, but ends at {y_max}");
    }

    /// Controllers of different types, boxed, share a `Vec` or a state of a `Vec`: a chart
    /// whose chart types are chosen at runtime. Boxed controllers keep their behaviour: the bars
    /// still pick the category x scale and include zero, and the line's values reach the y scale.
    #[test]
    fn boxed_controllers_of_different_types_can_be_mixed() {
        let bars = BarController::new(vec![12.0, 19.0, 3.0]).boxed();
        let line = LineController::new(vec![40.0, 50.0, 45.0]).boxed();

        let chart = fitted(Chart::new(vec![bars.clone(), line.clone()]));
        assert_eq!(chart.controllers.count(), 2);
        let (y_min, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_min <= 0.0 && y_max >= 50.0, "Y should cover the bars from zero and the line, got {y_min}..{y_max}");
        assert!(chart.scales.get(&ScaleId::X).unwrap().normalize(0.0) > 0.0, "the bars should offset the x scale");

        let mut controllers = LocalState::new(vec![bars.clone()]);
        let mut chart = Chart::new(controllers.clone());
        let mut env = Environment::new();

        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 40.0, "Y should fit only the bars, but reaches {y_max}");

        *controllers.value_mut() = vec![bars, line.boxed()];
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 50.0, "Y should include the added line, but ends at {y_max}");
    }

    /// Datasets of different types, boxed, share a `Vec` or a state of a `Vec` in one controller.
    /// A boxed dataset keeps its options: the one bound to `percent` fits that scale, not Y.
    #[test]
    fn boxed_datasets_of_different_types_can_be_mixed() {
        const PERCENT: ScaleId = ScaleId::new("percent");

        let positions = vec![Position::new(0.0, 10.0), Position::new(1.0, 20.0)].boxed::<dyn ScatterDataSet>();
        let values = LocalState::new(vec![15.0, 25.0]).boxed::<dyn ScatterDataSet>();
        let margin = vec![0.2, 0.4].y_scale(PERCENT).boxed();

        let chart = fitted(
            Chart::new(ScatterController::new(vec![positions.clone(), values.clone(), margin]))
                .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
        );

        let (y_min, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_min >= 5.0 && y_max >= 25.0, "Y should fit the positions and values only, got {y_min}..{y_max}");
        let (_, percent_max) = scale_range(&chart, PERCENT);
        assert!(percent_max < 1.0, "percent should fit only the margin, but ends at {percent_max}");

        let mut datasets = LocalState::new(vec![positions]);
        let mut chart = Chart::new(ScatterController::new(datasets.clone()));
        let mut env = Environment::new();

        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 25.0, "Y should fit only the positions, but reaches {y_max}");

        *datasets.value_mut() = vec![vec![Position::new(0.0, 10.0)].boxed(), values];
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 25.0, "Y should include the added values, but ends at {y_max}");
    }

    /// A dataset mapped from the environment (`read_map_env`) only has a value once synced. The
    /// chart syncs it, as carbide syncs the chart before every update and render, so the scales
    /// fit the mapped values.
    #[test]
    fn the_chart_syncs_datasets_mapped_from_the_environment() {
        let model = LocalState::new(vec![10.0, 20.0, 30.0]);
        let mapped = Map1::read_map_env(model, |_env: &mut Environment, values: &Vec<Scalar>| values.clone());

        let mut chart = Chart::new(ScatterController::new(mapped));
        let mut env = Environment::new();

        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max < 10.0, "unsynced, the mapped dataset should have no values, but Y reaches {y_max}");

        chart.sync(&mut env);
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 30.0, "synced, Y should fit the mapped values, but ends at {y_max}");
    }

    /// The chart syncs datasets held inside a state too: a `LocalState` lends its collection
    /// mutably, so the environment-mapped dataset in it is synced.
    #[test]
    fn the_chart_syncs_datasets_inside_a_state() {
        let model = LocalState::new(vec![10.0, 20.0, 30.0]);
        let mapped = Map1::read_map_env(model, |_env: &mut Environment, values: &Vec<Scalar>| values.clone());
        let datasets = LocalState::new(vec![mapped]);

        let mut chart = Chart::new(ScatterController::new(datasets));
        let mut env = Environment::new();

        chart.sync(&mut env);
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 30.0, "Y should fit the synced dataset inside the state, but ends at {y_max}");
    }

    /// `sync` reports whether anything may have changed: plain data never changes, while a state
    /// may have, and says so until carbide can tell.
    #[test]
    fn sync_reports_whether_anything_may_have_changed() {
        let mut env = Environment::new();

        let mut plain = ScatterController::new((vec![1.0, 2.0], vec![(0.0, 1.0)]));
        assert!(!DatasetController::sync(&mut plain, &mut env), "plain data has nothing that changes");

        let mut with_state = ScatterController::new((vec![1.0, 2.0], LocalState::new(vec![3.0])));
        assert!(DatasetController::sync(&mut with_state, &mut env), "a state may have changed");

        let mut with_option_state = ScatterController::new(vec![1.0, 2.0].color(LocalState::new(DataColor::Color(RED))));
        assert!(DatasetController::sync(&mut with_option_state, &mut env), "an option state may have changed");
    }

    /// Any other state, such as one mapped from a model, works once boxed with `as_dyn_read`.
    /// Plain values in it still use their index as x.
    #[test]
    fn a_dataset_mapped_from_a_model() {
        let mut model = LocalState::new(vec![2.0, 4.0, 6.0]);
        let doubled = Map1::read_map(model.clone(), |values: &Vec<Scalar>| {
            values.iter().map(|value| value * 2.0).collect::<Vec<Scalar>>()
        }).as_dyn_read();

        let mut chart = Chart::new(ScatterController::new(doubled));
        let mut env = Environment::new();

        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 12.0, "Y should fit the doubled values, but ends at {y_max}");
        let (_, x_max) = scale_range(&chart, ScaleId::X);
        assert!(x_max >= 2.0 && x_max < 6.0, "X should be the indices 0..=2, but ends at {x_max}");

        *model.value_mut() = vec![2.0, 4.0, 60.0];
        chart.fit_scales(&mut env);
        let (_, y_max) = scale_range(&chart, ScaleId::Y);
        assert!(y_max >= 120.0, "Y should follow the model, but ends at {y_max}");
    }

    /// A dataset bound to a custom scale fits that scale, not `ScaleId::Y`, and a scale on the
    /// right gets a band on the right of the chart area.
    #[test]
    fn custom_scales_get_their_own_bounds_and_side() {
        const PERCENT: ScaleId = ScaleId::new("percent");

        let revenue = vec![120.0, 150.0, 190.0];
        let margin = vec![0.12, 0.25, 0.31].y_scale(PERCENT);

        let mut chart = fitted(
            Chart::new(ScatterController::new((revenue, margin)))
                .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
        );

        let (percent_min, percent_max) = scale_range(&chart, PERCENT);
        assert!(percent_min <= 0.12 && percent_max >= 0.31 && percent_max < 1.0, "percent should fit only the margin, got {percent_min}..{percent_max}");

        let (y_min, _) = scale_range(&chart, ScaleId::Y);
        assert!(y_min >= 100.0, "Y should fit only the revenue, but starts at {y_min}");

        chart.dimension = Dimension::new(400.0, 300.0);
        let area = chart.layout(&sizes(&[(ScaleId::Y, 35.0), (PERCENT, 45.0)]), &Decorations::default()).chart_area;
        assert_eq!(area.left(), 30.0 + 35.0, "left: padding + Y's band");
        assert_eq!(area.right(), 400.0 - 30.0 - 45.0, "right: padding + percent's band");
    }

    /// Scales can be named from runtime data, one per data source here, and a dataset bound
    /// with an owned id finds the scale added with an equal id.
    #[test]
    fn scales_named_at_runtime_fit_their_datasets() {
        let source = String::from("sensor");
        let id = ScaleId::owned(format!("{source}-y"));

        let readings = vec![40.0, 55.0, 70.0].y_scale(id.clone());

        let chart = fitted(
            Chart::new(ScatterController::new(readings))
                .scale(format!("{source}-y"), LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
        );

        let (min, max) = scale_range(&chart, id);
        assert!(min <= 40.0 && max >= 70.0, "the runtime scale should fit the readings, got {min}..{max}");
        assert_eq!(scale_range(&chart, ScaleId::Y), (0.0, 1.0), "Y has no data bound to it");
    }

    /// Decorations are laid out from the outside in: the chart title along the top, then the
    /// legend on its side, each taking its size and a gap off the space left, and the scale bands
    /// inside that.
    #[test]
    fn decorations_are_laid_out_from_the_outside_in() {
        use crate::LegendPosition;
        use super::DECORATION_SPACING;

        let mut chart = fitted(Chart::new(ScatterController::new(vec![1.0, 2.0])));
        chart.dimension = Dimension::new(400.0, 300.0);
        let bands = sizes(&[(ScaleId::X, 20.0), (ScaleId::Y, 35.0)]);

        let decorations = Decorations { title: Some(14.0), legend: Some((LegendPosition::Top, Dimension::new(340.0, 16.0))) };
        let layout = chart.layout(&bands, &decorations);

        assert_eq!(layout.title, Some(Position::new(200.0, 30.0)), "the title is centred along the top, inside the padding");
        let (side, legend) = layout.legend.unwrap();
        assert_eq!(side, LegendPosition::Top);
        assert_eq!((legend.left(), legend.bottom(), legend.width(), legend.height()), (30.0, 30.0 + 14.0 + DECORATION_SPACING, 340.0, 16.0), "the legend sits under the title, across the chart");
        assert_eq!(layout.chart_area.bottom(), 30.0 + 14.0 + DECORATION_SPACING + 16.0 + DECORATION_SPACING, "the chart area starts under the legend");
        assert_eq!(layout.chart_area.left(), 30.0 + 35.0, "nothing came off the sides");

        for (side, check) in [
            (LegendPosition::Left, Box::new(|area: Rect| area.left() == 30.0 + 50.0 + DECORATION_SPACING + 35.0) as Box<dyn Fn(Rect) -> bool>),
            (LegendPosition::Right, Box::new(|area: Rect| area.right() == 400.0 - 30.0 - 50.0 - DECORATION_SPACING)),
            (LegendPosition::Bottom, Box::new(|area: Rect| area.top() == 300.0 - 30.0 - 16.0 - DECORATION_SPACING - 20.0)),
        ] {
            let decorations = Decorations { legend: Some((side, Dimension::new(50.0, 16.0))), ..Decorations::default() };
            let layout = chart.layout(&bands, &decorations);
            assert!(check(layout.chart_area), "a legend on the {side:?} should take its size and a gap off that side: {:?}", layout.chart_area);
        }
    }

    /// The padding, the title and the legend position are states: the chart reads them as they
    /// were last synced.
    #[test]
    fn padding_and_legend_are_states() {
        use crate::LegendPosition;

        let mut padding = LocalState::new(10.0);
        let mut legend = LocalState::new(LegendPosition::Hidden);
        let mut chart = fitted(Chart::new(ScatterController::new(vec![1.0, 2.0])).padding(padding.clone()).legend(legend.clone()));
        chart.dimension = Dimension::new(400.0, 300.0);
        let bands = sizes(&[(ScaleId::X, 20.0), (ScaleId::Y, 35.0)]);

        assert_eq!(chart.layout(&bands, &Decorations::default()).chart_area.left(), 10.0 + 35.0);

        *padding.value_mut() = 40.0;
        *legend.value_mut() = LegendPosition::Right;
        chart.sync(&mut Environment::new());

        assert_eq!(chart.layout(&bands, &Decorations::default()).chart_area.left(), 40.0 + 35.0);
        assert_eq!(*chart.legend.value_dyn(), LegendPosition::Right);
    }

    /// A scale's options are states, synced with the chart, so the layout follows them: moving
    /// the y scale to the right moves its band, and the chart area with it.
    #[test]
    fn scale_options_are_states() {
        let mut position = LocalState::new(AxisPosition::Left);
        let mut chart = fitted(Chart::new(ScatterController::new(vec![1.0, 2.0])).scale(ScaleId::Y, LinearScale::new(Axis::Vertical).position(position.clone())));
        chart.dimension = Dimension::new(400.0, 300.0);
        let bands = sizes(&[(ScaleId::X, 20.0), (ScaleId::Y, 35.0)]);

        chart.sync(&mut Environment::new());
        assert_eq!(chart.layout(&bands, &Decorations::default()).chart_area.left(), 30.0 + 35.0, "the y band is on the left");

        *position.value_mut() = AxisPosition::Right;
        chart.sync(&mut Environment::new());
        let area = chart.layout(&bands, &Decorations::default()).chart_area;
        assert_eq!((area.left(), area.right()), (30.0, 400.0 - 30.0 - 35.0), "the y band moved to the right");
    }

    /// A scale's axis title is a state, synced with the chart, so a changed title is drawn on the
    /// next render.
    #[test]
    fn axis_titles_are_states() {
        let mut title = LocalState::new("Revenue".to_string());
        let mut chart = Chart::new(ScatterController::new(vec![1.0, 2.0])).scale(ScaleId::Y, LinearScale::new(Axis::Vertical).title(title.clone()));

        chart.sync(&mut Environment::new());
        assert_eq!(chart.scales.get(&ScaleId::Y).unwrap().axis_options().title().as_deref(), Some("Revenue"));

        *title.value_mut() = "Costs".to_string();
        chart.sync(&mut Environment::new());
        assert_eq!(chart.scales.get(&ScaleId::Y).unwrap().axis_options().title().as_deref(), Some("Costs"));
    }

    /// Controllers add a legend entry for each dataset with a name, in order, marked after what
    /// they draw: bars as squares, lines as lines (squares when filled), scatter points as points.
    #[test]
    fn controllers_report_their_named_datasets_for_the_legend() {
        use crate::{Fill, LegendMarker};

        let bars = BarController::new((vec![1.0].label("Revenue"), vec![2.0]));
        let lines = LineController::new((vec![1.0].label("Trend"), vec![2.0].fill(Fill::Origin).label("Area")));
        let points = ScatterController::new(vec![(1.0, 2.0)].label("Readings"));

        let env = Environment::new();
        let mut items = vec![];
        bars.legend_items(&env, &mut items);
        lines.legend_items(&env, &mut items);
        points.legend_items(&env, &mut items);
        ScatterController::new(vec![(1.0, 2.0)].label("Joined")).show_line(true).legend_items(&env, &mut items);

        let entries: Vec<(&str, LegendMarker)> = items.iter().map(|item| (item.label.as_str(), item.marker)).collect();
        assert_eq!(entries, vec![
            ("Revenue", LegendMarker::Square),
            ("Trend", LegendMarker::Line),
            ("Area", LegendMarker::Square),
            ("Readings", LegendMarker::Point),
            ("Joined", LegendMarker::Line),
        ], "the dataset without a name is left out");
    }

    /// Without a scale on the right there is no band there.
    #[test]
    fn default_scales_have_no_right_band() {
        let mut chart = fitted(Chart::new(ScatterController::new(vec![1.0, 2.0])));

        chart.dimension = Dimension::new(400.0, 300.0);
        let layout = chart.layout(&sizes(&[(ScaleId::X, 20.0), (ScaleId::Y, 35.0)]), &Decorations::default());
        assert_eq!(layout.chart_area.right(), 400.0 - 30.0);
        assert_eq!(layout.chart_area.left(), 30.0 + 35.0);
        assert_eq!(layout.chart_area.height(), 300.0 - 30.0 - 30.0 - 20.0, "X's band comes off the height");
    }

    /// Scales on the same side stack outwards in the order they were added, each starting
    /// `SCALE_SPACING` past where the band inside it ends; the chart area gives up the bands and
    /// the spacing between them.
    #[test]
    fn scales_on_one_side_stack_outwards_in_order() {
        const INNER: ScaleId = ScaleId::new("inner");
        const OUTER: ScaleId = ScaleId::new("outer");

        let mut chart = Chart::new(ScatterController::new(vec![1.0, 2.0]))
            .scale(INNER, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
            .scale(OUTER, LinearScale::new(Axis::Vertical).position(AxisPosition::Right));

        chart.dimension = Dimension::new(400.0, 300.0);
        let layout = chart.layout(&sizes(&[(INNER, 40.0), (OUTER, 25.0)]), &Decorations::default());

        assert_eq!(layout.offset(&INNER), 0.0);
        assert_eq!(layout.offset(&OUTER), 40.0 + SCALE_SPACING);
        assert_eq!(layout.chart_area.right(), 400.0 - 30.0 - 40.0 - SCALE_SPACING - 25.0);
    }

    /// No spacing is added next to a scale that takes no space, so hiding a scale's band doesn't
    /// leave a gap behind.
    #[test]
    fn empty_bands_get_no_spacing() {
        const EMPTY: ScaleId = ScaleId::new("empty");
        const OUTER: ScaleId = ScaleId::new("outer");

        let mut chart = Chart::new(ScatterController::new(vec![1.0, 2.0]))
            .scale(EMPTY, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
            .scale(OUTER, LinearScale::new(Axis::Vertical).position(AxisPosition::Right));

        chart.dimension = Dimension::new(400.0, 300.0);
        let layout = chart.layout(&sizes(&[(EMPTY, 0.0), (OUTER, 25.0)]), &Decorations::default());

        assert_eq!(layout.offset(&OUTER), 0.0);
        assert_eq!(layout.chart_area.right(), 400.0 - 30.0 - 25.0);
    }

    /// A higher weight moves a scale further out, whatever order it was added in; Y keeps its
    /// default weight 0 and stays innermost on the left.
    #[test]
    fn higher_weights_are_further_from_the_chart_area() {
        const HEAVY: ScaleId = ScaleId::new("heavy");
        const LIGHT: ScaleId = ScaleId::new("light");

        let mut chart = Chart::new(ScatterController::new(vec![1.0, 2.0]))
            .scale(HEAVY, LinearScale::new(Axis::Vertical).weight(2))
            .scale(LIGHT, LinearScale::new(Axis::Vertical).weight(1));

        chart.dimension = Dimension::new(400.0, 300.0);
        let layout = chart.layout(&sizes(&[(ScaleId::Y, 30.0), (HEAVY, 20.0), (LIGHT, 10.0)]), &Decorations::default());

        assert_eq!(layout.offset(&ScaleId::Y), 0.0);
        assert_eq!(layout.offset(&LIGHT), 30.0 + SCALE_SPACING);
        assert_eq!(layout.offset(&HEAVY), 30.0 + SCALE_SPACING + 10.0 + SCALE_SPACING);
        assert_eq!(layout.chart_area.left(), 30.0 + 30.0 + 10.0 + 20.0 + 2.0 * SCALE_SPACING);
    }

    /// A bar controller makes its y scale include zero, so bars grow from it, and offsets its x
    /// scale so the first and last bars sit inside the chart area. A scatter controller on its own
    /// does neither.
    #[test]
    fn bars_include_zero_and_offset_their_x_scale() {
        let revenue = vec![120.0, 150.0, 190.0];

        let bars = fitted(Chart::new(BarController::new(revenue.clone())));
        let (y_min, _) = scale_range(&bars, ScaleId::Y);
        assert!(y_min <= 0.0, "bars should start the y scale at zero, but it starts at {y_min}");

        let x = bars.scales.get(&ScaleId::X).unwrap();
        assert!(x.normalize(0.0) > 0.0 && x.normalize(2.0) < 1.0, "the first and last bar should sit inside the chart area");

        let points = fitted(Chart::new(ScatterController::new(revenue)));
        let (y_min, _) = scale_range(&points, ScaleId::Y);
        assert!(y_min > 0.0, "without bars the y scale needn't include zero, but starts at {y_min}");
        assert_eq!(points.scales.get(&ScaleId::X).unwrap().normalize(0.0), 0.0);
    }

    /// A bar chart's default x scale is a category scale, so six bars get six whole ticks; a
    /// scatter chart's stays linear and picks half steps for the same data (the case that made
    /// bars too narrow). A scale set with `Chart::scale` replaces the bar chart's default.
    #[test]
    fn bar_charts_default_to_a_category_x_scale() {
        let votes = vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0];

        let bars = fitted(Chart::new(BarController::new(votes.clone())));
        assert_eq!(bars.scales.get(&ScaleId::X).unwrap().ticks(), &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);

        let points = fitted(Chart::new(ScatterController::new(votes.clone())));
        assert!(points.scales.get(&ScaleId::X).unwrap().ticks().contains(&0.5), "a linear x scale picks half steps here");

        let bars_on_linear = fitted(
            Chart::new(BarController::new(votes))
                .scale(ScaleId::X, LinearScale::new(Axis::Horizontal))
        );
        assert!(bars_on_linear.scales.get(&ScaleId::X).unwrap().ticks().contains(&0.5), "an explicit scale replaces the default");
    }

    /// A line chart's default x scale is a category scale like a bar chart's, but without the
    /// offset: the first and last points sit on the chart area's edges. Bars added to it still
    /// offset it, as their `request_offset` applies to any scale.
    #[test]
    fn line_charts_default_to_a_category_x_scale_without_offset() {
        let values = vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0];

        let lines = fitted(Chart::new(LineController::new(values.clone())));
        let x = lines.scales.get(&ScaleId::X).unwrap();
        assert_eq!(x.ticks(), &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(x.normalize(0.0), 0.0);
        assert_eq!(x.normalize(5.0), 1.0);

        let lines_and_bars = fitted(Chart::new((LineController::new(values.clone()), BarController::new(values))));
        assert!(lines_and_bars.scales.get(&ScaleId::X).unwrap().normalize(0.0) > 0.0, "bars offset the line chart's x scale");
    }

    /// Bars get grid lines between their groups whichever controller comes first: the grid
    /// offset is a request any controller can make, not part of the first controller's default
    /// scale.
    #[test]
    fn bars_get_grid_lines_between_groups_in_any_controller_order() {
        let values = vec![12.0, 19.0, 3.0];

        for chart in [
            fitted(Chart::new((LineController::new(values.clone()).boxed(), BarController::new(values.clone()).boxed()))),
            fitted(Chart::new((BarController::new(values.clone()).boxed(), LineController::new(values.clone()).boxed()))),
        ] {
            let x = chart.scales.get(&ScaleId::X).unwrap();
            assert_eq!(x.ticks(), &[0.0, 1.0, 2.0], "the x scale should be a category scale");
            assert_eq!(x.grid_line_values(), vec![-0.5, 0.5, 1.5, 2.5], "grid lines should sit between the bar groups");
        }

        let lines = fitted(Chart::new(LineController::new(values)));
        assert_eq!(lines.scales.get(&ScaleId::X).unwrap().grid_line_values(), vec![0.0, 1.0, 2.0], "without bars, grid lines sit on the categories");
    }

    /// A category scale set with `Chart::scale` gets the bars' grid offset unless it sets one
    /// itself, and is never replaced by a default.
    #[test]
    fn explicit_category_scales_get_grid_offset_unless_set() {
        let votes = vec![12.0, 19.0, 3.0];

        let unset = fitted(Chart::new(BarController::new(votes.clone())).scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(["Red", "Blue", "Yellow"])));
        assert_eq!(unset.scales.get(&ScaleId::X).unwrap().grid_line_values(), vec![-0.5, 0.5, 1.5, 2.5]);

        let off = fitted(Chart::new(BarController::new(votes)).scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).grid_offset(false)));
        assert_eq!(off.scales.get(&ScaleId::X).unwrap().grid_line_values(), vec![0.0, 1.0, 2.0], "an explicit setting wins over the request");
    }

    /// The default scales follow the controllers on every fit: a chart whose controllers start
    /// empty gets a category x scale once bars are added, and a linear one again once they are
    /// removed.
    #[test]
    fn default_scales_follow_the_controllers() {
        let mut controllers: LocalState<Vec<Box<dyn DatasetController>>> = LocalState::new(vec![]);
        let mut chart = Chart::new(controllers.clone());
        let mut env = Environment::new();

        let x_ticks = |chart: &Chart<LocalState<Vec<Box<dyn DatasetController>>>>| chart.scales.get(&ScaleId::X).unwrap().ticks().to_vec();
        // A linear scale picks fractional steps for these small ranges; a category scale never does.
        let linear = |ticks: Vec<Scalar>| ticks.iter().any(|tick| tick.fract() != 0.0);

        chart.fit_scales(&mut env);
        assert!(linear(x_ticks(&chart)), "with no controllers, x should be linear: {:?}", x_ticks(&chart));

        *controllers.value_mut() = vec![BarController::new(vec![1.0, 2.0, 3.0]).boxed()];
        chart.fit_scales(&mut env);
        assert_eq!(x_ticks(&chart), vec![0.0, 1.0, 2.0], "bars should make x a category scale");

        *controllers.value_mut() = vec![ScatterController::new(vec![1.0, 2.0, 3.0]).boxed()];
        chart.fit_scales(&mut env);
        assert!(linear(x_ticks(&chart)), "without bars, x should be linear again: {:?}", x_ticks(&chart));
    }

    /// Dates make a default scale a time scale, whatever the controller asks for: the x of a bar
    /// chart, which would be a category scale, and the y of horizontal bars, where the dates go.
    /// Numbers keep the scales the controllers ask for, and a scale set on the chart is kept.
    #[test]
    fn dates_get_a_default_time_scale() {
        use chrono::{DateTime, FixedOffset, TimeZone};
        use crate::ScaleKind;

        fn kinds<C: ControllerSequence>(chart: &Chart<C>) -> (Option<ScaleKind>, Option<ScaleKind>) {
            (chart.default_kinds.get(&ScaleId::X).copied(), chart.default_kinds.get(&ScaleId::Y).copied())
        }

        let utc = FixedOffset::east_opt(0).unwrap();
        let days: Vec<(DateTime<FixedOffset>, Scalar)> = (1..=3)
            .map(|day| (utc.with_ymd_and_hms(2026, 3, day, 0, 0, 0).unwrap(), day as Scalar))
            .collect();

        let lines = fitted(Chart::new(LineController::new(days.clone())));
        assert_eq!(kinds(&lines), (Some(ScaleKind::Time), Some(ScaleKind::Linear)));

        let bars = fitted(Chart::new(BarController::new(days.clone())));
        assert_eq!(kinds(&bars), (Some(ScaleKind::Time), Some(ScaleKind::Linear)), "dates win over the category scale bars ask for");

        let horizontal = fitted(Chart::new(BarController::new(days.clone()).horizontal(true)));
        assert_eq!(kinds(&horizontal), (Some(ScaleKind::Linear), Some(ScaleKind::Time)), "horizontal bars place their dates on the y scale");

        let numbers = fitted(Chart::new(BarController::new(vec![1.0, 2.0])));
        assert_eq!(kinds(&numbers).0, Some(ScaleKind::Category));

        let explicit = fitted(Chart::new(LineController::new(days)).scale(ScaleId::X, LinearScale::new(Axis::Horizontal)));
        assert!(explicit.scales.get(&ScaleId::X).unwrap().ticks()[0] > 1e9, "a scale set on the chart is kept, and places the dates as numbers");
    }

    /// Text names categories: a default scale with text on it is a category scale, whose
    /// categories are the names in the order the data first names them, across datasets and
    /// controllers, and whose labels are the names. A scale's own labels come first, and names
    /// the data adds follow them.
    #[test]
    fn text_names_categories_in_first_seen_order() {
        use crate::{CategoryScale, ScaleKind};

        let monday = vec![("Mon", 3.0), ("Tue", 5.0), ("Wed", 2.0)];
        let later = vec![("Tue", 1.0), ("Thu", 4.0)];

        let bars = fitted(Chart::new((BarController::new((monday.clone(), later.clone())), LineController::new(vec![("Fri", 6.0)]))));
        assert_eq!(bars.default_kinds.get(&ScaleId::X).copied(), Some(ScaleKind::Category));

        let x = bars.scales.get(&ScaleId::X).unwrap();
        let mut env = Environment::new();
        let names: Vec<_> = x.ticks().iter().map(|tick| x.label(*tick, &mut env)).collect();
        assert_eq!(names, ["Mon", "Tue", "Wed", "Thu", "Fri"].map(|name| Some(name.to_string())), "first seen, across datasets and controllers");
        assert_eq!(x.category_index("Thu"), Some(3.0));

        let after_labels = fitted(
            Chart::new(BarController::new(later))
                .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(["Mon", "Tue"]))
        );
        let x = after_labels.scales.get(&ScaleId::X).unwrap();
        assert_eq!(x.category_index("Tue"), Some(1.0), "a name among the labels is placed on its label");
        assert_eq!(x.category_index("Thu"), Some(2.0), "and new names follow the labels");
        assert_eq!(x.ticks(), &[0.0, 1.0, 2.0]);

        let horizontal = fitted(Chart::new(BarController::new(monday).horizontal(true)));
        assert_eq!(horizontal.default_kinds.get(&ScaleId::Y).copied(), Some(ScaleKind::Category), "horizontal bars name their categories on the y scale");
    }

    /// Text placed on a scale without categories, such as a linear scale set on the chart, isn't
    /// drawn, and a chart of numbers is placed as before.
    #[test]
    fn text_on_a_scale_without_categories_has_no_place() {
        let chart = fitted(Chart::new(BarController::new(vec![("Mon", 3.0)])).scale(ScaleId::X, LinearScale::new(Axis::Horizontal)));
        assert_eq!(chart.scales.get(&ScaleId::X).unwrap().category_index("Mon"), None);
    }

    /// Setting the x or the y scale to a radial scale makes the other default scale radial too,
    /// its kind still picked by the controllers: a line chart's category scale for x.
    #[test]
    fn the_default_scale_follows_a_radial_scale() {
        let radar = fitted(Chart::new(LineController::new(vec![1.0, 2.0, 3.0]))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial)));
        assert_eq!(radar.scales.get(&ScaleId::Y).unwrap().axis(), Axis::Radial);

        let around = fitted(Chart::new(LineController::new(vec![1.0, 2.0, 3.0]))
            .scale(ScaleId::Y, LinearScale::new(Axis::Radial)));
        assert_eq!(around.scales.get(&ScaleId::X).unwrap().axis(), Axis::Radial);
        assert_eq!(around.default_kinds.get(&ScaleId::X).copied(), Some(ScaleKind::Category));

        let flat = fitted(Chart::new(LineController::new(vec![1.0, 2.0, 3.0])));
        assert_eq!((flat.scales.get(&ScaleId::X).unwrap().axis(), flat.scales.get(&ScaleId::Y).unwrap().axis()), (Axis::Horizontal, Axis::Vertical));
    }

    /// A dataset's x scale is the angle and its y scale the radius; `horizontal` swaps the roles,
    /// not the data, so the categories stay on the x scale and run out along the radius.
    #[test]
    fn horizontal_swaps_the_polar_roles() {
        let radar = fitted(Chart::new(LineController::new(vec![1.0, 2.0, 3.0]))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial)));
        assert_eq!((radar.scales.polar_role(&ScaleId::X), radar.scales.polar_role(&ScaleId::Y)), (Some(PolarRole::Angle), Some(PolarRole::Radius)));

        let horizontal = fitted(Chart::new(LineController::new(vec![1.0, 2.0, 3.0]).horizontal(true))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial)));
        assert_eq!((horizontal.scales.polar_role(&ScaleId::X), horizontal.scales.polar_role(&ScaleId::Y)), (Some(PolarRole::Radius), Some(PolarRole::Angle)));
        assert_eq!(horizontal.default_kinds.get(&ScaleId::Y).copied(), Some(ScaleKind::Linear), "the categories stayed on x");
        assert_eq!(scale_range(&horizontal, ScaleId::X), (0.0, 2.0), "the x scale spans the three categories");
    }

    /// A dataset is drawn in polar coordinates on two radial scales of different roles, and on a
    /// side each otherwise; one radial scale with a side scale, or two radial scales of the same
    /// role, belong to no system, so the dataset isn't drawn.
    #[test]
    fn scale_pairs_pick_a_coordinate_system() {
        let horizontal = ScaleId::new("horizontal");
        let chart = fitted(Chart::new((
            LineController::new(vec![1.0, 2.0]),
            LineController::new(vec![1.0, 2.0].x_scale(horizontal.clone())).horizontal(true),
        ))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial))
            .scale(horizontal.clone(), CategoryScale::new(Axis::Radial)));

        let scales = &chart.scales;
        assert_eq!(scales.system(&ScaleId::X, &ScaleId::Y), Some(System::Polar { x_is_angle: true }));
        assert_eq!(scales.system(&horizontal, &ScaleId::Y), None, "the y scale is the radius, and so is the horizontal x scale");
        assert_eq!(scales.polar_role(&horizontal), Some(PolarRole::Radius));

        let cartesian = Scales::new();
        assert_eq!(cartesian.system(&ScaleId::X, &ScaleId::Y), Some(System::Cartesian));
    }

    /// A radial scale bound as the angle by one dataset and as the radius by another keeps the
    /// first role asked for.
    #[test]
    fn a_radial_scale_keeps_the_first_role_asked_for() {
        let chart = fitted(Chart::new((
            LineController::new(vec![1.0, 2.0]),
            LineController::new(vec![1.0, 2.0]).horizontal(true),
        ))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial)));

        assert_eq!(chart.scales.polar_role(&ScaleId::X), Some(PolarRole::Angle));
    }

    /// Line controllers ask for polygon rings and scatter controllers for circles; any circle
    /// wins, and a shape set on the scale wins over both.
    #[test]
    fn grid_shapes_follow_the_controllers_unless_set() {
        let radar = fitted(Chart::new(LineController::new(vec![1.0, 2.0]))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial)));
        assert_eq!(radar.scales.grid_shape(&ScaleId::Y), GridShape::Polygon);

        let both = fitted(Chart::new((LineController::new(vec![1.0, 2.0]), ScatterController::new(vec![(0.0, 1.0)])))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial)));
        assert_eq!(both.scales.grid_shape(&ScaleId::Y), GridShape::Circle);

        let set = fitted(Chart::new(LineController::new(vec![1.0, 2.0]))
            .scale(ScaleId::X, CategoryScale::new(Axis::Radial))
            .scale(ScaleId::Y, LinearScale::new(Axis::Radial).grid_shape(GridShape::Circle)));
        assert_eq!(set.scales.grid_shape(&ScaleId::Y), GridShape::Circle);
    }

    fn sizes(entries: &[(ScaleId, Scalar)]) -> IndexMap<ScaleId, Scalar> {
        entries.iter().cloned().collect()
    }

    /// A value scale that fits to the visible data fits to the values in its index scale's zoom
    /// window, while the zoomed scale keeps fitting to all the data; without the option, the
    /// value scale fits all of it.
    #[test]
    fn value_scales_can_fit_to_the_visible_data() {
        use crate::ZoomWindow;
        let values = vec![1.0, 2.0, 3.0, 50.0, 60.0, 4.0, 5.0, 2.0, 3.0, 1.0];

        // The first three categories, from 0 to 2.
        let zoomed = CategoryScale::new(Axis::Horizontal).zoom(ZoomWindow::new(0.0, 0.2));
        let all = fitted(Chart::new(LineController::new(values.clone())).scale(ScaleId::X, zoomed.clone()));
        assert!(scale_range(&all, ScaleId::Y).1 >= 60.0);

        let visible = fitted(Chart::new(LineController::new(values))
            .scale(ScaleId::X, zoomed)
            .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).fit_visible(true)));
        let (_, y_max) = scale_range(&visible, ScaleId::Y);
        assert!(y_max < 10.0, "the first three values reach 3, so y shouldn't reach the 50 beyond, but ends at {y_max}");
        assert_eq!(scale_range(&visible, ScaleId::X), (0.0, 9.0), "x still fits every point");
    }

    fn fitted<C: ControllerSequence>(mut chart: Chart<C>) -> Chart<C> {
        chart.fit_scales(&mut Environment::new());
        chart
    }

    fn scale_range<C: ControllerSequence>(chart: &Chart<C>, id: ScaleId) -> (Scalar, Scalar) {
        let range = chart.scales.get(&id).expect("the chart should have the scale").range();
        (range.start, range.end)
    }
}
