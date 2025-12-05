use crate::slider::{SliderStepping, SliderSteppingType, SliderStyle};
use crate::slider::SliderValue;
use crate::slider::SliderStyleKey;
use crate::{EnabledState};
use carbide_core::draw::{AutomaticStyle, Dimension, Position};
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, ModifierKey, MouseEvent, MouseEventContext, MouseEventHandler};
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::{Focus, FocusManager, Refocus};
use carbide_core::layout::{Layout, LayoutContext};
use carbide_core::lifecycle::{InitializationContext, Initialize};
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{AnyState, IntoReadState, IntoState, LocalState, Map1, Map4, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide_core::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId};
use carbide_core::CommonWidgetImpl;

const SMOOTH_VALUE_INCREMENT: f64 = 0.05;
const SMOOTH_VALUE_SMALL_INCREMENT: f64 = 0.01;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout, MouseEvent, KeyboardEvent, Render, Initialize)]
pub struct Slider<Value, F, T, Start, End, StepMode, Enabled> where
    Value: SliderValue,
    F: State<T=Focus>,
    T: State<T=Value>,
    Start: ReadState<T=Value>,
    End: ReadState<T=Value>,
    StepMode: ReadState<T=SliderStepping<Value>>,
    Enabled: ReadState<T=bool>,
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] focus: F,
    #[state] enabled: Enabled,
    dragging: bool,

    #[state] state: T,
    #[state] percent: Box<dyn AnyState<T=f64>>,
    #[state] start: Start,
    #[state] end: End,
    #[state] steps: StepMode,

    thumb: Box<dyn AnyWidget>,
    track: Box<dyn AnyWidget>,
    background: Box<dyn AnyWidget>,
}

impl Slider<f64, Focus, f64, f64, f64, SliderStepping<f64>, bool> {
    pub fn new<Value: SliderValue, T: State<T=Value>, Start: IntoReadState<Value>, End: IntoReadState<Value>>(state: T, start: Start, end: End) -> Slider<Value, LocalState<Focus>, T, Start::Output, End::Output, SliderStepping<Value>, EnabledState> {
        let focus = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            focus,
            state,
            start.into_read_state(),
            end.into_read_state(),
            SliderStepping::Smooth,
            EnabledState::new(true)
        )
    }
}

impl<
    Value: SliderValue,
    F: State<T=Focus>,
    T: State<T=Value>,
    Start: ReadState<T=Value>,
    End: ReadState<T=Value>,
    Stepped: ReadState<T=SliderStepping<Value>>,
    Enabled: ReadState<T=bool>,
> Slider<Value, F, T, Start, End, Stepped, Enabled> {
    pub fn step<Stepped2: IntoReadState<SliderStepping<Value>>>(self, step_size: Stepped2) -> Slider<Value, F, T, Start, End, Stepped2::Output, Enabled> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            step_size.into_read_state(),
            self.enabled,
        )
    }

    pub fn focused<F2: IntoState<Focus>>(self, focus: F2) -> Slider<Value, F2::Output, T, Start, End, Stepped, Enabled> {
        Self::new_internal(
            focus.into_state(),
            self.state,
            self.start,
            self.end,
            self.steps,
            self.enabled,
        )
    }

    pub fn enabled<Enabled2: IntoReadState<bool>>(self, enabled: Enabled2) -> Slider<Value, F, T, Start, End, Stepped, Enabled2::Output> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            enabled.into_read_state(),
        )
    }

    fn new_internal<
        V2: SliderValue,
        F2: State<T=Focus>,
        St2: State<T=V2>,
        S2: ReadState<T=V2>,
        E2: ReadState<T=V2>,
        P2: ReadState<T=SliderStepping<V2>>,
        En2: ReadState<T=bool>,
    >(focus: F2, state: St2, start: S2, end: E2, steps: P2, enabled: En2) -> Slider<V2, F2, St2, S2, E2, P2, En2> {
        let percent = Map4::map(
            state.clone(),
            start.ignore_writes(),
            end.ignore_writes(),
            steps.ignore_writes(),
            |state: &V2, start: &V2, end: &V2, _steps: &SliderStepping<V2>| {
                V2::value_to_percent(state, start, end)
            },
            |new_percent: f64, mut state, start, end, steps| {
                match &*steps {
                    SliderStepping::Smooth => {
                        *state = V2::percent_to_value(&*start, &*end, new_percent);
                    }
                    SliderStepping::Stepped(steps) | SliderStepping::SmoothStepped(steps) => {
                        *state = V2::percent_to_stepped_value(&*start, &*end, new_percent, steps);
                    }
                }
            }
        ).as_dyn();

        let stepping_type = Map1::read_map(steps.clone(), |a| {
            match *a {
                SliderStepping::Smooth => SliderSteppingType::Smooth,
                SliderStepping::Stepped(_) => SliderSteppingType::Stepped,
                SliderStepping::SmoothStepped(_) => SliderSteppingType::SmoothStepped,
            }
        });

        let style = AutomaticStyle;

        let thumb = style.create_thumb(focus.as_dyn_read(), enabled.as_dyn_read(), percent.as_dyn_read(), stepping_type.as_dyn_read());
        let track = style.create_track(focus.as_dyn_read(), enabled.as_dyn_read(), percent.as_dyn_read(), stepping_type.as_dyn_read());
        let background = style.create_background(focus.as_dyn_read(), enabled.as_dyn_read(), percent.as_dyn_read(), stepping_type.as_dyn_read());

        Slider {
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
            thumb,
            track,
            background,
        }
    }
}

impl<
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=SliderStepping<V>>,
    En: ReadState<T=bool>,
> KeyboardEventHandler for Slider<V, F, St, S, E, P, En> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, _ctx: &mut KeyboardEventContext) {
        if !*self.enabled.value() {
            return;
        }

        if *self.focus.value() == Focus::Focused {
            let value = *self.percent.value();

            match event {
                KeyboardEvent::Press { key: Key::ArrowRight, modifiers: ModifierKey::CONTROL, .. } => {
                    self.percent.set_value(1.0);
                }
                KeyboardEvent::Press { key: Key::ArrowLeft, modifiers: ModifierKey::CONTROL, .. } => {
                    self.percent.set_value(0.0);
                }
                KeyboardEvent::Press { key: Key::ArrowRight, modifiers: ModifierKey::SHIFT, .. } => {
                    self.percent.set_value((value + SMOOTH_VALUE_SMALL_INCREMENT).min(1.0));
                }
                KeyboardEvent::Press { key: Key::ArrowLeft, modifiers: ModifierKey::SHIFT, .. } => {
                    self.percent.set_value((value - SMOOTH_VALUE_SMALL_INCREMENT).max(0.0));
                }
                KeyboardEvent::Press { key: Key::ArrowRight, modifiers: _, .. } => {
                    self.percent.set_value((value + SMOOTH_VALUE_INCREMENT).min(1.0));
                }
                KeyboardEvent::Press { key: Key::ArrowLeft, modifiers: _, .. } => {
                    self.percent.set_value((value - SMOOTH_VALUE_INCREMENT).max(0.0));
                }
                _ => ()
            }
        }
    }
}

impl<
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=SliderStepping<V>>,
    En: ReadState<T=bool>,
> MouseEventHandler for Slider<V, F, St, S, E, P, En> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !*self.enabled.value() {
            return;
        }

        match event {
            MouseEvent::Press { position, .. } => {
                if self.thumb.is_inside(*position) || self.background.is_inside(*position) {
                    if *self.focus.value() != Focus::Focused {
                        self.focus.set_value(Focus::FocusRequested);
                        FocusManager::get(ctx.env, |manager| {
                            manager.request_focus(Refocus::FocusRequest)
                        });
                    }

                    self.dragging = true;

                    let relative_to_position = *position - self.position;
                    let p = (relative_to_position.x - self.thumb.width() / 2.0)
                        / (self.dimension.width - self.thumb.width());
                    self.percent.set_value(p.max(0.0).min(1.0));
                } else {
                    if *self.focus.value() == Focus::Focused {
                        self.focus.set_value(Focus::FocusReleased);
                        FocusManager::get(ctx.env, |manager| {
                            manager.request_focus(Refocus::FocusRequest)
                        });
                    }
                }
            }
            MouseEvent::Release { .. } => {
                self.dragging = false;
            }
            MouseEvent::Move { to, .. } => {
                if !self.dragging {
                    return;
                }

                let relative_to_position = *to - self.position;
                let p = (relative_to_position.x - self.thumb.width() / 2.0)
                    / (self.dimension.width - self.thumb.width());
                self.percent.set_value(p.max(0.0).min(1.0));
            }
            _ => (),
        }
    }
}

impl<
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=SliderStepping<V>>,
    En: ReadState<T=bool>,
> Layout for Slider<V, F, St, S, E, P, En> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let percent = self.percent.value().max(0.0).min(1.0);

        let background = self.background.calculate_size(requested_size, ctx);

        let track_width = match &*self.steps.value() {
            SliderStepping::Smooth | SliderStepping::SmoothStepped(_) => requested_size.width * percent,
            SliderStepping::Stepped(steps) => {
                requested_size.width * V::percent_to_stepped_percent(&*self.start.value(), &*self.end.value(), steps, percent)
            }
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

        let background_y = position.y + self.height() / 2.0 - self.background.height() / 2.0;
        let track_y = position.y + self.height() / 2.0 - self.track.height() / 2.0;
        let thumb_y = position.y + self.height() / 2.0 - self.thumb.height() / 2.0;

        let thumb_x = match &*self.steps.value() {
            SliderStepping::Smooth | SliderStepping::SmoothStepped(_) => {
                self.x() + (self.background.width() - self.thumb.width()) * percent
            },
            SliderStepping::Stepped(steps) => {
                let stepped_percent = V::percent_to_stepped_percent(&*self.start.value(), &*self.end.value(), steps, percent);
                self.x() + (self.background.width() - self.thumb.width()) * stepped_percent
            }
        };

        self.background.set_position(Position::new(position.x, background_y));
        self.track.set_position(Position::new(position.x, track_y));
        self.thumb.set_position(Position::new(thumb_x, thumb_y));

        self.background.position_children(ctx);
        self.track.position_children(ctx);
        self.thumb.position_children(ctx);
    }
}

impl<
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=SliderStepping<V>>,
    En: ReadState<T=bool>,
> Initialize for Slider<V, F, St, S, E, P, En> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        let style = ctx.env.get::<SliderStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle);

        let stepping_type = Map1::read_map(self.steps.clone(), |a| {
            match *a {
                SliderStepping::Smooth => SliderSteppingType::Smooth,
                SliderStepping::Stepped(_) => SliderSteppingType::Stepped,
                SliderStepping::SmoothStepped(_) => SliderSteppingType::SmoothStepped,
            }
        });

        let thumb = style.create_thumb(self.focus.as_dyn_read(), self.enabled.as_dyn_read(), self.percent.as_dyn_read(), stepping_type.as_dyn_read());
        let track = style.create_track(self.focus.as_dyn_read(), self.enabled.as_dyn_read(), self.percent.as_dyn_read(), stepping_type.as_dyn_read());
        let background = style.create_background(self.focus.as_dyn_read(), self.enabled.as_dyn_read(), self.percent.as_dyn_read(), stepping_type.as_dyn_read());

        self.thumb = thumb;
        self.track = track;
        self.background = background;
    }
}

impl<
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=SliderStepping<V>>,
    En: ReadState<T=bool>,
> CommonWidget for Slider<V, F, St, S, E, P, En> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 1, focus: self.focus);
}

impl<
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=SliderStepping<V>>,
    En: ReadState<T=bool>,
> Render for Slider<V, F, St, S, E, P, En> {
    fn render(&mut self, context: &mut RenderContext) {
        self.background.render(context);
        self.track.render(context);
        self.thumb.render(context);
    }
}