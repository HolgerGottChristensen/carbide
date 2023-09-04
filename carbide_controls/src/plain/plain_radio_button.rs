use std::fmt::{Debug, Formatter};
use carbide_core::CommonWidgetImpl;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, ReadStateExtNew, State, StateContract, StateExtNew, TState};
use carbide_core::widget::{CommonWidget, HStack, MouseArea, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack};

use crate::PlainButton;

pub trait PlainRadioButtonDelegate: Clone + 'static {
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, selected: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget>;
}

impl<K> PlainRadioButtonDelegate for K where K: Fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> + Clone + 'static {
    fn call(&self, item: Box<dyn AnyReadState<T=Focus>>, index: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> {
        self(item, index)
    }
}

type DefaultPlainRadioButtonDelegate = fn(focus: Box<dyn AnyReadState<T=Focus>>, selected: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget>;

#[derive(Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainRadioButton<T, F, C, D>
where
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    C: State<T=T>,
    D: PlainRadioButtonDelegate,
{
    id: WidgetId,
    #[state] focus: F,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: D,
    reference: T,
    #[state] local_state: C,
    #[state] selected_state: Box<dyn AnyReadState<T=bool>>,
}

impl PlainRadioButton<bool, Focus, bool, DefaultPlainRadioButtonDelegate> {
    pub fn new<T: StateContract + PartialEq, S: IntoState<T>>(reference: T, selected: S) -> PlainRadioButton<T, TState<Focus>, S::Output, DefaultPlainRadioButtonDelegate> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(focus_state, reference, selected.into_state(), Self::default_delegate)
    }

    fn default_delegate(focus: Box<dyn AnyReadState<T=Focus>>, selected: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget> {
        let background_color = Map1::read_map(selected.clone(), |is_checked| {
            if *is_checked {
                EnvironmentColor::Green
            } else {
                EnvironmentColor::Red
            }
        });

        let val = Map2::read_map(selected, focus, |checked: &bool, focus: &Focus| {
            format!("{:?}, {:?}", *checked, focus)
        });

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(val),
        ])
    }
}

impl<T: StateContract + PartialEq, F: State<T=Focus>, C: State<T=T>, D: PlainRadioButtonDelegate> PlainRadioButton<T, F, C, D> {

    pub fn focused<F2: IntoState<Focus>>(mut self, focused: F2) -> PlainRadioButton<T, F2::Output, C, D> {
        Self::new_internal(focused.into_state(), self.reference, self.local_state, self.delegate)
    }

    pub fn delegate<D2: PlainRadioButtonDelegate>(
        self,
        delegate: D2,
    ) -> PlainRadioButton<T, F, C, D2> {
        Self::new_internal(self.focus, self.reference, self.local_state, delegate)
    }

    pub fn new_internal<T2: StateContract + PartialEq, F2: State<T=Focus>, C2: State<T=T2>, D2: PlainRadioButtonDelegate>(
        focus: F2,
        reference: T2,
        state: C2,
        delegate: D2,
    ) -> PlainRadioButton<T2, F2, C2, D2> {

        let local_reference = reference.clone();
        let local_reference2 = reference.clone();

        let selected = Map1::read_map(state.clone(), move |a| {
            *a == local_reference
        }).as_dyn_read();

        let delegate_widget = delegate.call(focus.as_dyn_read(), selected.clone());

        let button = MouseArea::new(delegate_widget)
            .on_click(capture!(
                [state, focus],
                |env: &mut Environment| {
                    state.set_value(local_reference2.clone());

                    if *focus.value() != Focus::Focused {
                        focus.set_value(Focus::FocusRequested);
                        env.request_focus(Refocus::FocusRequest);
                    }
                }
            )).on_click_outside(capture!(
                [focus],
                |env: &mut Environment| {
                    if *focus.value() == Focus::Focused {
                        focus.set_value(Focus::FocusReleased);
                        env.request_focus(Refocus::FocusRequest);
                    }
                }
            ))
            .focused(focus.clone());

        let button = Box::new(button);

        let child = button;

        PlainRadioButton {
            id: WidgetId::new(),
            focus,
            child,
            position: Default::default(),
            dimension: Default::default(),
            delegate,
            reference,
            local_state: state,
            selected_state: selected,
        }
    }
}

impl<T: StateContract + PartialEq, F: State<T=Focus>, C: State<T=T>, D: PlainRadioButtonDelegate> Focusable for PlainRadioButton<T, F, C, D> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<T: StateContract + PartialEq, F: State<T=Focus>, C: State<T=T>, D: PlainRadioButtonDelegate> CommonWidget for PlainRadioButton<T, F, C, D> {
    CommonWidgetImpl!(
        self,
        id: self.id,
        child: self.child,
        position: self.position,
        dimension: self.dimension,
        flag: Flags::FOCUSABLE,
        flexibility: 10,
        focus: self.focus
    );
}

impl<T: StateContract + PartialEq, F: State<T=Focus>, C: State<T=T>, D: PlainRadioButtonDelegate> WidgetExt for PlainRadioButton<T, F, C, D> {}

impl<T: StateContract + PartialEq, F: State<T=Focus>, C: State<T=T>, D: PlainRadioButtonDelegate> Debug for PlainRadioButton<T, F, C, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainRadioButton")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focus)
            .field("reference", &self.reference)
            .field("state", &self.local_state)
            .field("selected", &self.selected_state)
            .finish()
    }
}