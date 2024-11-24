use carbide::CommonWidgetImpl;
use carbide::draw::{Dimension, Position};
use carbide::event::{MouseEvent, MouseEventContext, MouseEventHandler};
use carbide::layout::{Layout, LayoutContext};
use carbide::state::{AnyReadState, ReadState};
use carbide::widget::{AnyWidget, CommonWidget, OverlayManager, Widget, WidgetId, WidgetSync};
use crate::ControlsOverlayKey;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout, MouseEvent)]
pub struct MenuStylePopupBase {
    #[id] pub id: WidgetId,
    #[state] pub position: Box<dyn AnyReadState<T=Position>>,
    #[state] pub dimension: Box<dyn AnyReadState<T=Dimension>>,
    pub child: Box<dyn AnyWidget>,
}

impl Layout for MenuStylePopupBase {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.sync(ctx.env_stack);

        let res = self.child.calculate_size(self.dimension(), ctx);

        res
    }
}

impl MouseEventHandler for MenuStylePopupBase {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Click(_, position, _) => {
                if !self.child.is_inside(*position) {
                    OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                        manager.clear();
                    })
                }
            }
            _ => ()
        }
    }
}

impl CommonWidget for MenuStylePopupBase {
    CommonWidgetImpl!(self, child: self.child);

    fn position(&self) -> Position {
        self.position.value().clone()
    }

    fn set_position(&mut self, position: Position) {}

    fn dimension(&self) -> Dimension {
        self.dimension.value().clone()
    }

    fn set_dimension(&mut self, dimension: Dimension) {}
}