use std::fmt::{Debug, Formatter};
use carbide::environment::IntoColorReadState;
use carbide_core::{CommonWidgetImpl};
use carbide_core::color::ORANGE;
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::ModifierKey;
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::Focus;
use carbide_core::focus::Refocus;
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map1, Map4, ReadState, ReadStateExtNew, State};
use carbide_core::widget::{Action, CommonWidget, MouseArea, Rectangle, Text, AnyWidget, WidgetExt, WidgetId, ZStack, Widget};
use crate::{enabled_state, EnabledState};

pub trait PlainButtonDelegate: Clone + 'static {
    type Output: Widget;
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Self::Output;
}

impl<K, C: Widget> PlainButtonDelegate for K where K: Fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> C + Clone + 'static {
    type Output = C;
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> C {
        self(focus, hovered, pressed, enabled)
    }
}

type DefaultPlainButtonDelegate = fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget>;
type DefaultPlainButtonAction = fn(&mut Environment, ModifierKey);

#[derive(Clone, Widget)]
pub struct PlainButton<F, A, D, E, H, P> where
    F: State<T=Focus>,
    A: Action + Clone + 'static,
    D: PlainButtonDelegate,
    E: ReadState<T=bool>,
    H: State<T=bool>,
    P: State<T=bool>,
{
    id: WidgetId,
    #[state] focus: F,
    #[state] enabled: E,
    child: Box<dyn AnyWidget>,
    position: Position,
    dimension: Dimension,
    delegate: D,
    action: A,
    #[state] hovered: H,
    #[state] pressed: P,
    cursor: MouseCursor,
}

impl PlainButton<Focus, DefaultPlainButtonAction, DefaultPlainButtonDelegate, bool, bool, bool> {
    pub fn new<A: Action + Clone + 'static>(action: A) -> PlainButton<LocalState<Focus>, A, DefaultPlainButtonDelegate, EnabledState, LocalState<bool>, LocalState<bool>> {

        let focus_state = LocalState::new(Focus::Unfocused);
        let hovered = LocalState::new(false);
        let pressed = LocalState::new(false);

        Self::new_internal(
            action,
            focus_state,
            PlainButton::default_delegate,
            enabled_state(),
            MouseCursor::Pointer,
            hovered,
            pressed
        )
    }

    fn default_delegate(focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let val = Map1::read_map(focus, |focused| {
            format!("{:?}", focused)
        });

        let background_color = Map4::read_map(EnvironmentColor::Accent.color(), pressed, hovered, enabled, |col, pressed, hovered, enabled| {
            if !*enabled {
                return ORANGE;
            }

            if *pressed {
                col.darkened(0.1)
            } else if *hovered {
                col.lightened(0.1)
            } else {
                *col
            }
        });

        ZStack::new(vec![
            Rectangle::new().fill(background_color).boxed(),
            Text::new(val).boxed(),
        ]).boxed()
    }
}

impl<F: State<T=Focus>, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>> PlainButton<F, A, D, E, H, P> {

    pub fn delegate<D2: PlainButtonDelegate>(
        self,
        delegate: D2,
    ) -> PlainButton<F, A, D2, E, H, P> {
        let action = self.action;
        let focus_state = self.focus;

        Self::new_internal(action, focus_state, delegate, self.enabled, self.cursor, self.hovered, self.pressed)
    }

    pub fn focused<F2: IntoState<Focus>>(self, focused: F2) -> PlainButton<F2::Output, A, D, E, H, P> {
        Self::new_internal(self.action, focused.into_state(), self.delegate, self.enabled, self.cursor, self.hovered, self.pressed)
    }

    pub fn pressed<P2: IntoState<bool>>(self, pressed: P2) -> PlainButton<F, A, D, E, H, P2::Output> {
        Self::new_internal(self.action, self.focus, self.delegate, self.enabled, self.cursor, self.hovered, pressed.into_state())
    }

    pub fn hovered<H2: IntoState<bool>>(self, hovered: H2) -> PlainButton<F, A, D, E, H2::Output, P> {
        Self::new_internal(self.action, self.focus, self.delegate, self.enabled, self.cursor, hovered.into_state(), self.pressed)
    }

    pub fn enabled<E2: IntoReadState<bool>>(self, enabled: E2) -> PlainButton<F, A, D, E2::Output, H, P> {
        Self::new_internal(self.action, self.focus, self.delegate, enabled.into_read_state(), self.cursor, self.hovered, self.pressed)
    }

    pub fn cursor(self, cursor: MouseCursor) -> PlainButton<F, A, D, E, H, P> {
        Self::new_internal(self.action, self.focus, self.delegate, self.enabled, cursor, self.hovered, self.pressed)
    }

    fn new_internal<
        F2: State<T=Focus> + Clone,
        A2: Action + Clone + 'static,
        D2: PlainButtonDelegate,
        E2: ReadState<T=bool>,
        H2: State<T=bool>,
        P2: State<T=bool>,
    >(
        action: A2,
        focus: F2,
        delegate: D2,
        enabled: E2,
        cursor: MouseCursor,
        hovered: H2,
        pressed: P2,
    ) -> PlainButton<F2, A2, D2, E2, H2, P2> {

        let delegate_widget = delegate.call(focus.as_dyn_read(), hovered.as_dyn_read(), pressed.as_dyn_read(), enabled.as_dyn_read());

        let area = MouseArea::new(delegate_widget)
            .on_click({
                let focus = focus.clone();
                let action = action.clone();
                let enabled = enabled.clone();

                move |env: &mut Environment, modifier: ModifierKey| {
                    let mut focus = focus.clone();
                    let action = action.clone();
                    let mut enabled = enabled.clone();
                    enabled.sync(env);
                    focus.sync(env);
                    enabled.sync(env);

                    {
                        if *enabled.value() {
                            if *focus.value() != Focus::Focused {
                                focus.set_value(Focus::FocusRequested);
                                env.request_focus(Refocus::FocusRequest);
                            }
                            (action)(env, modifier);
                        }


                    }
                }
            })
            .on_click_outside(capture!([focus, enabled], |env: &mut Environment| {
                focus.sync(env);
                if *focus.value() == Focus::Focused {
                    focus.set_value(Focus::FocusReleased);
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone())
            .pressed(pressed.clone())
            .hovered(hovered.clone())
            .hover_cursor(cursor);


        PlainButton {
            id: WidgetId::new(),
            focus,
            child: area.boxed(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            delegate,
            action,
            hovered,
            enabled,
            cursor,
            pressed,
        }
    }

}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>> CommonWidget for PlainButton<F, A, D, E, H, P> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>> WidgetExt for PlainButton<F, A, D, E, H, P> {}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>> Debug for PlainButton<F, A, D, E, H, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainButton")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focus)
            .field("child", &self.child)
            .finish()
    }
}