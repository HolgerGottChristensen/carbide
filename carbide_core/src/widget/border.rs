use carbide_core::render::{RenderContext};
use carbide_core::state::IntoReadState;
use carbide_macro::{carbide_default_builder2};

use crate::{CommonWidgetImpl};
use crate::draw::{Dimension, Position, Rect, Color};
use crate::draw::draw_style::DrawStyle;
use crate::environment::Environment;
use crate::layout::Layout;
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::state::{ReadState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Border<W, C> where
    W: Widget + Clone,
    C: ReadState<T=Color> + Clone + 'static
{
    id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
    #[state] color: C,
    border_width: u32,
}

impl Border<Empty, Color> {
    #[carbide_default_builder2]
    pub fn new<W: Widget + Clone>(child: W) -> Box<Border<W, Color>> {
        Box::new(Border {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            color: Color::random(),
            border_width: 2,
        })
    }
}

impl<W: Widget + Clone, D: ReadState<T=Color> + Clone + 'static> Border<W, D> {
    pub fn color<C: IntoReadState<Color>>(self, color: C) -> Box<Border<W, C::Output>> {
        Box::new(Border {
            id: self.id,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            color: color.into_read_state(),
            border_width: 1,
        })
    }

    pub fn border_width(mut self, width: u32) -> Box<Border<W, D>> {
        self.border_width = width;
        Box::new(self)
    }
}

impl<W: Widget + Clone, C: ReadState<T=Color> + Clone + 'static> Layout for Border<W, C> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let border_width = self.border_width as f64;
        let dimensions = Dimension::new(
            requested_size.width - border_width - border_width,
            requested_size.height - border_width - border_width,
        );

        let child_dimensions = self.child.calculate_size(dimensions, env);

        self.dimension = Dimension::new(
            child_dimensions.width + border_width + border_width,
            child_dimensions.height + border_width + border_width,
        );

        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
        let border_width = self.border_width as f64;
        let positioning = self.alignment().positioner();
        let position = Position::new(self.x() + border_width, self.y() + border_width);
        let dimension = Dimension::new(
            self.width() - border_width - border_width,
            self.height() - border_width - border_width,
        );

        positioning(position, dimension, &mut self.child);
        self.child.position_children(env);
    }
}

impl<W: Widget + Clone, C: ReadState<T=Color> + Clone + 'static> CommonWidget for Border<W, C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<W: Widget + Clone, C: ReadState<T=Color> + Clone + 'static> Render for Border<W, C> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        let rect = Rect::new(self.position, self.dimension);
        let (l, r, b, t) = rect.l_r_b_t();

        let border_width = self.border_width as f64;

        let left_border = Rect::new(
            Position::new(l, b),
            Dimension::new(border_width, rect.height()),
        );
        let right_border = Rect::new(
            Position::new(r - border_width, b),
            Dimension::new(border_width, rect.height()),
        );

        let top_border = Rect::new(
            Position::new(l + border_width, b),
            Dimension::new(rect.width() - border_width * 2.0, border_width),
        );
        let bottom_border = Rect::new(
            Position::new(l + border_width, t - border_width),
            Dimension::new(rect.width() - border_width * 2.0, border_width),
        );

        self.foreach_child_mut(&mut |child| {
            child.render(context, env);
        });

        context.style(DrawStyle::Color(*self.color.value()), |this| {
            this.rect(left_border);
            this.rect(right_border);
            this.rect(top_border);
            this.rect(bottom_border);
        })
    }

    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, _env: &mut Environment) {
        let rect = Rect::new(self.position, self.dimension);
        let (l, r, b, t) = rect.l_r_b_t();

        let border_width = self.border_width as f64;

        let left_border = Rect::new(
            Position::new(l, b),
            Dimension::new(border_width, rect.height()),
        );
        let right_border = Rect::new(
            Position::new(r - border_width, b),
            Dimension::new(border_width, rect.height()),
        );

        let top_border = Rect::new(
            Position::new(l + border_width, b),
            Dimension::new(rect.width() - border_width * 2.0, border_width),
        );
        let bottom_border = Rect::new(
            Position::new(l + border_width, t - border_width),
            Dimension::new(rect.width() - border_width * 2.0, border_width),
        );

        let border_color = *self.color.value();
        primitives.push(Primitive {
            kind: PrimitiveKind::RectanglePrim {
                color: border_color,
            },
            bounding_box: left_border,
        });
        primitives.push(Primitive {
            kind: PrimitiveKind::RectanglePrim {
                color: border_color,
            },
            bounding_box: right_border,
        });
        primitives.push(Primitive {
            kind: PrimitiveKind::RectanglePrim {
                color: border_color,
            },
            bounding_box: top_border,
        });
        primitives.push(Primitive {
            kind: PrimitiveKind::RectanglePrim {
                color: border_color,
            },
            bounding_box: bottom_border,
        });
    }
}

impl<W: Widget + Clone, C: ReadState<T=Color> + Clone + 'static> WidgetExt for Border<W, C> {}
