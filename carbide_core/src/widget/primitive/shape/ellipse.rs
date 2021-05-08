use lyon::algorithms::math::{Angle, rect};
use lyon::algorithms::path::builder::PathBuilder;
use lyon::algorithms::path::geom::euclid::vec2;
use lyon::algorithms::path::Winding;
use lyon::math::point;

use crate::prelude::*;
use crate::state::environment_color::EnvironmentColor;
use crate::widget::primitive::shape::{Shape, tessellate};
use crate::widget::types::shape_style::ShapeStyle;
use crate::widget::types::stroke_style::StrokeStyle;
use crate::widget::types::triangle_store::TriangleStore;

/// A simple, non-interactive widget for drawing a single **Ellipse**.
#[derive(Debug, Clone, Widget)]
pub struct Ellipse<GS> where GS: GlobalState {
    pub id: Uuid,
    position: Point,
    dimension: Dimensions,

    #[state] stroke_color: ColorState<GS>,
    #[state] fill_color: ColorState<GS>,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    triangle_store: TriangleStore,
}

impl<GS: GlobalState> Ellipse<GS> {
    pub fn fill<C: Into<ColorState<GS>>>(mut self, color: C) -> Box<Self> {
        self.fill_color = color.into();
        self.style += ShapeStyle::Fill;
        Box::new(self)
    }

    pub fn stroke<C: Into<ColorState<GS>>>(mut self, color: C) -> Box<Self> {
        self.stroke_color = color.into();
        self.style += ShapeStyle::Stroke;
        Box::new(self)
    }

    pub fn stroke_style(mut self, line_width: f64) -> Box<Self> {
        self.stroke_style = StrokeStyle::Solid { line_width };
        self.style += ShapeStyle::Stroke;
        Box::new(self)
    }

    pub fn new() -> Box<Ellipse<GS>> {
        Box::new(Ellipse {
            id: Uuid::new_v4(),
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
            stroke_color: EnvironmentColor::Blue.into(),
            fill_color: EnvironmentColor::Blue.into(),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: TriangleStore::new(),
        })
    }
}

impl<GS: GlobalState> Layout<GS> for Ellipse<GS> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, _: &Environment<GS>) -> Dimensions {
        self.dimension = requested_size;

        requested_size
    }

    fn position_children(&mut self) {}
}

impl<GS: GlobalState> CommonWidget<GS> for Ellipse<GS> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        WidgetIter::Empty
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::Empty
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::Empty
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::Empty
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<GS: GlobalState> Render<GS> for Ellipse<GS> {
    fn get_primitives(&mut self, _: &Environment<GS>, _: &GS) -> Vec<Primitive> {
        let radii = vec2(self.get_width() as f32 / 2.0, self.get_height() as f32 / 2.0);
        let center = point(self.get_x() as f32 + radii.x, self.get_y() as f32 + radii.y);
        let rectangle = rect(self.get_x() as f32, self.get_y() as f32, self.get_width() as f32, self.get_height() as f32);

        tessellate(self, &rectangle, &|builder, _| {
            builder.add_ellipse(
                center,
                radii,
                Angle::degrees(0.0),
                Winding::Positive
            );
        });

        let mut prims = self.triangle_store.get_primitives(*self.fill_color.get_latest_value(), *self.stroke_color.get_latest_value());

        prims.extend(Rectangle::<GS>::debug_outline(Rect::new(self.position, self.dimension), 1.0));

        return prims;
    }
}

impl<GS: GlobalState> Shape<GS> for Ellipse<GS> {
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

impl<GS: GlobalState> WidgetExt<GS> for Ellipse<GS> {}
