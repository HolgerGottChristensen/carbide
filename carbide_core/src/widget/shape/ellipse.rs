use lyon::algorithms::math::Angle;
use lyon::algorithms::path::geom::euclid::vec2;
use lyon::algorithms::path::Winding;
use lyon::geom::euclid::rect;
use lyon::math::point;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Color, Dimension, Position};
use crate::environment::EnvironmentColor;
use crate::render::{Render, RenderContext, Style};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Blur, CommonWidget, Widget, WidgetExt, WidgetId, WidgetSync, ZStack};
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::TriangleStore;
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;

/// A simple, non-interactive widget for drawing a single **Ellipse**.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Ellipse<S, F> where S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state]
    stroke_color: S,
    #[state]
    fill_color: F,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    triangle_store: TriangleStore,
}

impl Ellipse<Style, Style> {
    #[carbide_default_builder2]
    pub fn new() -> Ellipse<impl ReadState<T=Style>, impl ReadState<T=Style>> {
        Ellipse {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            stroke_color: EnvironmentColor::Accent.style(),
            fill_color: EnvironmentColor::Accent.style(),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: TriangleStore::new(),
        }
    }
}

impl<S2: ReadState<T=Style> + Clone, F2: ReadState<T=Style> + Clone> Ellipse<S2, F2> {
    pub fn fill<F: IntoReadState<Style>>(self, color: F) -> Ellipse<S2, F::Output> {
        Ellipse {
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

    pub fn stroke<S: IntoReadState<Style>>(self, color: S) -> Ellipse<S::Output, F2> {
        Ellipse {
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

    pub fn material<M: IntoReadState<Color>>(self, material: M) -> ZStack<(Blur, Ellipse<S2, impl ReadState<T=Style> + Clone>)> {
        let comp = self.fill(material.clone().into_read_state());
        ZStack::new((Blur::gaussian(10.0), comp))
    }
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> CommonWidget for Ellipse<S, F> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Render for Ellipse<S, F> {
    fn render(&mut self, context: &mut RenderContext) {
        self.sync(context.env);

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
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Shape for Ellipse<S, F> {
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