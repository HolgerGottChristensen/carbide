use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Border {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state]
    color: ColorState,
    border_width: u32,
}

impl Border {
    pub fn color(mut self, color: impl Into<ColorState>) -> Box<Self> {
        self.color = color.into();
        Box::new(self)
    }

    pub fn border_width(mut self, width: u32) -> Box<Self> {
        self.border_width = width;
        Box::new(self)
    }

    pub fn initialize(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Border {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            color: Color::random().into(),
            border_width: 2,
        })
    }
}

impl Layout for Border {
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

    fn position_children(&mut self) {
        let border_width = self.border_width as f64;
        let positioning = self.alignment().positioner();
        let position = Position::new(self.x() + border_width, self.y() + border_width);
        let dimension = Dimension::new(
            self.width() - border_width - border_width,
            self.height() - border_width - border_width,
        );

        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl CommonWidget for Border {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl Render for Border {
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

impl WidgetExt for Border {}
