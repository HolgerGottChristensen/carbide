use std::fmt::{Debug};
use carbide::event::MouseEventContext;
use carbide::layout::LayoutContext;
use carbide_core::{CommonWidgetImpl};
use carbide_core::draw::{Dimension, Position, Scalar};
use carbide_core::event::{MouseEvent, MouseEventHandler};
use carbide_core::layout::Layout;
use carbide_core::render::{Render, RenderContext};
use carbide_core::widget::{CommonWidget, Empty, AnyWidget, WidgetExt, WidgetId, Widget};

const PADDING: Scalar = 8.0;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(MouseEvent, Render, Layout)]
pub struct Help<C> where C: Widget {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: C,
    help: Box<dyn AnyWidget>,
    hovered: bool,
    tooltip_position: TooltipPosition,
}

impl Help<Empty> {
    pub fn new<C: Widget>(child: C, help: Box<dyn AnyWidget>) -> Help<C> {
        Self::new_internal(child, help)
    }
}

impl<C: Widget> Help<C> {
    fn new_internal<C2: Widget>(
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

impl<C: Widget> MouseEventHandler for Help<C> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _ctx: &mut MouseEventContext) {
        if self.is_inside(event.get_current_mouse_position()) {
            self.hovered = true;
        } else {
            self.hovered = false;
        }
    }
}

impl<C: Widget> Layout for Help<C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let dimension = self.child.calculate_size(requested_size, ctx);
        self.set_dimension(dimension);

        //self.help.calculate_size(Dimension::new(ctx.env.current_window_width(), ctx.env.current_window_height()), ctx);

        //dimension
        todo!()
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let alignment = self.alignment();
        let position = self.position();
        let dimension = self.dimension();

        self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
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

                /*if y > ctx.env.current_window_height() - self.help.height() {
                    y = self.y() - PADDING - self.help.height();
                }*/

                x = self.x() + (self.width()) / 2.0 - self.help.width() / 2.0;
            }
            TooltipPosition::Top => {
                x = self.x() + (self.width()) / 2.0 - self.help.width() / 2.0;
                y = self.y() - PADDING - self.help.height();
            }
            TooltipPosition::Mouse => {
                let mouse_position = ctx.env.mouse_position();

                x = mouse_position.x;
                y = mouse_position.y - PADDING - self.help.height();
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

        //x = x.max(0.0).min(ctx.env.current_window_width() - self.help.width());
        //y = y.max(0.0).min(ctx.env.current_window_height() - self.help.height());

        self.help.set_position(Position::new(x, y));
        self.help.position_children(ctx);
    }
}

impl<C: Widget> Render for Help<C> {
    fn render(&mut self, context: &mut RenderContext) {
        self.child.render(context);

        if self.hovered {
            self.help.render(context);
        }
    }
}


impl<C: Widget> CommonWidget for Help<C> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}

#[derive(Clone, Debug, PartialEq)]
pub enum TooltipPosition {
    Auto,
    Mouse,
    Top,
    Bottom,
    Left,
    Right,
}