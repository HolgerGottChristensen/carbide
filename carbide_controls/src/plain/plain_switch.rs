use carbide_core::{CommonWidgetImpl};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::focus::Refocus;
use carbide_core::state::{AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, State, StateExtNew, TState};
use carbide_core::widget::{
    CommonWidget, HStack, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack,
};

use crate::PlainButton;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainSwitch<F, L, C> where
    F: State<T=Focus> + Clone,
    L: ReadState<T=String> + Clone,
    C: State<T=bool> + Clone
{
    id: WidgetId,
    #[state] focus: F,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget>,
    #[state] label: L,
    #[state] checked: C,
}

impl PlainSwitch<Focus, String, bool> {
    pub fn new<L: IntoReadState<String>, C: IntoState<bool>>(label: L, checked: C) -> PlainSwitch<TState<Focus>, L::Output, C::Output> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            checked.into_state(),
            focus_state,
            PlainSwitch::default_delegate,
            label.into_read_state(),
        )
    }

    fn default_delegate(focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget> {
        let background_color = Map1::read_map(checked.clone(), |is_checked| {
            if *is_checked {
                EnvironmentColor::Green
            } else {
                EnvironmentColor::Red
            }
        });

        let val = Map2::read_map(checked, focus, |checked: &bool, focus: &Focus| {
            format!("{:?}, {:?}", *checked, focus)
        });

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(val),
        ])
    }
}

impl<F: State<T=Focus> + Clone, L: ReadState<T=String> + Clone, C: State<T=bool> + Clone> PlainSwitch<F, L, C> {

    pub fn delegate(
        self,
        delegate: fn(focus: Box<dyn AnyState<T=Focus>>, selected: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget>,
    ) -> PlainSwitch<F, L, C> {
        let checked = self.checked;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(checked, focus_state, delegate, label_state)
    }

    pub fn focused<F2: IntoState<Focus>>(mut self, focused: F2) -> PlainSwitch<F2::Output, L, C> {
        Self::new_internal(self.checked, focused.into_state(), self.delegate, self.label)
    }

    fn new_internal<F2: State<T=Focus> + Clone, L2: ReadState<T=String> + Clone, C2: State<T=bool> + Clone>(
        checked: C2,
        focus: F2,
        delegate: fn(focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget>,
        label_state: L2,
    ) -> PlainSwitch<F2, L2, C2> {

        let delegate_widget = delegate(focus.as_dyn(), checked.as_dyn());

        let button = Box::new(PlainButton::new(delegate_widget)
            .on_click(capture!([checked, focus], |env: &mut Environment| {
                *checked = !*checked;

                if *focus != Focus::Focused {
                    *focus = Focus::FocusRequested;
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone()));

        let child = HStack::new(vec![button, Text::new(label_state.clone())]).spacing(5.0);

        PlainSwitch {
            id: WidgetId::new(),
            focus,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            delegate,
            label: label_state,
            checked,
        }
    }

}

impl<F: State<T=Focus> + Clone, L: ReadState<T=String> + Clone, C: State<T=bool> + Clone> Focusable for PlainSwitch<F, L, C> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<F: State<T=Focus> + Clone, L: ReadState<T=String> + Clone, C: State<T=bool> + Clone> CommonWidget for PlainSwitch<F, L, C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 10);
}

impl<F: State<T=Focus> + Clone, L: ReadState<T=String> + Clone, C: State<T=bool> + Clone> WidgetExt for PlainSwitch<F, L, C> {}
