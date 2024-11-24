use dyn_clone::clone_box;
use carbide::CommonWidgetImpl;
use carbide::draw::{Dimension, Position};
use carbide::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler};
use carbide::layout::{Layout, LayoutContext};
use carbide::state::{AnyReadState, AnyState, ReadState};
use carbide::widget::{AnySequence, AnyWidget, CommonWidget, OverlayManager, Widget, WidgetId, WidgetSync};
use crate::ControlsOverlayKey;
use crate::identifiable::AnySelectableWidget;
use crate::picker::style::menu::key_command::PopupButtonKeyCommand;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout, MouseEvent, KeyboardEvent)]
pub struct MenuStylePopupBase {
    #[id] pub id: WidgetId,
    #[state] pub position: Box<dyn AnyReadState<T=Position>>,
    #[state] pub dimension: Box<dyn AnyReadState<T=Dimension>>,
    #[state] pub hovered: Box<dyn AnyState<T=WidgetId>>,
    pub model: Box<dyn AnySequence<dyn AnySelectableWidget>>,
    pub enabled: Box<dyn AnyReadState<T=bool>>,
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

impl KeyboardEventHandler for MenuStylePopupBase {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if !*self.enabled.value() {
            OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                manager.clear()
            });
            return;
        }

        ctx.prevent_default();

        if event == PopupButtonKeyCommand::Close {
            OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                manager.clear()
            });
            return;
        }

        let id = *self.hovered.value();

        self.model.foreach_direct(&mut |child| {
            child.sync(ctx.env_stack);
        });

        if event == PopupButtonKeyCommand::Select {
            if id != WidgetId::default() {
                self.model.foreach(&mut |selectable| {
                    if selectable.as_widget().id() == id {
                        let mut state = clone_box(selectable.selection());
                        let prev = *state.value();
                        state.set_value_dyn(!prev);
                    }
                })
            }

            OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                manager.clear()
            })
        } else if event == PopupButtonKeyCommand::Next {
            let mut next = id == WidgetId::default();
            let mut already_moved = false;

            self.model.foreach(&mut |selectable| {
                if already_moved { return; }

                let selectable_id = selectable.as_widget().id();

                if next {
                    self.hovered.set_value_dyn(selectable_id);
                    already_moved = true;
                    return;
                }

                next = id == selectable_id;
            });

            if !already_moved {
                self.model.foreach(&mut |selectable| {
                    if already_moved { return; }

                    self.hovered.set_value_dyn(selectable.as_widget().id());
                    already_moved = true;
                })
            }
        } else if event == PopupButtonKeyCommand::Prev {
            let mut next = id == WidgetId::default();
            let mut already_moved = false;

            self.model.foreach_rev(&mut |selectable| {
                if already_moved { return; }

                let selectable_id = selectable.as_widget().id();

                if next {
                    self.hovered.set_value_dyn(selectable_id);
                    already_moved = true;
                    return;
                }

                next = id == selectable_id;
            });

            if !already_moved {
                self.model.foreach_rev(&mut |selectable| {
                    if already_moved { return; }

                    self.hovered.set_value_dyn(selectable.as_widget().id());
                    already_moved = true;
                })
            }
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