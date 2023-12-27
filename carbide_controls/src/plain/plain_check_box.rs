use std::fmt::{Debug, Formatter};
use carbide::a;
use carbide::event::ModifierKey;
use carbide_core::{CommonWidgetImpl};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::environment::EnvironmentColor;
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, ReadStateExtNew};
use carbide_core::state::State;
use carbide_core::widget::{CommonWidget, MouseArea, Rectangle, Text, AnyWidget, WidgetExt, WidgetId, ZStack, Widget};
use crate::{enabled_state, EnabledState};

use crate::types::*;

pub trait PlainCheckBoxDelegate: Clone + 'static {
    fn call(&self, focus: impl ReadState<T=Focus>, checked: impl ReadState<T=CheckBoxValue>, enabled: impl ReadState<T=bool>) -> Box<dyn AnyWidget>;
}

impl<K> PlainCheckBoxDelegate for K where K: Fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=CheckBoxValue>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> + Clone + 'static {
    fn call(&self, item: impl ReadState<T=Focus>, index: impl ReadState<T=CheckBoxValue>, enabled: impl ReadState<T=bool>) -> Box<dyn AnyWidget> {
        self(item.as_dyn_read(), index.as_dyn_read(), enabled.as_dyn_read())
    }
}

#[derive(Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainCheckBox<F, C, D, E> where
    F: State<T=Focus> + Clone,
    C: State<T=CheckBoxValue> + Clone,
    D: PlainCheckBoxDelegate,
    E: ReadState<T=bool>,
{
    id: WidgetId,
    #[state] focus: F,
    #[state] enabled: E,
    child: Box<dyn AnyWidget>,
    position: Position,
    dimension: Dimension,
    delegate: D,
    #[state] checked: C,
}

type DefaultPlainCheckBoxDelegate = fn(focus: Box<dyn AnyReadState<T=Focus>>, selected: Box<dyn AnyReadState<T=CheckBoxValue>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget>;

impl PlainCheckBox<Focus, CheckBoxValue, DefaultPlainCheckBoxDelegate, bool> {
    pub fn new<C: IntoState<CheckBoxValue>>(checked: C) -> PlainCheckBox<LocalState<Focus>, C::Output, DefaultPlainCheckBoxDelegate, EnabledState> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            checked.into_state(),
            focus_state,
            PlainCheckBox::default_delegate,
            enabled_state()
        )
    }

    fn default_delegate(focus: Box<dyn AnyReadState<T=Focus>>, checked: Box<dyn AnyReadState<T=CheckBoxValue>>, _enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let background_color = Map1::read_map(checked.clone(), |value| {
            match value {
                CheckBoxValue::True => EnvironmentColor::Green,
                CheckBoxValue::Indeterminate => EnvironmentColor::Blue,
                CheckBoxValue::False => EnvironmentColor::Red,
            }
        });

        let val = Map2::read_map(checked, focus, |checked, focus| {
            format!("{:?}, {:?}", *checked, focus)
        });

        ZStack::new(vec![
            Rectangle::new().fill(background_color).boxed(),
            Text::new(val).boxed(),
        ]).boxed()
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>,> PlainCheckBox<F, C, D, E> {

    pub fn delegate<D2: PlainCheckBoxDelegate>(
        self,
        delegate: D2,
    ) -> PlainCheckBox<F, C, D2, E> {
        Self::new_internal(self.checked, self.focus, delegate, self.enabled)
    }

    pub fn focused<F2: IntoState<Focus>>(self, focused: F2) -> PlainCheckBox<F2::Output, C, D, E> {
        Self::new_internal(self.checked, focused.into_state(), self.delegate, self.enabled)
    }

    pub fn enabled<E2: IntoReadState<bool>>(self, enabled: E2) -> PlainCheckBox<F, C, D, E2::Output> {
        Self::new_internal(
            self.checked,
            self.focus,
            self.delegate,
            enabled.into_read_state(),
        )
    }

    fn new_internal<F2: State<T=Focus> + Clone, C2: State<T=CheckBoxValue> + Clone, D2: PlainCheckBoxDelegate, E2: ReadState<T=bool>,>(
        checked: C2,
        focus: F2,
        delegate: D2,
        enabled: E2,
    ) -> PlainCheckBox<F2, C2, D2, E2> {
        let delegate_widget = delegate.call(focus.as_dyn_read(), checked.as_dyn_read(), enabled.as_dyn_read());

        let button = MouseArea::new(delegate_widget)
            .on_click(capture!([checked, focus, enabled], |env: &mut Environment| {
                enabled.sync(env);
                checked.sync(env);
                focus.sync(env);

                if !*enabled.value() {
                    return;
                }

                if *checked.value() == CheckBoxValue::True {
                    checked.set_value(CheckBoxValue::False);
                } else {
                    checked.set_value(CheckBoxValue::True);
                }

                if *focus.value() != Focus::Focused {
                    focus.set_value(Focus::FocusRequested);
                    println!("Focus request");
                    env.request_focus(Refocus::FocusRequest);
                }
            })).on_click_outside(a!(|env: &mut Environment, _: ModifierKey| {
                if *$focus == Focus::Focused {
                    *$focus = Focus::FocusReleased;
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone());

        let button = Box::new(button);

        let child = button;

        PlainCheckBox {
            id: WidgetId::new(),
            focus,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            delegate,
            checked,
            enabled
        }
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> Focusable for PlainCheckBox<F, C, D, E> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> CommonWidget for PlainCheckBox<F, C, D, E> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> WidgetExt for PlainCheckBox<F, C, D, E> {}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> Debug for PlainCheckBox<F, C, D, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainCheckBox")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focus)
            .field("checked", &self.checked)
            .field("enabled", &self.enabled)
            .finish()
    }
}