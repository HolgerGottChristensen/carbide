use std::fmt::Debug;
use carbide_core::event::{KeyboardEventContext, MouseEventContext};
use carbide_core::layout::LayoutContext;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::EnvironmentColor;
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseEvent, MouseEventHandler};
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::{Focus, Refocus};
use carbide_core::layout::Layout;
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map3, Map4, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide_core::widget::{CommonWidget, Empty, Rectangle, WidgetExt, WidgetId, Widget};
use crate::{enabled_state, EnabledState};

const SMOOTH_VALUE_INCREMENT: f64 = 0.05;
const SMOOTH_VALUE_SMALL_INCREMENT: f64 = 0.01;
const STEP_SMOOTH_BEHAVIOR: bool = false;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout, MouseEvent, KeyboardEvent, Render)]
pub struct PlainSlider<V, F, St, S, E, P, Th, In, Bg, En> where
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=Option<V>>,
    Th: Widget,
    In: Widget,
    Bg: Widget,
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

    thumb_delegate: Delegate<Th, V>,
    thumb: Th,
    track_delegate: Delegate<In, V>,
    track: In,
    background_delegate: Delegate<Bg, V>,
    background: Bg,
}

impl PlainSlider<f64, Focus, f64, f64, f64, Option<f64>, Empty, Empty, Empty, bool> {
    pub fn new<V: SliderValue, St: State<T=V>, S: IntoReadState<V>, E: IntoReadState<V>>(state: St, start: S, end: E) -> PlainSlider<V, LocalState<Focus>, St, S::Output, E::Output, Option<V>, impl Widget, impl Widget, impl Widget, EnabledState> {
        let focus = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            focus,
            state,
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
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=Option<V>>,
    Th: Widget,
    In: Widget,
    Bg: Widget,
    En: ReadState<T=bool>,
> PlainSlider<V, F, St, S, E, P, Th, In, Bg, En> {
    pub fn step<P2: IntoReadState<Option<V>>>(self, steps: P2) -> PlainSlider<V, F, St, S, E, P2::Output, Th, In, Bg, En> {
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

    pub fn focused<F2: IntoState<Focus>>(self, focus: F2) -> PlainSlider<V, F2::Output, St, S, E, P, Th, In, Bg, En> {
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

    pub fn enabled<En2: IntoReadState<bool>>(self, enabled: En2) -> PlainSlider<V, F, St, S, E, P, Th, In, Bg, En2::Output> {
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

    pub fn background<Bg2: Widget>(self, delegate: Delegate<Bg2, V>) -> PlainSlider<V, F, St, S, E, P, Th, In, Bg2, En> {
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

    pub fn thumb<Th2: Widget>(self, delegate: Delegate<Th2, V>) -> PlainSlider<V, F, St, S, E, P, Th2, In, Bg, En> {
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

    pub fn track<In2: Widget>(self, delegate: Delegate<In2, V>) -> PlainSlider<V, F, St, S, E, P, Th, In2, Bg, En> {
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

    #[allow(unused)]
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
        V2: SliderValue,
        F2: State<T=Focus>,
        St2: State<T=V2>,
        S2: ReadState<T=V2>,
        E2: ReadState<T=V2>,
        P2: ReadState<T=Option<V2>>,
        Th2: Widget,
        In2: Widget,
        Bg2: Widget,
        En2: ReadState<T=bool>,
    >(focus: F2, state: St2, start: S2, end: E2, steps: P2, thumb_delegate: Delegate<Th2, V2>, track_delegate: Delegate<In2, V2>, background_delegate: Delegate<Bg2, V2>, enabled: En2) -> PlainSlider<V2, F2, St2, S2, E2, P2, Th2, In2, Bg2, En2> {
        let percent = Map4::map(
            state.clone(),
            start.ignore_writes(),
            end.ignore_writes(),
            steps.ignore_writes(),
            |state: &V2, start: &V2, end: &V2, _steps: &Option<V2>| {
                V2::percent(state, start, end)
            },
            |new_percent: f64, mut state, start, end, steps| {
                if let Some(step_size) = &*steps {
                    *state = V2::stepped_interpolate(&*start, &*end, step_size, new_percent);
                } else {
                    *state = V2::interpolate(&*start, &*end, new_percent);
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
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=Option<V>>,
    Th: Widget,
    In: Widget,
    Bg: Widget,
    En: ReadState<T=bool>,
> KeyboardEventHandler for PlainSlider<V, F, St, S, E, P, Th, In, Bg, En> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, _ctx: &mut KeyboardEventContext) {
        if !*self.enabled.value() {
            return;
        }

        if *self.focus.value() == Focus::Focused && self.steps.value().is_none() {
            let value = *self.percent.value();

            match event {
                KeyboardEvent::Press(Key::ArrowRight, ModifierKey::CONTROL) => {
                    self.percent.set_value(1.0);
                }
                KeyboardEvent::Press(Key::ArrowLeft, ModifierKey::CONTROL) => {
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
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=Option<V>>,
    Th: Widget,
    In: Widget,
    Bg: Widget,
    En: ReadState<T=bool>,
> MouseEventHandler for PlainSlider<V, F, St, S, E, P, Th, In, Bg, En> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !*self.enabled.value() {
            return;
        }

        match event {
            MouseEvent::Press { position, .. } => {
                if self.thumb.is_inside(*position) || self.background.is_inside(*position) {
                    if *self.focus.value() != Focus::Focused {
                        self.focus.set_value(Focus::FocusRequested);
                        ctx.env.request_focus(Refocus::FocusRequest);
                    }

                    self.dragging = true;

                    let relative_to_position = *position - self.position;
                    let p = (relative_to_position.x - self.thumb.width() / 2.0)
                        / (self.dimension.width - self.thumb.width());
                    self.percent.set_value(p.max(0.0).min(1.0));
                } else {
                    if *self.focus.value() == Focus::Focused {
                        self.focus.set_value(Focus::FocusReleased);
                        ctx.env.request_focus(Refocus::FocusRequest);
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
    P: ReadState<T=Option<V>>,
    Th: Widget,
    In: Widget,
    Bg: Widget,
    En: ReadState<T=bool>,
> Layout for PlainSlider<V, F, St, S, E, P, Th, In, Bg, En> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let percent = self.percent.value().max(0.0).min(1.0);

        let background = self.background.calculate_size(requested_size, ctx);

        let track_width = if let Some(steps) = &*self.steps.value() {
            if STEP_SMOOTH_BEHAVIOR {
                requested_size.width * percent
            } else {
                requested_size.width * V::stepped_percent(&*self.start.value(), &*self.end.value(), steps, percent)
            }
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

        let background_y = position.y + self.height() / 2.0 - self.background.height() / 2.0;
        let track_y = position.y + self.height() / 2.0 - self.track.height() / 2.0;
        let thumb_y = position.y + self.height() / 2.0 - self.thumb.height() / 2.0;

        let thumb_x = if let Some(steps) = &*self.steps.value() {
            if STEP_SMOOTH_BEHAVIOR {
                self.x() + (self.background.width() - self.thumb.width()) * percent
            } else {
                let stepped_percent = V::stepped_percent(&*self.start.value(), &*self.end.value(), steps, percent);
                self.x() + (self.background.width() - self.thumb.width()) * stepped_percent
            }
        } else {
            self.x() + (self.background.width() - self.thumb.width()) * percent
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
    P: ReadState<T=Option<V>>,
    Th: Widget,
    In: Widget,
    Bg: Widget,
    En: ReadState<T=bool>,
> CommonWidget for PlainSlider<V, F, St, S, E, P, Th, In, Bg, En> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 1, focus: self.focus);
}

impl<
    V: SliderValue,
    F: State<T=Focus>,
    St: State<T=V>,
    S: ReadState<T=V>,
    E: ReadState<T=V>,
    P: ReadState<T=Option<V>>,
    Th: Widget,
    In: Widget,
    Bg: Widget,
    En: ReadState<T=bool>,
> Render for PlainSlider<V, F, St, S, E, P, Th, In, Bg, En> {
    fn render(&mut self, context: &mut RenderContext) {
        self.background.render(context);
        self.track.render(context);
        self.thumb.render(context);
    }
}


// ---------------------------------------------------
//  Delegates
// ---------------------------------------------------
type Delegate<W, V> = fn(
    state: Box<dyn AnyState<T=V>>,
    start: Box<dyn AnyReadState<T=V>>,
    end: Box<dyn AnyReadState<T=V>>,
    steps: Box<dyn AnyReadState<T=Option<V>>>,
    focus: Box<dyn AnyReadState<T=Focus>>,
    enabled: Box<dyn AnyReadState<T=bool>>,
) -> W;

fn default_background<V: SliderValue>(_state: Box<dyn AnyState<T=V>>, _start: Box<dyn AnyReadState<T=V>>, _end: Box<dyn AnyReadState<T=V>>, _steps: Box<dyn AnyReadState<T=Option<V>>>, _focus: Box<dyn AnyReadState<T=Focus>>, _enabled: Box<dyn AnyReadState<T=bool>>,) -> impl Widget {
    Rectangle::new()
        .fill(EnvironmentColor::Red)
        .frame_fixed_height(26.0)
        .boxed()
}

fn default_track<V: SliderValue>(_state: Box<dyn AnyState<T=V>>, _start: Box<dyn AnyReadState<T=V>>, _end: Box<dyn AnyReadState<T=V>>, _steps: Box<dyn AnyReadState<T=Option<V>>>, _focus: Box<dyn AnyReadState<T=Focus>>, _enabled: Box<dyn AnyReadState<T=bool>>,) -> impl Widget {
    Rectangle::new()
        .fill(EnvironmentColor::Green)
        .frame_fixed_height(26.0)
        .boxed()
}

fn default_thumb<V: SliderValue>(state: Box<dyn AnyState<T=V>>, start: Box<dyn AnyReadState<T=V>>, end: Box<dyn AnyReadState<T=V>>, _steps: Box<dyn AnyReadState<T=Option<V>>>, _focus: Box<dyn AnyReadState<T=Focus>>, _enabled: Box<dyn AnyReadState<T=bool>>,) -> impl Widget {
    let color = Map3::read_map(state, start, end, |state, start, end| {
        if state < start || state > end {
            EnvironmentColor::Purple
        } else {
            EnvironmentColor::Blue
        }
    });

    Rectangle::new().fill(color).frame(26.0, 26.0).boxed()
}


pub trait SliderValue: Debug + Clone + PartialEq + PartialOrd + 'static {
    fn interpolate(&self, other: &Self, percentage: f64) -> Self;
    fn stepped_interpolate(&self, other: &Self, step_size: &Self, percentage: f64) -> Self {
        let stepped = Self::stepped_percent(self, other, step_size, percentage);
        Self::interpolate(self, other, stepped)
    }
    fn percent(&self, start: &Self, end: &Self) -> f64;
    fn stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64;
}

impl SliderValue for f64 {
    fn interpolate(&self, other: &Self, percentage: f64) -> Self {
        percentage * (*other - *self) + *self
    }

    fn percent(&self, start: &Self, end: &Self) -> f64 {
        (*self - *start) / (*end - *start)
    }

    fn stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64 {
        let range = *other - *self;
        let range_mod = range % step_size;
        let percent_lost = range_mod / range;
        let number_of_steps = range / step_size;
        let percent_per_step = (number_of_steps * percentage).round() / number_of_steps;

        if percentage > 1.0 - percent_lost / 2.0 {
            1.0
        } else {
            percent_per_step
        }
    }
}

impl SliderValue for f32 {
    fn interpolate(&self, other: &Self, percentage: f64) -> Self {
        (percentage * ((*other - *self) as f64)) as f32 + *self
    }

    fn percent(&self, start: &Self, end: &Self) -> f64 {
        (*self - *start) as f64 / (*end - *start) as f64
    }

    fn stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64 {
        let range = *other - *self;
        let range_mod = (range % step_size) as f64;
        let percent_lost = range_mod / range as f64;
        let number_of_steps = range as f64 / *step_size as f64;
        let percent_per_step = (number_of_steps * percentage).round() / number_of_steps;

        if percentage > 1.0 - percent_lost / 2.0 {
            1.0
        } else {
            percent_per_step
        }
    }
}

macro_rules! impl_slider_value {
    ($($typ: ty),*) => {
        $(
        impl SliderValue for $typ {
            fn interpolate(&self, other: &Self, percentage: f64) -> Self {
                (percentage * (*other - *self) as f64).round() as $typ + *self
            }

            fn percent(&self, start: &Self, end: &Self) -> f64 {
                self.saturating_sub(*start) as f64 / (*end - *start) as f64
            }

            fn stepped_percent(&self, other: &Self, step_size: &Self, percentage: f64) -> f64 {
                let range = *other - *self;
                let range_mod = (range % step_size) as f64;
                let percent_lost = range_mod / range as f64;
                let number_of_steps = range as f64 / *step_size as f64;
                let percent_per_step = (number_of_steps * percentage).round() / number_of_steps;

                if percentage > 1.0 - percent_lost / 2.0 {
                    1.0
                } else {
                    percent_per_step
                }
            }
        }
        )*
    };
}

impl_slider_value!(u8, u16, u32, u64, u128, usize);
impl_slider_value!(i8, i16, i32, i64, i128, isize);
