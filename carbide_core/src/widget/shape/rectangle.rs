use std::thread::sleep;
use std::time::Duration;
use lyon::algorithms::math::rect;
use lyon::algorithms::path::builder::PathBuilder;
use lyon::algorithms::path::Winding;
use crate::Color::Rgba;

use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;
use crate::widget::types::TriangleStore;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Rectangle {
    id: Uuid,
    position: Position,
    dimension: Dimension,
    #[state]
    fill_color: TState<AdvancedColor>,
    #[state]
    stroke_color: ColorState,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    // Store the triangles for the border
    triangle_store: TriangleStore,
}

impl Rectangle {
    pub fn fill(mut self, color: impl Into<TState<AdvancedColor>>) -> Box<Self> {
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
        let material_state = material.into();
        let advanced_material_state: RState<AdvancedColor> = material_state.clone().into();
        self.fill_color = advanced_material_state.ignore_writes();
        self.stroke_color = material_state;

        ZStack::new(vec![
            Blur::gaussian(10.0),
            Box::new(self),
        ])
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
                kind: PrimitiveKind::RectanglePrim {
                    color: border_color.clone(),
                },
                rect: left_border,
            },
            Primitive {
                kind: PrimitiveKind::RectanglePrim {
                    color: border_color.clone(),
                },
                rect: right_border,
            },
            Primitive {
                kind: PrimitiveKind::RectanglePrim {
                    color: border_color.clone(),
                },
                rect: top_border,
            },
            Primitive {
                kind: PrimitiveKind::RectanglePrim {
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

    pub fn new() -> Box<Rectangle> {
        Box::new(Rectangle {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            fill_color: EnvironmentColor::Blue.into(),
            stroke_color: EnvironmentColor::Blue.into(),
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
        WidgetIter::Empty
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
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

impl Render for Rectangle {
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, _env: &mut Environment) {
        let rect = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );
        tessellate(self, &rect, &|builder, rectangle| {
            builder.add_rectangle(rectangle, Winding::Positive)
        });

        let fill_color =  self.fill_color.value().clone();

        self.triangle_store
            .insert_primitives(primitives, fill_color, *self.stroke_color.value(), self.position, self.dimension);
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
