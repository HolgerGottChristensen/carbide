use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::{MouseEvent, MouseEventHandler};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::layout::Layout;
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{AnyState, IntoReadState, IntoState, LocalState, Map4, ReadState, ReadStateExtNew, State, StateExtNew, TState};
use carbide_core::widget::{CommonWidget, Empty, Rectangle, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable, Layout, MouseEvent, Render)]
pub struct PlainSlider<F, St, S, E, P, Th, In, Bg> where
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: Widget + Clone,
    In: Widget + Clone,
    Bg: Widget + Clone,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] focus: F,
    dragging: bool,

    #[state] state: St,
    #[state] percent: Box<dyn AnyState<T=f64>>,
    #[state] start: S,
    #[state] end: E,
    #[state] steps: P,

    thumb: Th,
    track: In,
    background: Bg,
}

impl PlainSlider<Focus, f64, f64, f64, Option<f64>, Empty, Empty, Empty> {
    pub fn new<St: IntoState<f64>, S: IntoReadState<f64>, E: IntoReadState<f64>>(state: St, start: S, end: E) -> PlainSlider<TState<Focus>, St::Output, S::Output, E::Output, Option<f64>, Box<dyn Widget>, Box<dyn Widget>, Box<dyn Widget>> {
        let focus = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            focus,
            state.into_state(),
            start.into_read_state(),
            end.into_read_state(),
            None,
            default_thumb(),
            default_track(),
            default_background(),
        )
    }
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: Widget + Clone,
    In: Widget + Clone,
    Bg: Widget + Clone,
> PlainSlider<F, St, S, E, P, Th, In, Bg> {
    pub fn step<P2: IntoReadState<Option<f64>>>(self, steps: P2) -> PlainSlider<F, St, S, E, P2::Output, Th, In, Bg> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            steps.into_read_state(),
            self.thumb,
            self.track,
            self.background,
        )
    }

    fn percent_to_stepped_percent(percent: f64, start: f64, end: f64, step_size: f64) -> f64 {
        let range = (end - start);
        let range_mod = range % step_size;
        let percent_lost = range_mod / range;
        let number_of_steps = (range / step_size);
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
        Th2: Widget + Clone,
        In2: Widget + Clone,
        Bg2: Widget + Clone,
    >(focus: F2, state: St2, start: S2, end: E2, steps: P2, thumb: Th2, track: In2, background: Bg2) -> PlainSlider<F2, St2, S2, E2, P2, Th2, In2, Bg2> {
        let percent = Map4::map(
            state.clone(),
            start.ignore_writes(),
            end.ignore_writes(),
            steps.ignore_writes(),
            |state: &f64, start: &f64, end: &f64, steps: &Option<f64>| {
                (*state - *start) / (*end - *start)
            },
            |new_percent: f64, state: &f64, start: &f64, end: &f64, steps: &Option<f64>| {
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

        PlainSlider {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            focus,
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

/*impl PlainSlider {
    pub fn new(value: impl Into<TState<f64>>, start: f64, end: f64) -> Box<Self> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            focus_state,
            value.into(),
            start,
            end,
            ValueState::new(None),
            Self::default_background,
            Self::default_indicator,
            Self::default_thumb,
        )
    }

    pub fn step(mut self, step: f64) -> Box<Self> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            ValueState::new(Some(step)),
            self.background,
            self.indicator,
            self.thumb,
        )
    }

    pub fn background(mut self, background: fn() -> Box<dyn Widget>) -> Box<Self> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            background,
            self.indicator,
            self.thumb,
        )
    }

    pub fn indicator(mut self, indicator: fn() -> Box<dyn Widget>) -> Box<Self> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            self.background,
            indicator,
            self.thumb,
        )
    }

    pub fn thumb(mut self, thumb: fn() -> Box<dyn Widget>) -> Box<Self> {
        Self::new_internal(
            self.focus,
            self.state,
            self.start,
            self.end,
            self.steps,
            self.background,
            self.indicator,
            thumb,
        )
    }

    fn new_internal(
        focus: TState<Focus>,
        state: TState<f64>,
        start: f64,
        end: f64,
        steps: TState<Option<f64>>,
        background: fn() -> Box<dyn Widget>,
        indicator: fn() -> Box<dyn Widget>,
        thumb: fn() -> Box<dyn Widget>,
    ) -> Box<Self> {
        let progress_state = Map4::map(
            state.clone(),
            start,
            end,
            steps.clone(),
            |state: &f64, start: &f64, end: &f64, steps: &Option<f64>| {
                (*state - *start) / (*end - *start)
            },
            |new_value: f64, state: &f64, start: &f64, end: &f64, steps: &Option<f64>| {
                if let Some(steps) = *steps {
                    let number_of_steps = ((end - start) / steps).ceil();

                    let stepped_percent = (number_of_steps * new_value).round() / number_of_steps;

                    (
                        Some(stepped_percent * (*end - *start) + *start),
                        None,
                        None,
                        None,
                    )
                } else {
                    (Some(new_value * (*end - *start) + *start), None, None, None)
                }
            },
        );

        let children = vec![background(), indicator(), thumb()];

        Box::new(PlainSlider {
            id: WidgetId::new(),
            focus,
            child: children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            state,
            percent: progress_state,
            start,
            end,
            cross_axis_alignment: CrossAxisAlignment::Center,
            dragging: false,
            steps,
            thumb,
            indicator,
            background,
        })
    }
}*/

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: Widget + Clone,
    In: Widget + Clone,
    Bg: Widget + Clone,
> Focusable for PlainSlider<F, St, S, E, P, Th, In, Bg> {
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
    Th: Widget + Clone,
    In: Widget + Clone,
    Bg: Widget + Clone,
> MouseEventHandler for PlainSlider<F, St, S, E, P, Th, In, Bg> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {
        match event {
            MouseEvent::Press(_, position, _) => {
                if self.thumb.is_inside(*position) || self.background.is_inside(*position) {
                    self.dragging = true;

                    let relative_to_position = *position - self.position;
                    let p = (relative_to_position.x() - self.thumb.width() / 2.0)
                        / (self.dimension.width - self.thumb.width());
                    self.percent.set_value(p.max(0.0).min(1.0));
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
    Th: Widget + Clone,
    In: Widget + Clone,
    Bg: Widget + Clone,
> Layout for PlainSlider<F, St, S, E, P, Th, In, Bg> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let percent = *self.percent.value();

        let background = self.background.calculate_size(requested_size, env);

        let track_width = if let Some(steps) = *self.steps.value() {
            let stepped_percent = Self::percent_to_stepped_percent(
                *self.percent.value(),
                *self.start.value(),
                *self.end.value(),
                steps
            );

            requested_size.width * stepped_percent
        } else {
            requested_size.width * percent
        };

        let track_dimensions = Dimension::new(track_width, requested_size.height);
        let track = self.track.calculate_size(track_dimensions, env);

        let thumb = self.thumb.calculate_size(requested_size, env);

        let max_height = background.height.max(track.height).max(thumb.height);

        self.set_dimension(Dimension::new(requested_size.width, max_height));
        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
        let position = self.position();

        let background_y = position.y() + self.height() / 2.0 - self.background.height() / 2.0;
        let track_y = position.y() + self.height() / 2.0 - self.track.height() / 2.0;
        let thumb_y = position.y() + self.height() / 2.0 - self.thumb.height() / 2.0;

        let thumb_x = if let Some(steps) = *self.steps.value() {
            let stepped_percent = Self::percent_to_stepped_percent(
                *self.percent.value(),
                *self.start.value(),
                *self.end.value(),
                steps
            );

            self.x() + (self.background.width() - self.thumb.width()) * stepped_percent
        } else {
            self.x() + (self.background.width() - self.thumb.width()) * *self.percent.value()
        };

        self.background.set_position(Position::new(position.x(), background_y));
        self.track.set_position(Position::new(position.x(), track_y));
        self.thumb.set_position(Position::new(thumb_x, thumb_y));

        self.background.position_children(env);
        self.track.position_children(env);
        self.thumb.position_children(env);
    }
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: Widget + Clone,
    In: Widget + Clone,
    Bg: Widget + Clone,
> CommonWidget for PlainSlider<F, St, S, E, P, Th, In, Bg> {
    CommonWidgetImpl!(self, id: self.id, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 1);
}

impl<
    F: State<T=Focus>,
    St: State<T=f64>,
    S: ReadState<T=f64>,
    E: ReadState<T=f64>,
    P: ReadState<T=Option<f64>>,
    Th: Widget + Clone,
    In: Widget + Clone,
    Bg: Widget + Clone,
> Render for PlainSlider<F, St, S, E, P, Th, In, Bg> {
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
    Th: Widget + Clone,
    In: Widget + Clone,
    Bg: Widget + Clone,
> WidgetExt for PlainSlider<F, St, S, E, P, Th, In, Bg> {}


// ---------------------------------------------------
//  Delegates
// ---------------------------------------------------
fn default_background() -> Box<dyn Widget> {
    Rectangle::new()
        .fill(EnvironmentColor::Red)
        .frame_fixed_height(26.0)
}

fn default_track() -> Box<dyn Widget> {
    Rectangle::new()
        .fill(EnvironmentColor::Green)
        .frame_fixed_height(26.0)
}

fn default_thumb() -> Box<dyn Widget> {
    Rectangle::new().fill(EnvironmentColor::Blue).frame(26.0, 26.0).boxed()
}