use lyon::algorithms::path::Winding;
use lyon::geom::euclid::rect;

use carbide_core::render::{RenderContext, Style};
use carbide_core::state::StateSync;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Color, Dimension, Position};
use crate::environment::Environment;
use crate::environment::EnvironmentColor;
use crate::render::Render;
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Blur, CommonWidget, Widget, WidgetExt, WidgetId, ZStack};
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::PrimitiveStore;
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Rectangle<S, F> where S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state]
    fill_color: F,
    #[state]
    stroke_color: S,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    // Store the triangles for the border
    triangle_store: PrimitiveStore,
}

impl Rectangle<Style, Style> {
    #[carbide_default_builder2]
    pub fn new() -> Rectangle<impl ReadState<T=Style>, impl ReadState<T=Style>> {
        Rectangle {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            stroke_color: EnvironmentColor::Blue.style(),
            fill_color: EnvironmentColor::Blue.style(),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: PrimitiveStore::new(),
        }
    }
}

impl<S2: ReadState<T=Style> + Clone, F2: ReadState<T=Style> + Clone> Rectangle<S2, F2> {
    pub fn fill<F: IntoReadState<Style>>(self, color: F) -> Rectangle<S2, F::Output> {
        Rectangle {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            stroke_color: self.stroke_color,
            fill_color: color.into_read_state(),
            style: self.style + ShapeStyle::Fill,
            stroke_style: self.stroke_style,
            triangle_store: self.triangle_store,
        }
    }

    pub fn stroke<S: IntoReadState<Style>>(self, color: S) -> Rectangle<S::Output, F2> {
        Rectangle {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            stroke_color: color.into_read_state(),
            fill_color: self.fill_color,
            style: self.style + ShapeStyle::Stroke,
            stroke_style: self.stroke_style,
            triangle_store: self.triangle_store,
        }
    }

    pub fn stroke_style(mut self, line_width: f64) -> Self {
        self.stroke_style = StrokeStyle::Solid { line_width };
        self.style += ShapeStyle::Stroke;
        self
    }

    pub fn material<M: IntoReadState<Color>>(self, material: M) -> ZStack<(Blur, Rectangle<S2, impl ReadState<T=Style> + Clone>)> {
        let comp = self.fill(material.clone().into_read_state());
        ZStack::new((Blur::gaussian(10.0), comp))
    }

    pub fn position(mut self, position: Position) -> Box<Self> {
        self.position = position;
        Box::new(self)
    }
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> CommonWidget for Rectangle<S, F> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension);
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Render for Rectangle<S, F> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {

        self.capture_state(env);

        let rect = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );

        tessellate(self, &rect.to_box2d(), &|builder, rectangle| {
            builder.add_rectangle(rectangle, Winding::Positive)
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
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Shape for Rectangle<S, F> {
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

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> WidgetExt for Rectangle<S, F> {}
