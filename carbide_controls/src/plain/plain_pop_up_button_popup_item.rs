use std::time::Duration;

use carbide::event::EventId;
use carbide::state::AnyState;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::WidgetTransferAction;
use carbide_core::event::{
    MouseButton, MouseEvent, MouseEventContext, MouseEventHandler
};
use carbide_core::state::{ReadState, State, StateContract};
use carbide_core::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent)]
pub struct PlainPopUpButtonPopUpItem<T, S> where
    T: StateContract,
    S: State<T=T>,
{
    id: WidgetId,
    child: Box<dyn AnyWidget>,
    position: Position,
    dimension: Dimension,

    selected: S,
    item: Box<dyn AnyState<T=T>>,
    hovered: Box<dyn AnyState<T=bool>>,
    overlay_id: Option<String>,

    /// The ID of the event causing this to open.
    event_id: EventId,
    has_dragged: bool,
}

impl<T: StateContract, S: State<T=T>> PlainPopUpButtonPopUpItem<T, S> {
    pub fn new(child: Box<dyn AnyWidget>, selected: S, item: Box<dyn AnyState<T=T>>, hovered: Box<dyn AnyState<T=bool>>, overlay_id: Option<String>, event_id: EventId) -> PlainPopUpButtonPopUpItem<T, S> {
        PlainPopUpButtonPopUpItem {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            overlay_id,
            selected,
            event_id,
            item,
            has_dragged: false,
            hovered,
        }
    }
}

impl<T: StateContract, S: State<T=T>> MouseEventHandler for PlainPopUpButtonPopUpItem<T, S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Drag { button: MouseButton::Left, .. } => {
                self.has_dragged = true;
            }
            MouseEvent::Release { button: MouseButton::Left, position, press_id, duration, .. } => {
                if self.is_inside(*position) {
                    self.hovered.set_value(true);

                    // If the press was a click on the popupbutton, and the user has not dragged
                    // We dont see it as a selection.
                    if *press_id == self.event_id && *duration < Duration::from_secs_f64(0.5) && !self.has_dragged {
                        return;
                    }

                    self.selected.set_value(self.item.value().clone());
                    ctx.env.transfer_widget(self.overlay_id.clone(), WidgetTransferAction::Pop);
                }
            }
            MouseEvent::Move { to, .. } => {
                if *self.hovered.value() {
                    if !self.is_inside(*to) {
                        self.hovered.set_value(false);
                        self.hovered.set_value(false);
                    }
                } else {
                    if self.is_inside(*to) {
                        self.hovered.set_value(true);
                    }
                }
            }
            _ => ()
        }
        /*if !*self.enabled.value() {
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
        }*/
    }
}

impl<T: StateContract, S: State<T=T>> CommonWidget for PlainPopUpButtonPopUpItem<T, S> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}