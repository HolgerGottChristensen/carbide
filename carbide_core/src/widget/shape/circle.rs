use lyon::algorithms::math::rect;
use lyon::algorithms::path::builder::PathBuilder;
use lyon::algorithms::path::Winding;
use lyon::math::point;

use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;
use crate::widget::types::TriangleStore;

/// A simple, non-interactive widget for drawing a single **Ellipse**.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Circle {
    pub id: Uuid,
    position: Position,
    dimension: Dimension,
    #[state]
    stroke_color: ColorState,
    #[state]
    fill_color: ColorState,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    triangle_store: TriangleStore,
}

impl Circle {
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
            Blur::gaussian(10.0)
                .clip_shape(Box::new(self.clone())),
            Box::new(self),
        ])
    }

    pub fn new() -> Box<Circle> {
        Box::new(Circle {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            stroke_color: EnvironmentColor::Blue.into(),
            fill_color: EnvironmentColor::Blue.into(),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: TriangleStore::new(),
        })
    }
}

impl CommonWidget for Circle {
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

impl Layout for Circle {
    fn calculate_size(&mut self, requested_size: Dimension, _: &mut Environment) -> Dimension {
        let min_dimension = requested_size.width.min(requested_size.height);
        self.dimension = Dimension::new(min_dimension, min_dimension);

        requested_size
    }
}

impl Render for Circle {
    fn get_primitives(&mut self, _: &mut Environment) -> Vec<Primitive> {
        let radius = self.width() as f32 / 2.0;
        let center = point(self.x() as f32 + radius, self.y() as f32 + radius);
        let rectangle = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );

        tessellate(self, &rectangle, &|builder, _| {
            builder.add_circle(center, radius, Winding::Positive);
        });

        let mut prims = self
            .triangle_store
            .get_primitives(*self.fill_color.value(), *self.stroke_color.value());

        prims.extend(Rectangle::debug_outline(
            Rect::new(self.position, self.dimension),
            1.0,
        ));

        return prims;
    }
}

impl Shape for Circle {
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

impl WidgetExt for Circle {}
