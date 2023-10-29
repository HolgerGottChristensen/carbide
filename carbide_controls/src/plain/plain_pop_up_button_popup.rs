use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{
    KeyboardEvent, KeyboardEventHandler, MouseButton, MouseEvent, MouseEventHandler,
};
use carbide_core::state::{AnyState, ReadState, State, StateContract};
use carbide_core::widget::{CommonWidget, AnyWidget, WidgetExt, WidgetId, Widget};

use crate::plain::plain_pop_up_button::PopupButtonKeyCommand;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent)]
pub struct PlainPopUpButtonPopUp<T, S, M, H, E> where
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
{
    id: WidgetId,
    child: Box<dyn AnyWidget>,
    position: Position,
    dimension: Dimension,
    #[state] model: M,
    #[state] selected: S,
    #[state] hover_model: H,
    #[state] enabled: E,
    #[state] popup_open: Box<dyn AnyState<T=bool>>,
}

impl PlainPopUpButtonPopUp<bool, bool, Vec<bool>, Option<usize>, bool> {
    pub fn new<T: StateContract + PartialEq, S: State<T=T>, M: ReadState<T=Vec<T>>, H: State<T=Option<usize>>, E: ReadState<T=bool>>(child: Box<dyn AnyWidget>, hover_model: H, popup_open: Box<dyn AnyState<T=bool>>, model: M, selected: S, enabled: E) -> PlainPopUpButtonPopUp<T, S, M, H, E> {
        PlainPopUpButtonPopUp {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            model,
            selected,
            hover_model,
            popup_open,
            enabled,
        }
    }
}

impl<
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
> KeyboardEventHandler for PlainPopUpButtonPopUp<T, S, M, H, E> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, _env: &mut Environment) {
        if !*self.enabled.value() {
            self.popup_open.set_value(false);
            return;
        }

        if event == PopupButtonKeyCommand::Close {
            self.popup_open.set_value(false);
        } else if event == PopupButtonKeyCommand::Select {
            let value: Option<usize> = self.hover_model.value().clone();
            if let Some(h) = value {
                let value = self.model.value()[h].clone();
                self.selected.set_value(value);
            }
            self.popup_open.set_value(false);
        } else if event == PopupButtonKeyCommand::Next {
            let value: Option<usize> = self.hover_model.value().clone();
            if let Some(h) = value {
                let new = if h == self.model.value().len() - 1 { 0 } else { h + 1 };
                self.hover_model.set_value(Some(new));
            } else {
                self.hover_model.set_value(Some(0));
            }
        } else if event == PopupButtonKeyCommand::Prev {
            let value: Option<usize> = self.hover_model.value().clone();
            if let Some(h) = value {
                let new = if h == 0 { self.model.value().len() - 1 } else { h - 1 };
                self.hover_model.set_value(Some(new));
            } else {
                self.hover_model.set_value(Some(self.model.value().len() - 1));
            }
        }
    }
}

impl<
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
> MouseEventHandler for PlainPopUpButtonPopUp<T, S, M, H, E> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {
        if !*self.enabled.value() {
            self.popup_open.set_value(false);
            return;
        }

        match event {
            MouseEvent::Click(MouseButton::Left, mouse_position, _) => {
                if !self.is_inside(*mouse_position) {
                    self.popup_open.set_value(false);
                }
            }
            _ => (),
        }
    }
}

impl<
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
> CommonWidget for PlainPopUpButtonPopUp<T, S, M, H, E> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
> WidgetExt for PlainPopUpButtonPopUp<T, S, M, H, E> {}