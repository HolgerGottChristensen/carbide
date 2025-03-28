use carbide_macro::carbide_default_builder2;

use crate::draw::{Color, Dimension, CompositeDrawShape, Position, DrawShape};
use crate::environment::EnvironmentColor;
use crate::render::{Render, RenderContext, Style};
use crate::state::{IntoReadState, ReadState};
use crate::widget::shape::AnyShape;
use crate::widget::types::ShapeStyle;
use crate::widget::{Blur, CommonWidget, CornerRadii, Widget, WidgetId, WidgetSync, ZStack};
use crate::CommonWidgetImpl;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct RoundedRectangle<S, F> where S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    corner_radii: CornerRadii,
    #[state] stroke_color: S,
    #[state] fill_color: F,
    style: ShapeStyle,
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
            style: self.style + ShapeStyle::Stroke { line_width: 2.0 },
        }
    }

    pub fn stroke_style(mut self, line_width: f64) -> Self {
        self.style += ShapeStyle::Stroke { line_width };
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

        let primitive = DrawShape::RoundedRectangle(self.bounding_box(), self.corner_radii);

        match self.style {
            ShapeStyle::Default | ShapeStyle::Fill => {
                context.style(self.fill_color.value().convert(self.position, self.dimension), |this| {
                    this.shape(primitive, ShapeStyle::Fill)
                })
            }
            ShapeStyle::Stroke { line_width } => {
                context.style(self.stroke_color.value().convert(self.position, self.dimension), |this| {
                    this.shape(primitive, ShapeStyle::Stroke { line_width })
                })
            }
            ShapeStyle::FillAndStroke { line_width } => {
                context.style(self.fill_color.value().convert(self.position, self.dimension), |this| {
                    this.shape(primitive.clone(), ShapeStyle::Fill)
                });
                context.style(self.stroke_color.value().convert(self.position, self.dimension), |this| {
                    this.shape(primitive, ShapeStyle::Stroke { line_width })
                });
            }
        }
    }
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> AnyShape for RoundedRectangle<S, F> {
    fn cache_key(&self) -> Option<WidgetId> {
        todo!()
    }

    fn description(&self) -> CompositeDrawShape {
        todo!()
    }
}