use lyon::geom::euclid::rect;
use lyon::tessellation::path::builder::BorderRadii;
use lyon::tessellation::path::Winding;
use carbide::draw::shape::{DrawShape, StrokeAlignment};
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Color, Dimension, Position};
use crate::environment::EnvironmentColor;
use crate::render::{Render, RenderContext, Style};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Blur, CommonWidget, Widget, WidgetId, WidgetSync, ZStack};
use crate::widget::shape::{AnyShape};
use crate::widget::types::ShapeStyle;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Capsule<S, F> where S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] stroke_color: S,
    #[state] fill_color: F,
    style: ShapeStyle,
}

impl Capsule<Style, Style> {
    #[carbide_default_builder2]
    pub fn new() -> Capsule<impl ReadState<T=Style>, impl ReadState<T=Style>> {
        Capsule {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            stroke_color: EnvironmentColor::Accent.style(),
            fill_color: EnvironmentColor::Accent.style(),
            style: ShapeStyle::Default,
        }
    }
}

impl<S2: ReadState<T=Style> + Clone, F2: ReadState<T=Style> + Clone> Capsule<S2, F2> {
    pub fn fill<F: IntoReadState<Style>>(self, color: F) -> Capsule<S2, F::Output> {
        Capsule {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            stroke_color: self.stroke_color,
            fill_color: color.into_read_state(),
            style: self.style + ShapeStyle::Fill,
        }
    }

    pub fn stroke<S: IntoReadState<Style>>(self, color: S) -> Capsule<S::Output, F2> {
        Capsule {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            stroke_color: color.into_read_state(),
            fill_color: self.fill_color,
            style: self.style + ShapeStyle::Stroke { line_width: 2.0 },
        }
    }

    pub fn stroke_style(mut self, line_width: f64) -> Self {
        self.style += ShapeStyle::Stroke { line_width };
        self
    }

    pub fn material<M: IntoReadState<Color>>(self, material: M) -> ZStack<(Blur, Capsule<S2, impl ReadState<T=Style> + Clone>)> {
        let comp = self.fill(material.into_read_state());
        ZStack::new((Blur::gaussian(10.0), comp))
    }
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> CommonWidget for Capsule<S, F> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> AnyShape for Capsule<S, F> {
    fn cache_key(&self) -> Option<WidgetId> {
        todo!()
    }

    fn description(&self) -> DrawShape {
        DrawShape::Capsule(self.bounding_box())
    }
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> Render for Capsule<S, F> {
    fn render(&mut self, context: &mut RenderContext) {
        self.sync(context.env);

        match self.style {
            ShapeStyle::Default | ShapeStyle::Fill => {
                context.style(self.fill_color.value().convert(self.position, self.dimension), |this| {
                    this.fill_shape(self)
                })
            }
            ShapeStyle::Stroke { line_width } => {
                context.style(self.stroke_color.value().convert(self.position, self.dimension), |this| {
                    this.stroke_shape(self, line_width, StrokeAlignment::Positive)
                })
            }
            ShapeStyle::FillAndStroke { line_width } => {
                context.style(self.fill_color.value().convert(self.position, self.dimension), |this| {
                    this.fill_shape(self)
                });
                context.style(self.stroke_color.value().convert(self.position, self.dimension), |this| {
                    this.stroke_shape(self, line_width, StrokeAlignment::Positive)
                });
            }
        }
    }
}