use carbide::environment::WidgetTransferAction;
use carbide::event::{KeyboardEventContext, MouseEventContext};
use carbide::layout::{Layout, LayoutContext};
use carbide::state::LocalState;
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
#[carbide_exclude(MouseEvent, KeyboardEvent, Layout)]
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
    parent_position: LocalState<Position>,
    parent_dimension: LocalState<Dimension>,
    overlay_id: Option<String>,
    #[state] model: M,
    #[state] selected: S,
    #[state] hover_model: H,
    #[state] enabled: E,
}

impl PlainPopUpButtonPopUp<bool, bool, Vec<bool>, Option<usize>, bool> {
    pub fn new<T: StateContract + PartialEq, S: State<T=T>, M: ReadState<T=Vec<T>>, H: State<T=Option<usize>>, E: ReadState<T=bool>>(child: Box<dyn AnyWidget>, hover_model: H, model: M, selected: S, enabled: E, overlay_id: Option<String>, parent_position: LocalState<Position>, parent_dimension: LocalState<Dimension>) -> PlainPopUpButtonPopUp<T, S, M, H, E> {
        PlainPopUpButtonPopUp {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            parent_position,
            parent_dimension,
            overlay_id,
            model,
            selected,
            hover_model,
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
> Layout for PlainPopUpButtonPopUp<T, S, M, H, E> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let new_requested = Dimension::new(self.parent_dimension.value().width, requested_size.height);
        let result = self.child.calculate_size(new_requested, ctx);
        self.set_dimension(result);

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let positioning = self.alignment().positioner();
        let position = *self.parent_position.value();
        let dimension = *self.parent_dimension.value();

        positioning(position, dimension, self);
        self.position = self.position
            .min(&Position::new(ctx.env.current_window_width() - self.width(), ctx.env.current_window_height() - self.height()))
            .max(&Position::new(0.0, 0.0));

        let position = self.position();
        let dimension = self.dimension();
        positioning(position, dimension, &mut self.child);
        self.child.position_children(ctx);
    }
}

impl<
    T: StateContract + PartialEq,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    H: State<T=Option<usize>>,
    E: ReadState<T=bool>,
> KeyboardEventHandler for PlainPopUpButtonPopUp<T, S, M, H, E> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if !*self.enabled.value() {
            ctx.env.transfer_widget(self.overlay_id.clone(), WidgetTransferAction::Pop);
            //self.popup_open.set_value(false);
            return;
        }

        if event == PopupButtonKeyCommand::Close {
            ctx.env.transfer_widget(self.overlay_id.clone(), WidgetTransferAction::Pop);
            //self.popup_open.set_value(false);
        } else if event == PopupButtonKeyCommand::Select {
            let value: Option<usize> = self.hover_model.value().clone();
            if let Some(h) = value {
                let value = self.model.value()[h].clone();
                self.selected.set_value(value);
            }
            ctx.env.transfer_widget(self.overlay_id.clone(), WidgetTransferAction::Pop);
            //self.popup_open.set_value(false);
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
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !*self.enabled.value() {
            ctx.env.transfer_widget(self.overlay_id.clone(), WidgetTransferAction::Pop);
            //self.popup_open.set_value(false);
            return;
        }

        match event {
            MouseEvent::Click(MouseButton::Left, mouse_position, _) => {
                if !self.is_inside(*mouse_position) {
                    ctx.env.transfer_widget(self.overlay_id.clone(), WidgetTransferAction::Pop);
                    //self.popup_open.set_value(false);
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