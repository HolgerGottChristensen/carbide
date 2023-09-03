use std::fmt::{Debug, Formatter};
use carbide_core::{CommonWidgetImpl};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::ModifierKey;
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::focus::Refocus;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, Map3, ReadState, ReadStateExtNew, State, StateExtNew, TState};
use carbide_core::widget::{Action, CommonWidget, MouseArea, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack};

pub trait PlainButtonDelegate: Clone + 'static {
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget>;
}

impl<K> PlainButtonDelegate for K where K: Fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> + Clone + 'static {
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> {
        self(focus, hovered, pressed)
    }
}

type DefaultPlainButtonDelegate = fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget>;
type DefaultPlainButtonAction = fn(&mut Environment, ModifierKey);

#[derive(Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainButton<F, A, D> where
    F: State<T=Focus> + Clone,
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
}

impl PlainButton<Focus, DefaultPlainButtonAction, DefaultPlainButtonDelegate> {
    pub fn new<A: Action + Clone + 'static>(action: A) -> PlainButton<TState<Focus>, A, DefaultPlainButtonDelegate> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            action,
            focus_state,
            PlainButton::default_delegate,
        )
    }

    fn default_delegate(focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> {
        let val = Map1::read_map(focus, |focused| {
            format!("{:?}", focused)
        });

        let background_color = Map3::read_map(EnvironmentColor::Accent.color(), pressed, hovered, |col, pressed, hovered| {
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

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate> PlainButton<F, A, D> {

    pub fn delegate<D2: PlainButtonDelegate>(
        self,
        delegate: D2,
    ) -> PlainButton<F, A, D2> {
        let action = self.action;
        let focus_state = self.focus;

        Self::new_internal(action, focus_state, delegate)
    }

    pub fn focused<F2: IntoState<Focus>>(mut self, focused: F2) -> PlainButton<F2::Output, A, D> {
        Self::new_internal(self.action, focused.into_state(), self.delegate)
    }

    fn new_internal<F2: State<T=Focus> + Clone, A2: Action + Clone + 'static, D2: PlainButtonDelegate>(
        action: A2,
        focus: F2,
        delegate: D2,
    ) -> PlainButton<F2, A2, D2> {

        let hovered = LocalState::new(false);
        let pressed = LocalState::new(false);

        let delegate_widget = delegate.call(focus.as_dyn_read(), hovered.as_dyn_read(), pressed.as_dyn_read());

        let area = MouseArea::new(delegate_widget)
            .on_click({
                let focus = focus.clone();
                let action = action.clone();

                move |env: &mut Environment, modifier: ModifierKey| {
                    use carbide_core::state::State;
                    let mut focus = focus.clone();
                    let mut action = action.clone();

                    {
                        if *focus.value() != Focus::Focused {
                            focus.set_value(Focus::FocusRequested);
                            env.request_focus(Refocus::FocusRequest);
                        }
                        (action)(env, modifier);
                    }
                }
            })
            .on_click_outside(capture!([focus], |env: &mut Environment| {
                if *focus.value() == Focus::Focused {
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
        }
    }

}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate> Focusable for PlainButton<F, A, D> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate> CommonWidget for PlainButton<F, A, D> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate> WidgetExt for PlainButton<F, A, D> {}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate> Debug for PlainButton<F, A, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainButton")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focus)
            .finish()
    }
}