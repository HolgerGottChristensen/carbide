use carbide::event::MouseEventContext;
use carbide::layout::LayoutContext;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseEvent, MouseEventHandler};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::layout::Layout;
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map3, Map4, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide_core::widget::{CommonWidget, Empty, Rectangle, AnyWidget, WidgetExt, WidgetId, Widget};
use crate::{enabled_state, EnabledState};

const SMOOTH_VALUE_INCREMENT: f64 = 0.05;
const SMOOTH_VALUE_SMALL_INCREMENT: f64 = 0.01;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable, Layout, MouseEvent, KeyboardEvent, Render)]
pub struct PlainSlider<F, St, S, E, P, Th, In, Bg, En> where
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: AnyWidget + Clone,
    In: AnyWidget + Clone,
    Bg: AnyWidget + Clone,
    En: ReadState<T=bool>,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] focus: F,
    #[state] enabled: En,
    dragging: bool,

    #[state] state: St,
    #[state] percent: Box<dyn AnyState<T=f64>>,
    #[state] start: S,
    #[state] end: E,
    #[state] steps: P,

    thumb_delegate: Delegate<Th>,
    thumb: Th,
    track_delegate: Delegate<In>,
    track: In,
    background_delegate: Delegate<Bg>,
    background: Bg,
}

impl PlainSlider<Focus, f64, f64, f64, Option<f64>, Empty, Empty, Empty, bool> {
    pub fn new<St: IntoState<f64>, S: IntoReadState<f64>, E: IntoReadState<f64>>(state: St, start: S, end: E) -> PlainSlider<LocalState<Focus>, St::Output, S::Output, E::Output, Option<f64>, Box<dyn AnyWidget>, Box<dyn AnyWidget>, Box<dyn AnyWidget>, EnabledState> {
        let focus = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            focus,
            state.into_state(),
            start.into_read_state(),
            end.into_read_state(),
            None,
            default_thumb,
            default_track,
            default_background,
            enabled_state()
        )
    }
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: AnyWidget + Clone,
    In: AnyWidget + Clone,
    Bg: AnyWidget + Clone,
    En: ReadState<T=bool>,
> PlainSlider<F, St, S, E, P, Th, In, Bg, En> {
    pub fn step<P2: IntoReadState<Option<f64>>>(self, steps: P2) -> PlainSlider<F, St, S, E, P2::Output, Th, In, Bg, En> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            steps.into_read_state(),
            self.thumb_delegate,
            self.track_delegate,
            self.background_delegate,
            self.enabled,
        )
    }

    pub fn focused<F2: IntoState<Focus>>(self, focus: F2) -> PlainSlider<F2::Output, St, S, E, P, Th, In, Bg, En> {
        Self::new_internal(
            focus.into_state(),
            self.state,
            self.start,
            self.end,
            self.steps,
            self.thumb_delegate,
            self.track_delegate,
            self.background_delegate,
            self.enabled,
        )
    }

    pub fn enabled<En2: IntoReadState<bool>>(self, enabled: En2) -> PlainSlider<F, St, S, E, P, Th, In, Bg, En2::Output> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            self.thumb_delegate,
            self.track_delegate,
            self.background_delegate,
            enabled.into_read_state(),
        )
    }

    pub fn background<Bg2: AnyWidget + Clone>(self, delegate: Delegate<Bg2>) -> PlainSlider<F, St, S, E, P, Th, In, Bg2, En> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            self.thumb_delegate,
            self.track_delegate,
            delegate,
            self.enabled,
        )
    }

    pub fn thumb<Th2: AnyWidget + Clone>(self, delegate: Delegate<Th2>) -> PlainSlider<F, St, S, E, P, Th2, In, Bg, En> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            delegate,
            self.track_delegate,
            self.background_delegate,
            self.enabled,
        )
    }

    pub fn track<In2: AnyWidget + Clone>(self, delegate: Delegate<In2>) -> PlainSlider<F, St, S, E, P, Th, In2, Bg, En> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            self.thumb_delegate,
            delegate,
            self.background_delegate,
            self.enabled,
        )
    }

    fn percent_to_stepped_percent(percent: f64, start: f64, end: f64, step_size: f64) -> f64 {
        let range = end - start;
        let range_mod = range % step_size;
        let percent_lost = range_mod / range;
        let number_of_steps = range / step_size;
        let percent_per_step = (number_of_steps * percent).round() / number_of_steps;

        if percent > 1.0 - percent_lost / 2.0 {
            1.0
        } else {
            percent_per_step
        }
    }

    fn new_internal<
        F2: State<T=Focus>,
        St2: State<T=f64>,
        S2: ReadState<T=f64>,
        E2: ReadState<T=f64>,
        P2: ReadState<T=Option<f64>>,
        Th2: AnyWidget + Clone,
        In2: AnyWidget + Clone,
        Bg2: AnyWidget + Clone,
        En2: ReadState<T=bool>,
    >(focus: F2, state: St2, start: S2, end: E2, steps: P2, thumb_delegate: Delegate<Th2>, track_delegate: Delegate<In2>, background_delegate: Delegate<Bg2>, enabled: En2) -> PlainSlider<F2, St2, S2, E2, P2, Th2, In2, Bg2, En2> {
        let percent = Map4::map(
            state.clone(),
            start.ignore_writes(),
            end.ignore_writes(),
            steps.ignore_writes(),
            |state: &f64, start: &f64, end: &f64, _steps: &Option<f64>| {
                (*state - *start) / (*end - *start)
            },
            |new_percent: f64, _state: &f64, start: &f64, end: &f64, steps: &Option<f64>| {
                if let Some(step_size) = *steps {
                    let stepped_percent = Self::percent_to_stepped_percent(new_percent, *start, *end, step_size);
                    let stepped_value = stepped_percent * (*end - *start) + *start;

                    (
                        Some(stepped_value),
                        None,
                        None,
                        None,
                    )
                } else {
                    (Some(new_percent * (*end - *start) + *start), None, None, None)
                }
            }
        ).as_dyn();

        let thumb = thumb_delegate(state.as_dyn(), start.as_dyn_read(), end.as_dyn_read(), steps.as_dyn_read(), focus.as_dyn_read(), enabled.as_dyn_read());
        let track = track_delegate(state.as_dyn(), start.as_dyn_read(), end.as_dyn_read(), steps.as_dyn_read(), focus.as_dyn_read(), enabled.as_dyn_read());
        let background = background_delegate(state.as_dyn(), start.as_dyn_read(), end.as_dyn_read(), steps.as_dyn_read(), focus.as_dyn_read(), enabled.as_dyn_read());

        PlainSlider {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            focus,
            enabled,
            dragging: false,
            state,
            percent,
            start,
            end,
            steps,
            thumb_delegate,
            thumb,
            track_delegate,
            track,
            background_delegate,
            background,
        }
    }
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: AnyWidget + Clone,
    In: AnyWidget + Clone,
    Bg: AnyWidget + Clone,
    En: ReadState<T=bool>,
> Focusable for PlainSlider<F, St, S, E, P, Th, In, Bg, En> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: AnyWidget + Clone,
    In: AnyWidget + Clone,
    Bg: AnyWidget + Clone,
    En: ReadState<T=bool>,
> KeyboardEventHandler for PlainSlider<F, St, S, E, P, Th, In, Bg, En> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, _env: &mut Environment) {
        if !*self.enabled.value() {
            return;
        }

        if *self.focus.value() == Focus::Focused && self.steps.value().is_none() {
            let value = *self.percent.value();

            match event {
                KeyboardEvent::Press(Key::ArrowRight, ModifierKey::CTRL) => {
                    self.percent.set_value(1.0);
                }
                KeyboardEvent::Press(Key::ArrowLeft, ModifierKey::CTRL) => {
                    self.percent.set_value(0.0);
                }
                KeyboardEvent::Press(Key::ArrowRight, ModifierKey::SHIFT) => {
                    self.percent.set_value((value + SMOOTH_VALUE_SMALL_INCREMENT).min(1.0));
                }
                KeyboardEvent::Press(Key::ArrowLeft, ModifierKey::SHIFT) => {
                    self.percent.set_value((value - SMOOTH_VALUE_SMALL_INCREMENT).max(0.0));
                }
                KeyboardEvent::Press(Key::ArrowRight, _) => {
                    self.percent.set_value((value + SMOOTH_VALUE_INCREMENT).min(1.0));
                }
                KeyboardEvent::Press(Key::ArrowLeft, _) => {
                    self.percent.set_value((value - SMOOTH_VALUE_INCREMENT).max(0.0));
                }
                _ => ()
            }
        }
    }
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: AnyWidget + Clone,
    In: AnyWidget + Clone,
    Bg: AnyWidget + Clone,
    En: ReadState<T=bool>,
> MouseEventHandler for PlainSlider<F, St, S, E, P, Th, In, Bg, En> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, ctx: &mut MouseEventContext) {
        if !*self.enabled.value() {
            return;
        }

        match event {
            MouseEvent::Press(_, position, _) => {
                if self.thumb.is_inside(*position) || self.background.is_inside(*position) {
                    if *self.focus.value() != Focus::Focused {
                        self.focus.set_value(Focus::FocusRequested);
                        ctx.env.request_focus(Refocus::FocusRequest);
                    }

                    self.dragging = true;

                    let relative_to_position = *position - self.position;
                    let p = (relative_to_position.x() - self.thumb.width() / 2.0)
                        / (self.dimension.width - self.thumb.width());
                    self.percent.set_value(p.max(0.0).min(1.0));
                } else {
                    if *self.focus.value() == Focus::Focused {
                        self.focus.set_value(Focus::FocusReleased);
                        ctx.env.request_focus(Refocus::FocusRequest);
                    }
                }
            }
            MouseEvent::Release(_, _, _) => {
                self.dragging = false;
            }
            MouseEvent::Move { to, .. } => {
                if !self.dragging {
                    return;
                }

                let relative_to_position = *to - self.position;
                let p = (relative_to_position.x() - self.thumb.width() / 2.0)
                    / (self.dimension.width - self.thumb.width());
                self.percent.set_value(p.max(0.0).min(1.0));
            }
            _ => (),
        }
    }
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: AnyWidget + Clone,
    In: AnyWidget + Clone,
    Bg: AnyWidget + Clone,
    En: ReadState<T=bool>,
> Layout for PlainSlider<F, St, S, E, P, Th, In, Bg, En> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let percent = self.percent.value().max(0.0).min(1.0);

        let background = self.background.calculate_size(requested_size, ctx);

        let track_width = if let Some(steps) = *self.steps.value() {
            let stepped_percent = Self::percent_to_stepped_percent(
                percent,
                *self.start.value(),
                *self.end.value(),
                steps
            ).max(0.0).min(1.0);

            requested_size.width * stepped_percent
        } else {
            requested_size.width * percent
        };

        let track_dimensions = Dimension::new(track_width, requested_size.height);
        let track = self.track.calculate_size(track_dimensions, ctx);

        let thumb = self.thumb.calculate_size(requested_size, ctx);

        let max_height = background.height.max(track.height).max(thumb.height);

        self.set_dimension(Dimension::new(requested_size.width, max_height));
        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let percent = self.percent.value().max(0.0).min(1.0);
        let position = self.position();

        let background_y = position.y() + self.height() / 2.0 - self.background.height() / 2.0;
        let track_y = position.y() + self.height() / 2.0 - self.track.height() / 2.0;
        let thumb_y = position.y() + self.height() / 2.0 - self.thumb.height() / 2.0;

        let thumb_x = if let Some(steps) = *self.steps.value() {
            let stepped_percent = Self::percent_to_stepped_percent(
                percent,
                *self.start.value(),
                *self.end.value(),
                steps
            ).max(0.0).min(1.0);

            self.x() + (self.background.width() - self.thumb.width()) * stepped_percent
        } else {
            self.x() + (self.background.width() - self.thumb.width()) * percent
        };

        self.background.set_position(Position::new(position.x(), background_y));
        self.track.set_position(Position::new(position.x(), track_y));
        self.thumb.set_position(Position::new(thumb_x, thumb_y));

        self.background.position_children(ctx);
        self.track.position_children(ctx);
        self.thumb.position_children(ctx);
    }
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: AnyWidget + Clone,
    In: AnyWidget + Clone,
    Bg: AnyWidget + Clone,
    En: ReadState<T=bool>,
> CommonWidget for PlainSlider<F, St, S, E, P, Th, In, Bg, En> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 1, focus: self.focus);
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: AnyWidget + Clone,
    In: AnyWidget + Clone,
    Bg: AnyWidget + Clone,
    En: ReadState<T=bool>,
> Render for PlainSlider<F, St, S, E, P, Th, In, Bg, En> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.background.render(context, env);
        self.track.render(context, env);
        self.thumb.render(context, env);
    }
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: AnyWidget + Clone,
    In: AnyWidget + Clone,
    Bg: AnyWidget + Clone,
    En: ReadState<T=bool>,
> WidgetExt for PlainSlider<F, St, S, E, P, Th, In, Bg, En> {}


// ---------------------------------------------------
//  Delegates
// ---------------------------------------------------
type Delegate<W> = fn(
    state: Box<dyn AnyState<T=f64>>,
    start: Box<dyn AnyReadState<T=f64>>,
    end: Box<dyn AnyReadState<T=f64>>,
    steps: Box<dyn AnyReadState<T=Option<f64>>>,
    focus: Box<dyn AnyReadState<T=Focus>>,
    enabled: Box<dyn AnyReadState<T=bool>>,
) -> W;

fn default_background(_state: Box<dyn AnyState<T=f64>>, _start: Box<dyn AnyReadState<T=f64>>, _end: Box<dyn AnyReadState<T=f64>>, _steps: Box<dyn AnyReadState<T=Option<f64>>>, _focus: Box<dyn AnyReadState<T=Focus>>, _enabled: Box<dyn AnyReadState<T=bool>>,) -> Box<dyn AnyWidget> {
    Rectangle::new()
        .fill(EnvironmentColor::Red)
        .frame_fixed_height(26.0)
        .boxed()
}

fn default_track(_state: Box<dyn AnyState<T=f64>>, _start: Box<dyn AnyReadState<T=f64>>, _end: Box<dyn AnyReadState<T=f64>>, _steps: Box<dyn AnyReadState<T=Option<f64>>>, _focus: Box<dyn AnyReadState<T=Focus>>, _enabled: Box<dyn AnyReadState<T=bool>>,) -> Box<dyn AnyWidget> {
    Rectangle::new()
        .fill(EnvironmentColor::Green)
        .frame_fixed_height(26.0)
        .boxed()
}

fn default_thumb(state: Box<dyn AnyState<T=f64>>, start: Box<dyn AnyReadState<T=f64>>, end: Box<dyn AnyReadState<T=f64>>, _steps: Box<dyn AnyReadState<T=Option<f64>>>, _focus: Box<dyn AnyReadState<T=Focus>>, _enabled: Box<dyn AnyReadState<T=bool>>,) -> Box<dyn AnyWidget> {
    let color = Map3::read_map(state, start, end, |state, start, end| {
        if state < start || state > end {
            EnvironmentColor::Purple
        } else {
            EnvironmentColor::Blue
        }
    });

    Rectangle::new().fill(color).frame(26.0, 26.0).boxed()
}