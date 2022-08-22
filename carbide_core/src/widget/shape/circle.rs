use lyon::algorithms::math::rect;
use lyon::algorithms::path::builder::PathBuilder;
use lyon::algorithms::path::Winding;
use lyon::math::point;
use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::widget::shape::{tessellate, Shape};
use crate::widget::types::PrimitiveStore;
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;
use crate::{Color, CommonWidgetImpl};
use crate::environment::Environment;
use crate::environment::EnvironmentColor;
use crate::layout::Layout;
use crate::render::{Primitive, Render};
use crate::state::{ReadState, RState, TState};
use crate::widget::{AdvancedColor, Blur, CommonWidget, Widget, WidgetExt, WidgetId, ZStack};

/// A simple, non-interactive widget for drawing a single **Ellipse**.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Circle {
    pub id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state]
    stroke_color: TState<AdvancedColor>,
    #[state]
    fill_color: TState<AdvancedColor>,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    triangle_store: PrimitiveStore,
}

impl Circle {
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

    pub fn material(mut self, material: impl Into<TState<Color>>) -> Box<ZStack> {
        let material_state = material.into();
        let advanced_material_state: RState<AdvancedColor> = material_state.into();
        self.fill_color = advanced_material_state.clone().ignore_writes();
        self.stroke_color = advanced_material_state.clone().ignore_writes();

        ZStack::new(vec![
            Blur::gaussian(10.0).clip_shape(Box::new(self.clone())),
            Box::new(self),
        ])
    }

    #[carbide_default_builder]
    pub fn new() -> Box<Circle> {
        Box::new(Circle {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            stroke_color: EnvironmentColor::Blue.into(),
            fill_color: EnvironmentColor::Blue.into(),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: PrimitiveStore::new(),
        })
    }
}

CommonWidgetImpl!(Circle, self, id: self.id, position: self.position, dimension: self.dimension);

impl Layout for Circle {
    fn calculate_size(&mut self, requested_size: Dimension, _: &mut Environment) -> Dimension {
        let min_dimension = requested_size.width.min(requested_size.height);
        self.dimension = Dimension::new(min_dimension, min_dimension);

        requested_size
    }
}

impl Render for Circle {
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, _env: &mut Environment) {
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

        let fill_color = self.fill_color.value().clone();
        let stroke_color = self.stroke_color.value().clone();

        self.triangle_store.insert_primitives(
            primitives,
            fill_color,
            stroke_color,
            self.position,
            self.dimension,
        );
    }
}

impl Shape for Circle {
    fn get_triangle_store_mut(&mut self) -> &mut PrimitiveStore {
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
