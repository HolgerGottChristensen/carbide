use lyon::geom::euclid::rect;
use lyon::tessellation::path::builder::BorderRadii;
use lyon::tessellation::path::traits::PathBuilder;
use lyon::tessellation::path::Winding;


use carbide_macro::{carbide_default_builder, carbide_default_builder2};

use crate::{Color, CommonWidgetImpl};
use crate::color::RED;
use crate::draw::{Dimension, Position};
use crate::environment::{Environment,};
use crate::environment::EnvironmentColor;
use crate::render::{Primitive, Render, RenderContext, Style};
use crate::state::{IntoReadState, ReadState, ReadStateExtNew, RState, TState};
use crate::widget::{AdvancedColor, Blur, CommonWidget, CornerRadii, Widget, WidgetExt, WidgetId, ZStack};
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::PrimitiveStore;
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct RoundedRectangle<S, F> where S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    corner_radii: CornerRadii,
    #[state]
    stroke_color: S,
    #[state]
    fill_color: F,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    triangle_store: PrimitiveStore,
}

impl RoundedRectangle<Style, Style> {
    #[carbide_default_builder2]
    pub fn new(corner_radii: impl Into<CornerRadii>) -> Box<RoundedRectangle<impl ReadState<T=Style>, impl ReadState<T=Style>>> {
        Box::new(RoundedRectangle {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            corner_radii: corner_radii.into(),
            stroke_color: EnvironmentColor::Blue.style(),
            fill_color: EnvironmentColor::Blue.style(),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: PrimitiveStore::new(),
        })
    }

    pub fn shape(corner_radii: impl Into<CornerRadii>) -> RoundedRectangle<Style, Style> {
        *RoundedRectangle::new(corner_radii)
            .fill(Style::Color(RED))
            .stroke(Style::Color(RED))
    }
}

impl<S2: ReadState<T=Style> + Clone, F2: ReadState<T=Style> + Clone> RoundedRectangle<S2, F2> {
    pub fn fill<F: IntoReadState<Style>>(self, color: F) -> Box<RoundedRectangle<S2, F::Output>> {
        Box::new(RoundedRectangle {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            corner_radii: self.corner_radii,
            stroke_color: self.stroke_color,
            fill_color: color.into_read_state(),
            style: self.style + ShapeStyle::Fill,
            stroke_style: self.stroke_style,
            triangle_store: self.triangle_store,
        })
    }

    pub fn stroke<S: IntoReadState<Style>>(self, color: S) -> Box<RoundedRectangle<S::Output, F2>> {
        Box::new(RoundedRectangle {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            corner_radii: self.corner_radii,
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
        self.fill_color = advanced_material_state.ignore_writes();
        self.stroke_color = advanced_material_state.ignore_writes();

        ZStack::new(vec![
            Blur::gaussian(10.0).clip_shape(self.clone()),
            Box::new(self),
        ])
    }*/
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> CommonWidget for RoundedRectangle<S, F> {
    CommonWidgetImpl!(self, id: self.id, position: self.position, dimension: self.dimension);
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Render for RoundedRectangle<S, F> {
    fn render(&mut self, context: &mut RenderContext, _: &mut Environment) {
        let rectangle = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );

        let corner_radius = self.corner_radii;

        tessellate(self, &rectangle.to_box2d(), &|builder, rect| {
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
        let rectangle = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );

        let corner_radius = self.corner_radii;

        tessellate(self, &rectangle.to_box2d(), &|builder, rect| {
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

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Shape for RoundedRectangle<S, F> {
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

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> WidgetExt for RoundedRectangle<S, F> {}
