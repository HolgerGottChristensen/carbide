use std::fmt::{Debug};
use carbide::layout::LayoutContext;
use carbide_core::{CommonWidgetImpl};
use carbide_core::draw::{Dimension, Position, Scalar};
use carbide_core::environment::{Environment};
use carbide_core::event::{MouseEvent, MouseEventHandler};
use carbide_core::layout::Layout;
use carbide_core::render::{Render, RenderContext};
use carbide_core::widget::{CommonWidget, Empty, AnyWidget, WidgetExt, WidgetId, Widget};
use crate::types::TooltipPosition;

const PADDING: Scalar = 8.0;

/// # A plain switch widget
/// This widget contains the basic logic for a switch component, without any styling.
/// It can be styled by setting the delegate, using the delegate method.
///
/// For a styled version, use [crate::Switch] instead.
#[derive(Clone, Debug, Widget)]
#[carbide_exclude(MouseEvent, Render, Layout)]
pub struct Help<C> where C: AnyWidget + Clone {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: C,
    help: Box<dyn AnyWidget>,
    hovered: bool,
    tooltip_position: TooltipPosition,
}

impl Help<Empty> {
    pub fn new<C: AnyWidget + Clone>(child: C, help: Box<dyn AnyWidget>) -> Help<C> {
        Self::new_internal(child, help)
    }
}

impl<C: AnyWidget + Clone> Help<C> {
    fn new_internal<C2: AnyWidget + Clone>(
        child: C2,
        help: Box<dyn AnyWidget>,
    ) -> Help<C2> {
        Help {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child,
            help,
            hovered: false,
            tooltip_position: TooltipPosition::Auto,
        }
    }
}

impl<C: AnyWidget + Clone> MouseEventHandler for Help<C> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {
        if self.is_inside(event.get_current_mouse_position()) {
            self.hovered = true;
        } else {
            self.hovered = false;
        }
    }
}

impl<C: AnyWidget + Clone> Layout for Help<C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let dimension = self.child.calculate_size(requested_size, ctx);
        self.set_dimension(dimension);

        self.help.calculate_size(Dimension::new(ctx.current_window_width(), ctx.current_window_height()), ctx);

        dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let positioning = self.alignment().positioner();
        let position = self.position();
        let dimension = self.dimension();

        positioning(position, dimension, &mut self.child);
        self.child.position_children(ctx);

        #[allow(unused_assignments)]
        let mut x = 0.0;
        #[allow(unused_assignments)]
        let mut y = 0.0;

        match self.tooltip_position {
            TooltipPosition::Auto => {
                y = self.y() - PADDING - self.help.height();

                if y < 0.0 {
                    y = self.y() + PADDING + self.height();
                }

                if y > ctx.current_window_height() - self.help.height() {
                    y = self.y() - PADDING - self.help.height();
                }

                x = self.x() + (self.width()) / 2.0 - self.help.width() / 2.0;
            }
            TooltipPosition::Top => {
                x = self.x() + (self.width()) / 2.0 - self.help.width() / 2.0;
                y = self.y() - PADDING - self.help.height();
            }
            TooltipPosition::Mouse => {
                let mouse_position = ctx.mouse_position();

                x = mouse_position.x();
                y = mouse_position.y() - PADDING - self.help.height();
            }
            TooltipPosition::Bottom => {
                x = self.x() + self.width() / 2.0 - self.help.width() / 2.0;
                y = self.y() + PADDING + self.height();
            }
            TooltipPosition::Left => {
                x = self.x() - PADDING - self.help.width();
                y = self.y() + self.height() / 2.0 - self.help.height() / 2.0;
            }
            TooltipPosition::Right => {
                x = self.x() + PADDING + self.width();
                y = self.y() + self.height() / 2.0 - self.help.height() / 2.0;
            }
        }

        x = x.max(0.0).min(ctx.current_window_width() - self.help.width());
        y = y.max(0.0).min(ctx.current_window_height() - self.help.height());

        self.help.set_position(Position::new(x, y));
        self.help.position_children(ctx);
    }
}

impl<C: AnyWidget + Clone> Render for Help<C> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.child.render(context, env);

        if self.hovered {
            context.layer(1000, |this| {
                self.help.render(this, env);
            });
        }
    }
}


impl<C: AnyWidget + Clone> CommonWidget for Help<C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<C: AnyWidget + Clone> WidgetExt for Help<C> {}

