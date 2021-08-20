use lyon::tessellation::math::rect;
use lyon::tessellation::path::builder::BorderRadii;
use lyon::tessellation::path::traits::PathBuilder;
use lyon::tessellation::path::Winding;

use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::widget::CornerRadii;
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;
use crate::widget::types::TriangleStore;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct RoundedRectangle {
    id: Uuid,
    position: Position,
    dimension: Dimension,
    corner_radii: CornerRadii,
    #[state]
    stroke_color: ColorState,
    #[state]
    fill_color: ColorState,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    triangle_store: TriangleStore,
}

impl RoundedRectangle {
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

    pub fn new(corner_radii: CornerRadii) -> Box<RoundedRectangle> {
        Box::new(RoundedRectangle {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            corner_radii,
            stroke_color: EnvironmentColor::Blue.into(),
            fill_color: EnvironmentColor::Blue.into(),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: TriangleStore::new(),
        })
    }
}

impl CommonWidget for RoundedRectangle {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
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

impl Render for RoundedRectangle {
    fn get_primitives(&mut self, _: &mut Environment) -> Vec<Primitive> {
        let rectangle = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );

        let corner_radius = self.corner_radii;

        tessellate(self, &rectangle, &|builder, rect| {
            builder.add_rounded_rectangle(
                rect,
                &BorderRadii {
                    top_left: corner_radius.top_left as f32,
                    top_right: corner_radius.top_right as f32,
                    bottom_left: corner_radius.bottom_left as f32,
                    bottom_right: corner_radius.bottom_right as f32,
                },
                Winding::Positive,
            );
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

impl Shape for RoundedRectangle {
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

impl WidgetExt for RoundedRectangle {}