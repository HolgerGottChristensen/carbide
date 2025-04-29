use std::fmt::{Debug, Formatter};
use dyn_clone::DynClone;
use crate::draw::{CompositeDrawShape, DrawShape};
use crate::draw::stroke::StrokeAlignment;
use crate::widget::AnyShape;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Color, Dimension, DrawStyle, Position, Rect};
use crate::draw::stroke::StrokeOptions;
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{CommonWidget, Empty, Rectangle, Shape, Widget, WidgetId};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Border<W, C> where
    W: Widget,
    C: ReadState<T=Color>
{
    #[id] id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
    #[state] color: C,
    border_width: u32,
}

impl Border<Empty, Color> {
    #[carbide_default_builder2]
    pub fn new<W: Widget>(child: W) -> Border<W, Color> {
        Border {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            color: Color::random(),
            border_width: 2,
        }
    }
}

impl<W: Widget, D: ReadState<T=Color>> Border<W, D> {
    pub fn color<C: IntoReadState<Color>>(self, color: C) -> Border<W, C::Output> {
        Border {
            id: self.id,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            color: color.into_read_state(),
            border_width: 1,
        }
    }

    pub fn border_width(mut self, width: u32) -> Border<W, D> {
        self.border_width = width;
        self
    }
}

impl<W: Widget, C: ReadState<T=Color>> Layout for Border<W, C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let border_width = self.border_width as f64;
        let dimensions = Dimension::new(
            requested_size.width - border_width - border_width,
            requested_size.height - border_width - border_width,
        );

        let child_dimensions = self.child.calculate_size(dimensions, ctx);

        self.dimension = Dimension::new(
            child_dimensions.width + border_width + border_width,
            child_dimensions.height + border_width + border_width,
        );

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let border_width = self.border_width as f64;
        let alignment = self.alignment();
        let position = Position::new(self.x() + border_width, self.y() + border_width);
        let dimension = Dimension::new(
            self.width() - border_width - border_width,
            self.height() - border_width - border_width,
        );

        self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
        self.child.position_children(ctx);
    }
}

impl<W: Widget, C: ReadState<T=Color>> CommonWidget for Border<W, C> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}

impl<W: Widget, C: ReadState<T=Color>> Render for Border<W, C> {
    fn render(&mut self, context: &mut RenderContext) {
        #[derive(Debug, Clone)]
        struct BorderShape {
            rect: Rect,
        }

        impl From<BorderShape> for DrawShape {
            fn from(value: BorderShape) -> Self {
                DrawShape::Rectangle(value.rect)
            }
        }

        let rect = Rect::new(self.position, self.dimension);

        self.foreach_child_mut(&mut |child| {
            child.render(context);
        });

        context.style(DrawStyle::Color(*self.color.value()), |context| {
            context.shape(
                BorderShape { rect },
                StrokeOptions::default()
                    .with_stroke_width(self.border_width as f64)
                    .with_alignment(StrokeAlignment::Positive)
            );
        })
    }
}
