use lyon::algorithms::math::Angle;
use lyon::algorithms::path::builder::PathBuilder;
use lyon::algorithms::path::geom::euclid::vec2;
use lyon::algorithms::path::Winding;
use lyon::geom::euclid::rect;
use lyon::math::point;


use carbide_macro::{carbide_default_builder, carbide_default_builder2};

use crate::{Color, CommonWidgetImpl};
use crate::draw::{Dimension, Position};
use crate::environment::{Environment, EnvironmentColorState};
use crate::environment::EnvironmentColor;
use crate::render::{Primitive, Render, RenderContext, Style};
use crate::state::{ReadState, RState, TState, IntoReadState};
use crate::widget::{AdvancedColor, Blur, CommonWidget, Widget, WidgetExt, WidgetId, ZStack};
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::PrimitiveStore;
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;

/// A simple, non-interactive widget for drawing a single **Ellipse**.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Ellipse<S, F> where S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone {
    pub id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state]
    stroke_color: S,
    #[state]
    fill_color: F,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    triangle_store: PrimitiveStore,
}

impl Ellipse<EnvironmentColorState, EnvironmentColorState> {
    #[carbide_default_builder2]
    pub fn new() -> Box<Self> {
        Box::new(Ellipse {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            stroke_color: <EnvironmentColor as IntoReadState<Style>>::into_read_state(EnvironmentColor::Blue),
            fill_color: <EnvironmentColor as IntoReadState<Style>>::into_read_state(EnvironmentColor::Blue),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: PrimitiveStore::new(),
        })
    }
}

impl<S2: ReadState<T=Style> + Clone, F2: ReadState<T=Style> + Clone> Ellipse<S2, F2> {
    pub fn fill<F: IntoReadState<Style>>(self, color: F) -> Box<Ellipse<S2, F::Output>> {
        Box::new(Ellipse {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            stroke_color: self.stroke_color,
            fill_color: color.into_read_state(),
            style: self.style + ShapeStyle::Fill,
            stroke_style: self.stroke_style,
            triangle_store: self.triangle_store,
        })
    }

    pub fn stroke<S: IntoReadState<Style>>(self, color: S) -> Box<Ellipse<S::Output, F2>> {
        Box::new(Ellipse {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            stroke_color: color.into_read_state(),
            fill_color: self.fill_color,
            style: self.style + ShapeStyle::Stroke,
            stroke_style: self.stroke_style,
            triangle_store: self.triangle_store,
        })
    }

    pub fn stroke_style(mut self, line_width: f64) -> Box<Self> {
        self.stroke_style = StrokeStyle::Solid { line_width };
        self.style += ShapeStyle::Stroke;
        Box::new(self)
    }

    /*pub fn material(mut self, material: impl Into<TState<Color>>) -> Box<ZStack> {
        let material_state = material.into();
        let advanced_material_state: RState<Style> = material_state.into();
        self.fill_color = advanced_material_state.clone().ignore_writes();
        self.stroke_color = advanced_material_state.clone().ignore_writes();

        ZStack::new(vec![
            Blur::gaussian(10.0).clip_shape(self.clone()),
            Box::new(self),
        ])
    }*/
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> CommonWidget for Ellipse<S, F> {
    CommonWidgetImpl!(self, id: self.id, position: self.position, dimension: self.dimension);
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Render for Ellipse<S, F> {
    fn render(&mut self, context: &mut RenderContext, _: &mut Environment) {
        let radii = vec2(self.width() as f32 / 2.0, self.height() as f32 / 2.0);
        let center = point(self.x() as f32 + radii.x, self.y() as f32 + radii.y);
        let rectangle = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );

        tessellate(self, &rectangle.to_box2d(), &|builder, _| {
            builder.add_ellipse(center, radii, Angle::degrees(0.0), Winding::Positive);
        });

        if self.triangle_store.fill_triangles.len() > 0 {
            context.style(self.fill_color.value().convert(self.position, self.dimension), |this| {
                this.geometry(&self.triangle_store.fill_triangles)
            })
        }

        if self.triangle_store.stroke_triangles.len() > 0 {
            context.style(self.stroke_color.value().convert(self.position, self.dimension), |this| {
                this.geometry(&self.triangle_store.stroke_triangles)
            })
        }
    }

    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, _env: &mut Environment) {
        let radii = vec2(self.width() as f32 / 2.0, self.height() as f32 / 2.0);
        let center = point(self.x() as f32 + radii.x, self.y() as f32 + radii.y);
        let rectangle = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );

        tessellate(self, &rectangle.to_box2d(), &|builder, _| {
            builder.add_ellipse(center, radii, Angle::degrees(0.0), Winding::Positive);
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

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Shape for Ellipse<S, F> {
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

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> WidgetExt for Ellipse<S, F> {}
