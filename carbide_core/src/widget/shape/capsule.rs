use lyon::tessellation::math::rect;
use lyon::tessellation::path::builder::BorderRadii;
use lyon::tessellation::path::traits::PathBuilder;
use lyon::tessellation::path::Winding;

use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;
use crate::widget::types::TriangleStore;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Capsule {
    id: Uuid,
    position: Position,
    dimension: Dimension,
    #[state]
    stroke_color: TState<AdvancedColor>,
    #[state]
    fill_color: TState<AdvancedColor>,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    triangle_store: TriangleStore,
}

impl Capsule {
    pub fn fill(mut self, color: impl Into<TState<AdvancedColor>>) -> Box<Self> {
        self.fill_color = color.into();
        self.style += ShapeStyle::Fill;
        Box::new(self)
    }

    pub fn stroke(mut self, color: impl Into<TState<AdvancedColor>>) -> Box<Self> {
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
        let advanced_material_state: RState<AdvancedColor> = material_state.into();
        self.fill_color = advanced_material_state.clone().ignore_writes();
        self.stroke_color = advanced_material_state.clone().ignore_writes();

        ZStack::new(vec![
            Blur::gaussian(10.0)
                .clip_shape(Box::new(self.clone())),
            Box::new(self),
        ])
    }

    pub fn new() -> Box<Capsule> {
        Box::new(Capsule {
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

impl CommonWidget for Capsule {
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

impl Shape for Capsule {
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

impl Render for Capsule {
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, _env: &mut Environment) {
        let rectangle = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );

        tessellate(self, &rectangle, &|builder, rect| {
            builder.add_rounded_rectangle(
                rect,
                &BorderRadii {
                    top_left: f32::MAX,
                    top_right: f32::MAX,
                    bottom_left: f32::MAX,
                    bottom_right: f32::MAX,
                },
                Winding::Positive,
            );
        });

        let fill_color =  self.fill_color.value().clone();
        let stroke_color =  self.stroke_color.value().clone();

        self.triangle_store
            .insert_primitives(primitives, fill_color, stroke_color, self.position, self.dimension);
    }
}

impl WidgetExt for Capsule {}
