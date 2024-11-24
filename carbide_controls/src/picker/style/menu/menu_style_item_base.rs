use std::time::Duration;

use crate::ControlsOverlayKey;
use carbide_core::draw::{Dimension, Position};
use carbide_core::event::EventId;
use carbide_core::event::{
    MouseButton, MouseEvent, MouseEventContext, MouseEventHandler
};
use carbide_core::state::AnyState;
use carbide_core::state::{ReadState, State};
use carbide_core::widget::OverlayManager;
use carbide_core::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId};
use carbide_core::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent)]
pub struct MenuStyleItemBase {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: Box<dyn AnyWidget>,
    selected: Box<dyn AnyState<T=bool>>,
    hovered: Box<dyn AnyState<T=bool>>,

    /// The ID of the event causing this to open.
    event_id: EventId,
    has_dragged: bool,
}

impl MenuStyleItemBase {
    pub fn new(child: Box<dyn AnyWidget>, selected: Box<dyn AnyState<T=bool>>, hovered: Box<dyn AnyState<T=bool>>, event_id: EventId) -> MenuStyleItemBase {
        MenuStyleItemBase {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            selected,
            event_id,
            has_dragged: false,
            hovered,
        }
    }
}

impl MouseEventHandler for MenuStyleItemBase {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Drag { button: MouseButton::Left, total_delta_xy,.. } => {
                if total_delta_xy.dist(&Position::default()) > 3.0 {
                    self.has_dragged = true;
                }
            }
            MouseEvent::Release { button: MouseButton::Left, position, press_id, duration, .. } => {
                if self.is_inside(*position) {
                    self.hovered.set_value(true);

                    // If the press was a click on the popupbutton, and the user has not dragged
                    // We dont see it as a selection.
                    if *press_id == self.event_id && *duration < Duration::from_secs_f64(0.5) && !self.has_dragged {
                        return;
                    }
                    let prev = *self.selected.value();
                    *self.selected.value_mut() = !prev;

                    OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                        manager.clear()
                    })
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
    }
}

impl CommonWidget for MenuStyleItemBase {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}