use carbide::update::UpdateContext;

use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::{Environment, WidgetTransferAction};
use crate::event::{KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler};
use crate::event::Event;
use crate::layout::{Layout, LayoutContext};
use crate::render::Render;
use crate::update::Update;
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, MouseEvent, KeyboardEvent, OtherEvent, Update)]
pub struct Overlay<C> where C: Widget {
    id: WidgetId,
    child: C,
    overlay_id: &'static str,
    overlay: Option<Box<dyn AnyWidget>>,
    position: Position,
    dimension: Dimension,
    steal_events_when_some: bool,
}

impl Overlay<Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget>(overlay_id: &'static str, child: C) -> Overlay<C> {
        Overlay {
            id: WidgetId::new(),
            child,
            overlay_id,
            overlay: None,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            steal_events_when_some: false,
        }
    }
}

impl<C: Widget> Overlay<C> {
    pub fn steal_events(mut self) -> Overlay<C> {
        self.steal_events_when_some = true;
        self
    }
}

impl<C: Widget> Update for Overlay<C> {
    fn process_update(&mut self, ctx: &mut UpdateContext) {
        if let Some(action) = ctx.env.transferred_widget(&Some(self.overlay_id.to_string())) {
            match action {
                WidgetTransferAction::Push(widget) => {
                    self.overlay = Some(widget);
                }
                WidgetTransferAction::Pop => {
                    self.overlay = None;
                }
                _ => (),
            }
        }

        if let Some(overlay) = &mut self.overlay {
            overlay.process_update(ctx);
        }

        self.child.process_update(ctx);
    }
}

impl<C: Widget> MouseEventHandler for Overlay<C> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if let Some(overlay) = &mut self.overlay {
            overlay.process_mouse_event(event, ctx);
            if self.steal_events_when_some {
                return;
            }
        }

        self.child.process_mouse_event(event, ctx);
    }
}

impl<C: Widget> KeyboardEventHandler for Overlay<C> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if let Some(overlay) = &mut self.overlay {
            overlay.process_keyboard_event(event, ctx);
            if self.steal_events_when_some {
                return;
            }
        }

        self.child.process_keyboard_event(event, ctx);
    }
}

impl<C: Widget> OtherEventHandler for Overlay<C> {
    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        if let Some(overlay) = &mut self.overlay {
            overlay.process_other_event(event, ctx);
            if self.steal_events_when_some {
                return;
            }
        }

        self.child.process_other_event(event, ctx);
    }
}

impl<C: AnyWidget + Clone> Layout for Overlay<C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        if let Some(overlay) = &mut self.overlay {
            overlay.calculate_size(requested_size, ctx);
        }

        self.dimension = self.child.calculate_size(requested_size, ctx);
        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let positioning = self.alignment().positioner();
        let position = self.position();
        let dimension = self.dimension();

        positioning(position, dimension, &mut self.child);
        self.child.position_children(ctx);

        if let Some(overlay) = &mut self.overlay {
            positioning(position, dimension, overlay);
            overlay.position_children(ctx);
        }
    }
}

impl<C: AnyWidget + Clone> CommonWidget for Overlay<C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 0);
}

impl<C: AnyWidget + Clone> Render for Overlay<C> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.child.render(context, env);

        if let Some(overlay) = &mut self.overlay {
            overlay.render(context, env)
        }
    }
}

impl<C: AnyWidget + Clone> WidgetExt for Overlay<C> {}
