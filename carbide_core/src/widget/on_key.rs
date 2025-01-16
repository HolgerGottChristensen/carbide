use std::fmt::{Debug, Formatter};
use carbide::widget::AnyWidget;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::event::{Key, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, ModifierKey};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

pub trait OnKeyAction: Fn(&Key, ModifierKey, &mut KeyboardEventContext) + Clone + 'static {}

impl<I> OnKeyAction for I where I: Fn(&Key, ModifierKey, &mut KeyboardEventContext) + Clone + 'static {}

type DefaultAction = fn(&Key, ModifierKey, &mut KeyboardEventContext);

#[derive(Clone, Widget)]
#[carbide_exclude(KeyboardEvent)]
pub struct OnKey<A, B, C> where A: OnKeyAction, B: OnKeyAction, C: Widget {
    #[id] id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    pressed: A,
    released: B,
}

impl OnKey<DefaultAction, DefaultAction, Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget>(child: C) -> OnKey<DefaultAction, DefaultAction, C> {
        OnKey {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            pressed: |_, _, _| {},
            released: |_, _, _| {},
        }
    }
}

impl<A: OnKeyAction, B: OnKeyAction, C: Widget> OnKey<A, B, C> {
    pub fn on_key_pressed<A2: OnKeyAction>(self, action: A2) -> OnKey<A2, B, C> {
        OnKey {
            id: self.id,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            pressed: action,
            released: self.released,
        }
    }

    pub fn on_key_released<B2: OnKeyAction>(self, action: B2) -> OnKey<A, B2, C> {
        OnKey {
            id: self.id,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            pressed: self.pressed,
            released: action,
        }
    }
}


impl<A: OnKeyAction, B: OnKeyAction, C: Widget> KeyboardEventHandler for OnKey<A, B, C> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        match event {
            KeyboardEvent::Press { key: k, modifiers: m, .. } => {
                (self.pressed)(k, *m, ctx)
            }
            KeyboardEvent::Release { key: k, modifiers: m, .. } => {
                (self.released)(k, *m, ctx)
            }
            KeyboardEvent::Ime(_) => {}
        }
    }
}

impl<A: OnKeyAction, B: OnKeyAction, C: Widget> CommonWidget for OnKey<A, B, C> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}

impl<A: OnKeyAction, B: OnKeyAction, C: Widget> Debug for OnKey<A, B, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnKey")
            .field("child", &self.child)
            .finish()
    }
}