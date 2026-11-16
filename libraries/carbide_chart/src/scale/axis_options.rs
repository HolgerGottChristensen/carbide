use carbide::environment::Environment;
use carbide::state::{AnyReadState, IntoReadState, ReadStateExtNew, StateContract};
use crate::scale::{Axis, AxisPosition};

/// How a scale's axis is laid out and drawn: the side of the chart area it sits on, its order
/// among the scales on that side, whether it draws grid lines and tick marks, and its title. Every
/// scale type holds one and hands it out with `Scale::axis_options`; the axis code
/// (`scale/axis.rs`) reads it, so it is laid out and drawn the same way for every scale. The
/// scales' builder methods (`LinearScale::position` and so on) set it.
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
    tick_marks: Box<dyn AnyReadState<T=bool>>,
    title: Option<Box<dyn AnyReadState<T=String>>>,
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
            tick_marks: constant(true),
            title: None,
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

    /// Whether the axis draws tick marks in its band, between the chart area and its labels.
    pub fn tick_marks(&self) -> bool {
        *self.tick_marks.value_dyn()
    }

    /// The axis's title, as its state was last synced, or `None` when it has none.
    pub fn title(&self) -> Option<String> {
        self.title.as_ref().map(|title| title.value_dyn().clone())
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
}

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
