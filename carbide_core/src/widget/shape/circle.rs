use lyon::algorithms::path::Winding;
use lyon::geom::euclid::rect;
use lyon::math::point;

use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Color, Dimension, Position};
use crate::environment::EnvironmentColor;
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext, Style};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Blur, CommonWidget, Widget, WidgetExt, WidgetId, WidgetSync, ZStack};
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::PrimitiveStore;
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;

/// A simple, non-interactive widget for drawing a single **Circle**.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Circle<S, F> where S: ReadState<T=Style>, F: ReadState<T=Style> {
    pub id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] stroke_color: S,
    #[state] fill_color: F,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    triangle_store: PrimitiveStore,
}

impl Circle<Style, Style> {

    #[carbide_default_builder2]
    pub fn new() -> Circle<impl ReadState<T=Style>, impl ReadState<T=Style>> {
        Circle {
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

impl<S2: ReadState<T=Style> + Clone, F2: ReadState<T=Style> + Clone> Circle<S2, F2> {
    pub fn fill<F: IntoReadState<Style>>(self, color: F) -> Circle<S2, F::Output> {
        Circle {
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

    pub fn stroke<S: IntoReadState<Style>>(self, color: S) -> Circle<S::Output, F2> {
        Circle {
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

    pub fn material<M: IntoReadState<Color>>(self, material: M) -> ZStack<(Blur, Circle<S2, impl ReadState<T=Style> + Clone>)> {
        let comp = self.fill(material.into_read_state());
        ZStack::new((Blur::gaussian(10.0), comp))
    }
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> CommonWidget for Circle<S, F> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension);
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Layout for Circle<S, F> {
    fn calculate_size(&mut self, requested_size: Dimension, _ctx: &mut LayoutContext) -> Dimension {
        let min_dimension = requested_size.width.min(requested_size.height);
        self.dimension = Dimension::new(min_dimension, min_dimension);

        requested_size
    }
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Render for Circle<S, F> {
    fn render(&mut self, context: &mut RenderContext) {
        self.sync(context.env);

        let radius = self.width() as f32 / 2.0;
        let center = point(self.x() as f32 + radius, self.y() as f32 + radius);
        let rectangle = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );

        tessellate(self, &rectangle.to_box2d(), &|builder, _| {
            builder.add_circle(center, radius, Winding::Positive);
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

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Shape for Circle<S, F> {
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