use carbide::draw::{Angle, Scalar};
use carbide::environment::Environment;
use carbide::state::{AnyReadState, AnyState, IntoReadState, IntoState, ReadStateExtNew, StateContract, StateExtNew};
use crate::scale::{AngleDirection, Axis, AxisPosition, GridShape, PolarRole, ZoomWindow};

/// How a scale's axis is laid out and drawn: the side of the chart area it sits on, its order
/// among the scales on that side, whether it draws grid lines and tick marks, and its title. Every
/// scale type holds one and hands it out with `Scale::axis_options`; the axis code
/// (`scale/axis.rs`) reads it, so it is laid out and drawn the same way for every scale. The
/// scales' builder methods (`LinearScale::position` and so on) set it.
///
/// A polar scale's options say how the circle is drawn: an angle scale's start angle, end angle and
/// direction, and a radius scale's inner radius and ring shape. Every scale keeps them, and only
/// the axis they belong to reads them.
///
/// Every option is a value or a state: the getters read the states as they were last synced
/// (see `sync`), so an axis can change side, order, grid lines, tick marks or title after the
/// chart is built.
#[derive(Clone, Debug)]
pub struct AxisOptions {
    axis: Axis,
    position: Box<dyn AnyReadState<T=AxisPosition>>,
    /// The last position checked against the axis, so a side the axis can't take is logged once.
    checked_position: Option<AxisPosition>,
    weight: Box<dyn AnyReadState<T=i32>>,
    grid_lines: Box<dyn AnyReadState<T=bool>>,
    /// Whether `with_grid_lines` set the grid lines, rather than the default.
    grid_lines_set: bool,
    tick_marks: Box<dyn AnyReadState<T=bool>>,
    title: Option<Box<dyn AnyReadState<T=String>>>,
    start_angle: Box<dyn AnyReadState<T=Angle>>,
    end_angle: Box<dyn AnyReadState<T=Angle>>,
    direction: Box<dyn AnyReadState<T=AngleDirection>>,
    inner_radius: Box<dyn AnyReadState<T=Scalar>>,
    grid_shape: Option<Box<dyn AnyReadState<T=GridShape>>>,
    /// A radial scale's role, as the chart gave it on the last fit.
    polar_role: Option<PolarRole>,
    /// The part of the fitted range the scale shows, when zooming was asked for.
    zoom: Option<Box<dyn AnyState<T=ZoomWindow>>>,
    /// Whether the scale fits to the data visible in the windows of the scales across from it.
    fit_visible: Box<dyn AnyReadState<T=bool>>,
    /// The window the scale shows since its last fit: the zoom asked for, as the scale can show
    /// it (see `ZoomWindow::limited`).
    window: ZoomWindow,
    /// The smallest share of the fitted range the scale shows (see `min_span`).
    min_span: Scalar,
}

impl AxisOptions {
    /// The defaults for a scale along `axis`: on its default side (see `Axis::default_position`),
    /// weight 0, with grid lines and tick marks.
    pub fn new(axis: Axis) -> AxisOptions {
        AxisOptions {
            axis,
            position: constant(axis.default_position()),
            checked_position: None,
            weight: constant(0),
            grid_lines: constant(true),
            grid_lines_set: false,
            tick_marks: constant(true),
            title: None,
            start_angle: constant(Angle::Degrees(0.0)),
            end_angle: constant(Angle::Degrees(360.0)),
            direction: constant(AngleDirection::Clockwise),
            inner_radius: constant(0.0),
            grid_shape: None,
            polar_role: None,
            zoom: None,
            fit_visible: constant(false),
            window: ZoomWindow::WHOLE,
            min_span: crate::scale::MIN_ZOOM_SPAN,
        }
    }

    /// The side of the chart area the axis's ticks, labels and border are drawn on, and where the
    /// chart reserves space for them. A side the axis can't take (see `Axis::allows`) gives its
    /// default side instead.
    pub fn position(&self) -> AxisPosition {
        let position = *self.position.value_dyn();

        if self.axis.allows(position) {
            position
        } else {
            self.axis.default_position()
        }
    }

    /// Orders the axis among those on the same side: a higher weight puts it further away from the
    /// chart area. Axes with the same weight keep the order their scales were added to the chart
    /// in, the first one closest to the area.
    pub fn weight(&self) -> i32 {
        *self.weight.value_dyn()
    }

    /// Whether the axis draws grid lines across the chart area.
    pub fn grid_lines(&self) -> bool {
        *self.grid_lines.value_dyn()
    }

    /// Whether the grid lines were set (see `with_grid_lines`), rather than left to the default. A
    /// controller asking for a scale's decoration to be hidden, as a pie does, leaves a scale whose
    /// grid lines were set to draw it.
    pub fn grid_lines_set(&self) -> bool {
        self.grid_lines_set
    }

    /// Whether the axis draws tick marks in its band, between the chart area and its labels.
    pub fn tick_marks(&self) -> bool {
        *self.tick_marks.value_dyn()
    }

    /// The axis's title, as its state was last synced, or `None` when it has none.
    pub fn title(&self) -> Option<String> {
        self.title.as_ref().map(|title| title.value_dyn().clone())
    }

    /// Where an angle scale starts, clockwise from the top of the circle whichever way it runs.
    /// 0° by default.
    pub fn start_angle(&self) -> Angle {
        *self.start_angle.value_dyn()
    }

    /// Where an angle scale ends: how far it reaches from the start angle, in its direction. 360°
    /// by default, a full circle.
    pub fn end_angle(&self) -> Angle {
        *self.end_angle.value_dyn()
    }

    /// Which way an angle scale runs around the circle. Clockwise by default.
    pub fn direction(&self) -> AngleDirection {
        *self.direction.value_dyn()
    }

    /// Whether an angle scale goes all the way around: its end angle a full turn or more from its
    /// start angle. Categories around a full circle are spaced so the last doesn't land on the
    /// first (see `CategoryScale`), and lines through them close into loops.
    pub fn is_full_circle(&self) -> bool {
        (self.end_angle().degrees() - self.start_angle().degrees()).abs() >= 360.0 - 1e-9
    }

    /// The hole in the middle of a radius scale, as a share of the radius: 0 for none, the
    /// default, and at most 0.95, so there is always a ring to draw in.
    pub fn inner_radius(&self) -> Scalar {
        self.inner_radius.value_dyn().clamp(0.0, 0.95)
    }

    /// The shape of a radius scale's rings when it was set (see `with_grid_shape`); `None` leaves
    /// it to what the controllers ask for.
    pub fn grid_shape(&self) -> Option<GridShape> {
        self.grid_shape.as_ref().map(|shape| *shape.value_dyn())
    }

    /// A radial scale's role on the last fit (see `Scale::set_polar_role`), or `None` before it,
    /// for a scale no dataset binds, and for the other axes.
    pub fn polar_role(&self) -> Option<PolarRole> {
        self.polar_role
    }

    /// The part of the fitted range asked for (see `with_zoom`), as last synced: the whole range
    /// when none was, and always on a radial scale, which isn't zoomed.
    pub fn zoom(&self) -> ZoomWindow {
        match &self.zoom {
            Some(zoom) if self.axis != Axis::Radial => *zoom.value_dyn(),
            _ => ZoomWindow::WHOLE,
        }
    }

    /// Whether the scale was given a zoom (see `with_zoom`), so the chart's gestures pan and zoom
    /// it. Radial scales never are.
    pub fn is_zoomable(&self) -> bool {
        self.zoom.is_some() && self.axis != Axis::Radial
    }

    /// Sets the zoom window asked for, writing it to the zoom state, as the chart's gestures do.
    /// Does nothing on a scale without a zoom.
    pub fn set_zoom(&mut self, window: ZoomWindow) {
        if let Some(zoom) = &mut self.zoom {
            zoom.set_value_dyn(window);
        }
    }

    /// The smallest share of the fitted range the scale shows, as it decided on its last fit:
    /// two categories on a category scale, a hundredth of the range on the others.
    pub fn min_span(&self) -> Scalar {
        self.min_span
    }

    /// Keeps the smallest share of the fitted range the scale shows (see `min_span`).
    pub fn set_min_span(&mut self, min_span: Scalar) {
        self.min_span = min_span;
    }

    /// The part of the fitted range the scale shows since its last fit: the zoom asked for, kept
    /// within the range and wide enough, and on a category scale snapped to whole categories.
    pub fn window(&self) -> ZoomWindow {
        self.window
    }

    /// Keeps the window the scale shows, which the scale decides on every fit.
    pub fn set_window(&mut self, window: ZoomWindow) {
        self.window = window;
    }

    /// Whether the scale fits to the data inside the windows of the scales its data is placed
    /// along, rather than to all of it (see `with_fit_visible`).
    pub fn fit_visible(&self) -> bool {
        *self.fit_visible.value_dyn()
    }

    /// Keeps a radial scale's role for the fit (see `Scale::set_polar_role`).
    pub fn set_polar_role(&mut self, role: Option<PolarRole>) {
        self.polar_role = role;
    }

    /// Syncs the states in the options with the environment, and returns whether anything may
    /// have changed. Scales call it from `Scale::sync`. A position the axis can't take is logged
    /// the first time it is seen, and the axis is drawn on its default side meanwhile.
    pub fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = self.position.sync(env);
        changed |= self.weight.sync(env);
        changed |= self.grid_lines.sync(env);
        changed |= self.tick_marks.sync(env);
        changed |= self.title.as_mut().is_some_and(|title| title.sync(env));
        changed |= self.start_angle.sync(env);
        changed |= self.end_angle.sync(env);
        changed |= self.direction.sync(env);
        changed |= self.inner_radius.sync(env);
        changed |= self.grid_shape.as_mut().is_some_and(|shape| shape.sync(env));
        changed |= self.zoom.as_mut().is_some_and(|zoom| zoom.sync(env));
        changed |= self.fit_visible.sync(env);

        let position = *self.position.value_dyn();
        if self.checked_position != Some(position) {
            self.checked_position = Some(position);

            if !self.axis.allows(position) {
                log::warn!("A {:?} scale can't be positioned {:?}; drawing it {:?}", self.axis, position, self.axis.default_position());
            }
        }

        changed
    }

    /// Puts the axis on `position`, a side or a state of one. Horizontal axes go above or below
    /// the chart area, vertical axes left or right of it; any other side is logged and the axis is
    /// drawn on its default side.
    pub fn with_position(mut self, position: impl IntoReadState<AxisPosition>) -> AxisOptions {
        self.position = position.into_read_state().as_dyn_read();
        self.checked_position = None;
        self
    }

    /// Sets the weight, a value or a state of one (see `weight`).
    pub fn with_weight(mut self, weight: impl IntoReadState<i32>) -> AxisOptions {
        self.weight = weight.into_read_state().as_dyn_read();
        self
    }

    /// Sets whether the axis draws grid lines, a value or a state of one.
    pub fn with_grid_lines(mut self, grid_lines: impl IntoReadState<bool>) -> AxisOptions {
        self.grid_lines = grid_lines.into_read_state().as_dyn_read();
        self.grid_lines_set = true;
        self
    }

    /// Sets whether the axis draws tick marks, a value or a state of one.
    pub fn with_tick_marks(mut self, tick_marks: impl IntoReadState<bool>) -> AxisOptions {
        self.tick_marks = tick_marks.into_read_state().as_dyn_read();
        self
    }

    /// Gives the axis a title, a text or a state of one.
    pub fn with_title(mut self, title: impl IntoReadState<String>) -> AxisOptions {
        self.title = Some(title.into_read_state().as_dyn_read());
        self
    }

    /// Sets where an angle scale starts, an `Angle` or a state of one.
    pub fn with_start_angle(mut self, angle: impl IntoReadState<Angle>) -> AxisOptions {
        self.start_angle = angle.into_read_state().as_dyn_read();
        self
    }

    /// Sets where an angle scale ends, an `Angle` or a state of one.
    pub fn with_end_angle(mut self, angle: impl IntoReadState<Angle>) -> AxisOptions {
        self.end_angle = angle.into_read_state().as_dyn_read();
        self
    }

    /// Sets which way an angle scale runs, a direction or a state of one.
    pub fn with_direction(mut self, direction: impl IntoReadState<AngleDirection>) -> AxisOptions {
        self.direction = direction.into_read_state().as_dyn_read();
        self
    }

    /// Sets a radius scale's hole in the middle, as a share of the radius, a number or a state
    /// of one (see `inner_radius`).
    pub fn with_inner_radius(mut self, ratio: impl IntoReadState<Scalar>) -> AxisOptions {
        self.inner_radius = ratio.into_read_state().as_dyn_read();
        self
    }

    /// Sets the shape of a radius scale's rings, a shape or a state of one, whatever the
    /// controllers ask for.
    pub fn with_grid_shape(mut self, shape: impl IntoReadState<GridShape>) -> AxisOptions {
        self.grid_shape = Some(shape.into_read_state().as_dyn_read());
        self
    }

    /// Shows only part of the fitted range, a `ZoomWindow` or a state of one (see `zoom`).
    pub fn with_zoom(mut self, window: impl IntoState<ZoomWindow>) -> AxisOptions {
        self.zoom = Some(window.into_state().as_dyn());
        self
    }

    /// Sets whether the scale fits to the visible data only, a value or a state of one.
    pub fn with_fit_visible(mut self, fit_visible: impl IntoReadState<bool>) -> AxisOptions {
        self.fit_visible = fit_visible.into_read_state().as_dyn_read();
        self
    }
}

/// The builder methods for zooming, in the impl block of a scale type with an `axis_options`
/// field: `zoom` and `fit_visible`.
macro_rules! zoom_options {
    ($scale:ty) => {
        /// Shows only part of the scale's fitted range: a `ZoomWindow`, or a state of one, which
        /// the app can read, set, animate, or share with another chart's scale. The scale keeps
        /// fitting to its data, and everything drawn on it is mapped through the window, clipped
        /// to the chart area, with ticks for the part shown. A window is kept within the fitted
        /// range and wide enough (see `ZoomWindow`); setting it back to `ZoomWindow::WHOLE`
        /// resets it. Radial scales aren't zoomed.
        pub fn zoom(mut self, window: impl carbide::state::IntoState<crate::scale::ZoomWindow>) -> $scale {
            self.axis_options = self.axis_options.with_zoom(window);
            self
        }

        /// Fits the scale to the data inside the zoom windows of the scales its data is placed
        /// along, rather than to all of it: a value scale whose index scale is zoomed fits the
        /// values shown. A value or a state of one; `false` by default.
        pub fn fit_visible(mut self, fit_visible: impl carbide::state::IntoReadState<bool>) -> $scale {
            self.axis_options = self.axis_options.with_fit_visible(fit_visible);
            self
        }
    };
}

pub(crate) use zoom_options;

/// The builder methods for a polar scale's options, in the impl block of a scale type with an
/// `axis_options` field: `start_angle`, `end_angle` and `direction` for angle scales, and
/// `inner_radius` and `grid_shape` for radius scales.
macro_rules! polar_options {
    ($scale:ty) => {
        /// On an angle scale, where the circle starts, clockwise from the top whichever way the
        /// scale runs, so the first category stays put when the direction changes: an `Angle`, or
        /// a state of one. 0° by default.
        pub fn start_angle(mut self, angle: impl carbide::state::IntoReadState<carbide::draw::Angle>) -> $scale {
            self.axis_options = self.axis_options.with_start_angle(angle);
            self
        }

        /// On an angle scale, where the circle ends, as far from the start angle as it says, in
        /// the scale's direction: an `Angle`, or a state of one. 360° by default, a full circle;
        /// less draws part of one.
        pub fn end_angle(mut self, angle: impl carbide::state::IntoReadState<carbide::draw::Angle>) -> $scale {
            self.axis_options = self.axis_options.with_end_angle(angle);
            self
        }

        /// On an angle scale, which way it runs around the circle, a direction or a state of
        /// one. Clockwise by default.
        pub fn direction(mut self, direction: impl carbide::state::IntoReadState<crate::scale::AngleDirection>) -> $scale {
            self.axis_options = self.axis_options.with_direction(direction);
            self
        }

        /// On a radius scale, the hole in the middle, as a share of the radius, a number or a
        /// state of one. 0 by default.
        pub fn inner_radius(mut self, ratio: impl carbide::state::IntoReadState<carbide::draw::Scalar>) -> $scale {
            self.axis_options = self.axis_options.with_inner_radius(ratio);
            self
        }

        /// On a radius scale, the shape of its rings, a shape or a state of one. When not set,
        /// line controllers ask for polygons and the others for circles, and any circle wins.
        pub fn grid_shape(mut self, shape: impl carbide::state::IntoReadState<crate::scale::GridShape>) -> $scale {
            self.axis_options = self.axis_options.with_grid_shape(shape);
            self
        }
    };
}

pub(crate) use polar_options;

/// A state that always holds `value`.
fn constant<T: StateContract + IntoReadState<T>>(value: T) -> Box<dyn AnyReadState<T=T>> {
    IntoReadState::<T>::into_read_state(value).as_dyn_read()
}

#[cfg(test)]
mod tests {
    use carbide::environment::Environment;
    use carbide::state::{LocalState, State};
    use crate::scale::{Axis, AxisPosition};
    use super::AxisOptions;

    /// Every option is a value or a state, read as it was last synced.
    #[test]
    fn options_are_states() {
        let mut position = LocalState::new(AxisPosition::Left);
        let mut weight = LocalState::new(0);
        let mut grid_lines = LocalState::new(true);
        let mut tick_marks = LocalState::new(true);

        let mut options = AxisOptions::new(Axis::Vertical)
            .with_position(position.clone())
            .with_weight(weight.clone())
            .with_grid_lines(grid_lines.clone())
            .with_tick_marks(tick_marks.clone());
        options.sync(&mut Environment::new());
        assert_eq!((options.position(), options.weight(), options.grid_lines(), options.tick_marks()), (AxisPosition::Left, 0, true, true));

        *position.value_mut() = AxisPosition::Right;
        *weight.value_mut() = 2;
        *grid_lines.value_mut() = false;
        *tick_marks.value_mut() = false;
        options.sync(&mut Environment::new());
        assert_eq!((options.position(), options.weight(), options.grid_lines(), options.tick_marks()), (AxisPosition::Right, 2, false, false));
    }

    /// A side the axis can't take, set as a value or reached by a state, draws the axis on its
    /// default side, and taking a side it can returns it there.
    #[test]
    fn a_side_the_axis_cant_take_gives_its_default_side() {
        let mut position = LocalState::new(AxisPosition::Right);
        let mut options = AxisOptions::new(Axis::Vertical).with_position(position.clone());
        options.sync(&mut Environment::new());
        assert_eq!(options.position(), AxisPosition::Right);

        *position.value_mut() = AxisPosition::Top;
        options.sync(&mut Environment::new());
        assert_eq!(options.position(), AxisPosition::Left, "a vertical axis can't sit on top");

        *position.value_mut() = AxisPosition::Right;
        options.sync(&mut Environment::new());
        assert_eq!(options.position(), AxisPosition::Right);

        assert_eq!(AxisOptions::new(Axis::Horizontal).with_position(AxisPosition::Left).position(), AxisPosition::Bottom);
    }
}
