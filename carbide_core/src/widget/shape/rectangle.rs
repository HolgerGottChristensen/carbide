use lyon::algorithms::math::rect;
use lyon::algorithms::path::builder::PathBuilder;
use lyon::algorithms::path::Winding;

use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;
use crate::widget::types::TriangleStore;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Rectangle {
    id: Uuid,
    children: Vec<Box<dyn Widget>>,
    position: Position,
    dimension: Dimension,
    #[state]
    fill_color: ColorState,
    #[state]
    stroke_color: ColorState,
    shrink_to_fit: bool,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    // Store the triangles for the border
    triangle_store: TriangleStore,
}

impl Rectangle {
    pub fn fill<C: Into<ColorState>>(mut self, color: C) -> Box<Self> {
        self.fill_color = color.into();
        self.style += ShapeStyle::Fill;
        Box::new(self)
    }

    pub fn stroke<C: Into<ColorState>>(mut self, color: C) -> Box<Self> {
        self.stroke_color = color.into();
        self.style += ShapeStyle::Stroke;
        Box::new(self)
    }

    pub fn stroke_style(mut self, line_width: f64) -> Box<Self> {
        self.stroke_style = StrokeStyle::Solid { line_width };
        self.style += ShapeStyle::Stroke;
        Box::new(self)
    }

    pub fn material<C: Into<ColorState>>(mut self, material: C) -> Box<ZStack> {
        let material = material.into();
        self.fill_color = material.clone();
        self.stroke_color = material;

        ZStack::new(vec![
            Blur::gaussian(10.0),
            Box::new(self),
        ])
    }

    pub fn shrink_to_fit(mut self) -> Box<Self> {
        self.shrink_to_fit = true;
        Box::new(self)
    }

    pub fn position(mut self, position: Position) -> Box<Self> {
        self.position = position;
        Box::new(self)
    }

    //#[cfg(not(feature = "debug-outline"))]
    pub fn debug_outline(_rect: Rect, _width: Scalar) -> Vec<Primitive> {
        vec![]
    }

    pub fn debug_outline_special(rect: Rect, border_width: Scalar) -> Vec<Primitive> {
        let (l, r, b, t) = rect.l_r_b_t();

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

        let border_color = Color::Rgba(0.0 / 255.0, 255.0 / 255.0, 251.0 / 255.0, 1.0); //Color::random();
        vec![
            Primitive {
                kind: PrimitiveKind::Rectangle {
                    color: border_color.clone(),
                },
                rect: left_border,
            },
            Primitive {
                kind: PrimitiveKind::Rectangle {
                    color: border_color.clone(),
                },
                rect: right_border,
            },
            Primitive {
                kind: PrimitiveKind::Rectangle {
                    color: border_color.clone(),
                },
                rect: top_border,
            },
            Primitive {
                kind: PrimitiveKind::Rectangle {
                    color: border_color.clone(),
                },
                rect: bottom_border,
            },
        ]
    }

    //#[cfg(feature = "debug-outline")]
    /*pub fn debug_outline(rect: Rect, width: Scalar) -> Vec<Primitive> {
        let (l, r, b, t) = rect.l_r_b_t();

        let left_border = Rect::new([l,b], [width, rect.h()]);
        let right_border = Rect::new([r-width,b], [width, rect.h()]);
        let top_border = Rect::new([l+width,b], [rect.w()-width*2.0, width]);
        let bottom_border = Rect::new([l+width,t-width], [rect.w()-width*2.0, width]);

        let border_color = Color::Rgba(0.0 / 255.0, 255.0 / 255.0, 251.0 / 255.0, 1.0);//Color::random();
        vec![
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: left_border
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: right_border
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: top_border
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: bottom_border
            },
        ]
    }*/

    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<Rectangle> {
        Box::new(Rectangle {
            id: Uuid::new_v4(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            fill_color: EnvironmentColor::Blue.into(),
            stroke_color: EnvironmentColor::Blue.into(),
            shrink_to_fit: false,
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: TriangleStore::new(),
        })
    }
}

impl CommonWidget for Rectangle {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        self.children.iter().rfold(WidgetIter::Empty, |acc, x| {
            if x.flag() == Flags::PROXY {
                WidgetIter::Multi(Box::new(x.children()), Box::new(acc))
            } else {
                WidgetIter::Single(x, Box::new(acc))
            }
        })
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.flag() == Flags::PROXY {
                    WidgetIterMut::Multi(Box::new(x.children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .fold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn flexibility(&self) -> u32 {
        0
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

impl Layout for Rectangle {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let mut max_child_size = Dimension::new(0.0, 0.0);

        for child in &mut self.children {
            let child_size = child.calculate_size(requested_size, env);

            if child_size.width > max_child_size.width {
                max_child_size.width = child_size.width;
            }

            if child_size.height > max_child_size.height {
                max_child_size.height = child_size.height;
            }
        }

        if self.shrink_to_fit {
            self.dimension = max_child_size;
        } else {
            self.dimension = requested_size;
        }

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = self.alignment().positioner();
        let position = self.position;
        let dimension = self.dimension;

        for child in &mut self.children {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl Render for Rectangle {
    fn get_primitives(&mut self, _: &mut Environment) -> Vec<Primitive> {
        let rect = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );
        tessellate(self, &rect, &|builder, rectangle| {
            builder.add_rectangle(rectangle, Winding::Positive)
        });

        let prims = self
            .triangle_store
            .get_primitives(*self.fill_color.value(), *self.stroke_color.value());

        return prims;
    }
}

impl Shape for Rectangle {
    fn get_triangle_store_mut(&mut self) -> &mut TriangleStore {
        &mut self.triangle_store
    }

    fn get_stroke_style(&self) -> StrokeStyle {
        self.stroke_style.clone()
    }

    fn get_shape_style(&self) -> ShapeStyle {
        self.style.clone()
    }
}

impl WidgetExt for Rectangle {}
