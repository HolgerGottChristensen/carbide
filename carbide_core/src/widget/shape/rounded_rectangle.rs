use lyon::geom::euclid::rect;
use lyon::tessellation::path::builder::BorderRadii;
use lyon::tessellation::path::Winding;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Color, Dimension, Position};
use crate::environment::EnvironmentColor;
use crate::render::{Render, RenderContext, Style};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Blur, CommonWidget, CornerRadii, Widget, WidgetExt, WidgetId, WidgetSync, ZStack};
use crate::widget::shape::{Shape, tessellate};
use crate::widget::types::PrimitiveStore;
use crate::widget::types::ShapeStyle;
use crate::widget::types::StrokeStyle;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct RoundedRectangle<S, F> where S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone {
    #[id] id: WidgetId,
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
    pub fn new(corner_radii: impl Into<CornerRadii>) -> RoundedRectangle<impl ReadState<T=Style>, impl ReadState<T=Style>> {
        RoundedRectangle {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            corner_radii: corner_radii.into(),
            stroke_color: EnvironmentColor::Accent.style(),
            fill_color: EnvironmentColor::Accent.style(),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: PrimitiveStore::new(),
        }
    }
}

impl<S2: ReadState<T=Style> + Clone, F2: ReadState<T=Style> + Clone> RoundedRectangle<S2, F2> {
    pub fn fill<F: IntoReadState<Style>>(self, color: F) -> RoundedRectangle<S2, F::Output> {
        RoundedRectangle {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            corner_radii: self.corner_radii,
            stroke_color: self.stroke_color,
            fill_color: color.into_read_state(),
            style: self.style + ShapeStyle::Fill,
            stroke_style: self.stroke_style,
            triangle_store: self.triangle_store,
        }
    }

    pub fn stroke<S: IntoReadState<Style>>(self, color: S) -> RoundedRectangle<S::Output, F2> {
        RoundedRectangle {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            corner_radii: self.corner_radii,
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

    pub fn material<M: IntoReadState<Color>>(self, material: M) -> ZStack<(Blur, RoundedRectangle<S2, impl ReadState<T=Style> + Clone>)> {
        let comp = self.fill(material.clone().into_read_state());
        ZStack::new((Blur::gaussian(10.0), comp))
    }
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> CommonWidget for RoundedRectangle<S, F> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Render for RoundedRectangle<S, F> {
    fn render(&mut self, context: &mut RenderContext) {

        self.sync(context.env);

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