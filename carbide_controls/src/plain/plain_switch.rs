use std::fmt::{Debug, Formatter};

use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide_core::widget::{AnyWidget, CommonWidget, MouseArea, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack};

use crate::{enabled_state, EnabledState};

pub trait PlainSwitchDelegate: Clone + 'static {
    fn call(&self, focus: impl ReadState<T=Focus>, checked: impl ReadState<T=bool>, enabled: impl ReadState<T=bool>) -> Box<dyn AnyWidget>;
}

impl<K> PlainSwitchDelegate for K where K: Fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> + Clone + 'static {
    fn call(&self, item: impl ReadState<T=Focus>, index: impl ReadState<T=bool>, enabled: impl ReadState<T=bool>) -> Box<dyn AnyWidget> {
        self(item.as_dyn_read(), index.as_dyn_read(), enabled.as_dyn_read())
    }
}

type DefaultPlainSwitchDelegate = fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget>;

/// # A plain switch widget
/// This widget contains the basic logic for a switch component, without any styling.
/// It can be styled by setting the delegate, using the delegate method.
///
/// For a styled version, use [crate::Switch] instead.
#[derive(Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainSwitch<F, C, D, E> where
    F: State<T=Focus>,
    C: State<T=bool>,
    D: PlainSwitchDelegate,
    E: ReadState<T=bool>,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] focus: F,
    #[state] enabled: E,

    child: Box<dyn AnyWidget>,
    delegate: D,
    #[state] checked: C,
}

impl PlainSwitch<Focus, bool, DefaultPlainSwitchDelegate, bool> {
    pub fn new<C: IntoState<bool>>(checked: C) -> PlainSwitch<LocalState<Focus>, C::Output, DefaultPlainSwitchDelegate, EnabledState> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            checked.into_state(),
            focus_state,
            PlainSwitch::default_delegate,
            enabled_state(),
        )
    }

    fn default_delegate(focus: Box<dyn AnyReadState<T=Focus>>, checked: Box<dyn AnyReadState<T=bool>>, _enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
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
            Rectangle::new().fill(background_color).boxed(),
            Text::new(val).boxed(),
        ]).boxed()
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate, E: ReadState<T=bool>> PlainSwitch<F, C, D, E> {

    pub fn delegate<D2: PlainSwitchDelegate>(
        self,
        delegate: D2,
    ) -> PlainSwitch<F, C, D2, E> {
        let checked = self.checked;
        let focus_state = self.focus;

        Self::new_internal(checked, focus_state, delegate, self.enabled)
    }

    pub fn focused<F2: IntoState<Focus>>(self, focused: F2) -> PlainSwitch<F2::Output, C, D, E> {
        Self::new_internal(self.checked, focused.into_state(), self.delegate, self.enabled)
    }

    pub fn enabled<E2: IntoReadState<bool>>(self, enabled: E2) -> PlainSwitch<F, C, D, E2::Output> {
        Self::new_internal(self.checked, self.focus, self.delegate, enabled.into_read_state())
    }

    fn new_internal<F2: State<T=Focus> + Clone, C2: State<T=bool> + Clone, D2: PlainSwitchDelegate, E2: ReadState<T=bool>>(
        checked: C2,
        focus: F2,
        delegate: D2,
        enabled: E2,
    ) -> PlainSwitch<F2, C2, D2, E2> {

        let delegate_widget = delegate.call(focus.as_dyn(), checked.as_dyn(), enabled.as_dyn_read());

        let button = Box::new(MouseArea::new(delegate_widget)
            .on_click(capture!([checked, focus, enabled], |env: &mut Environment| {
                enabled.sync(env);
                focus.sync(env);
                checked.sync(env);

                if !*enabled.value() {
                    return;
                }
                let current = !*checked.value();
                checked.set_value(current);

                if *focus.value() != Focus::Focused {
                    focus.set_value(Focus::FocusRequested);
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .on_click_outside(capture!([focus], |env: &mut Environment| {
                if *focus.value() == Focus::Focused {
                    focus.set_value(Focus::FocusReleased);
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone()));

        let child = button;

        PlainSwitch {
            id: WidgetId::new(),
            focus,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            delegate,
            checked,
            enabled,
        }
    }

}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate, E: ReadState<T=bool>> Focusable for PlainSwitch<F, C, D, E> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate, E: ReadState<T=bool>> CommonWidget for PlainSwitch<F, C, D, E> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate, E: ReadState<T=bool>> WidgetExt for PlainSwitch<F, C, D, E> {}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate, E: ReadState<T=bool>> Debug for PlainSwitch<F, C, D, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainSwitch")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focus)
            .field("checked", &self.checked)
            .finish()
    }
}
