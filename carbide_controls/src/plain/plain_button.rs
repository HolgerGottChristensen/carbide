use std::fmt::{Debug, Formatter};
use carbide_core::{CommonWidgetImpl};
use carbide_core::color::ORANGE;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::ModifierKey;
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::focus::Refocus;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, Map3, Map4, ReadState, ReadStateExtNew, State, StateExtNew, TState};
use carbide_core::widget::{Action, CommonWidget, MouseArea, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack};

pub trait PlainButtonDelegate: Clone + 'static {
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget>;
}

impl<K> PlainButtonDelegate for K where K: Fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> + Clone + 'static {
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> {
        self(focus, hovered, pressed, enabled)
    }
}

type DefaultPlainButtonDelegate = fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget>;
type DefaultPlainButtonAction = fn(&mut Environment, ModifierKey);

#[derive(Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainButton<F, A, D, E> where
    F: State<T=Focus>,
    E: ReadState<T=bool>,
    A: Action + Clone + 'static,
    D: PlainButtonDelegate,
{
    id: WidgetId,
    #[state] focus: F,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: D,
    action: A,
    enabled: E,
}

impl PlainButton<Focus, DefaultPlainButtonAction, DefaultPlainButtonDelegate, bool> {
    pub fn new<A: Action + Clone + 'static>(action: A) -> PlainButton<TState<Focus>, A, DefaultPlainButtonDelegate, bool> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            action,
            focus_state,
            PlainButton::default_delegate,
            true,
        )
    }

    fn default_delegate(focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> {
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
            Rectangle::new().fill(background_color),
            Text::new(val),
        ])
    }
}

impl<F: State<T=Focus>, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>> PlainButton<F, A, D, E> {

    pub fn delegate<D2: PlainButtonDelegate>(
        self,
        delegate: D2,
    ) -> PlainButton<F, A, D2, E> {
        let action = self.action;
        let focus_state = self.focus;

        Self::new_internal(action, focus_state, delegate, self.enabled)
    }

    pub fn focused<F2: IntoState<Focus>>(mut self, focused: F2) -> PlainButton<F2::Output, A, D, E> {
        Self::new_internal(self.action, focused.into_state(), self.delegate, self.enabled)
    }

    pub fn enabled<E2: IntoReadState<bool>>(mut self, enabled: E2) -> PlainButton<F, A, D, E2::Output> {
        Self::new_internal(self.action, self.focus, self.delegate, enabled.into_read_state())
    }

    fn new_internal<F2: State<T=Focus> + Clone, A2: Action + Clone + 'static, D2: PlainButtonDelegate, E2: ReadState<T=bool>>(
        action: A2,
        focus: F2,
        delegate: D2,
        enabled: E2,
    ) -> PlainButton<F2, A2, D2, E2> {

        let hovered = LocalState::new(false);
        let pressed = LocalState::new(false);

        let delegate_widget = delegate.call(focus.as_dyn_read(), hovered.as_dyn_read(), pressed.as_dyn_read(), enabled.as_dyn_read());

        let area = MouseArea::new(delegate_widget)
            .on_click({
                let focus = focus.clone();
                let action = action.clone();
                let enabled = enabled.clone();

                move |env: &mut Environment, modifier: ModifierKey| {
                    use carbide_core::state::State;
                    let mut focus = focus.clone();
                    let mut action = action.clone();
                    let mut enabled = enabled.clone();

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
                if *focus.value() == Focus::Focused && *enabled.value() {
                    focus.set_value(Focus::FocusReleased);
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone())
            .pressed(pressed)
            .hovered(hovered);


        PlainButton {
            id: WidgetId::new(),
            focus,
            child: area.boxed(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            delegate,
            action,
            enabled,
        }
    }

}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>> Focusable for PlainButton<F, A, D, E> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>> CommonWidget for PlainButton<F, A, D, E> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>> WidgetExt for PlainButton<F, A, D, E> {}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>> Debug for PlainButton<F, A, D, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainButton")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focus)
            .finish()
    }
}