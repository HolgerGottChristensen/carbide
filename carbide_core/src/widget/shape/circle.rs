use crate::draw::DrawShape;
use crate::draw::{Color, Dimension, CompositeDrawShape, Position};
use crate::environment::EnvironmentColor;
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext, Style};
use crate::state::{IntoReadState, ReadState};
use crate::widget::shape::AnyShape;
use crate::widget::types::ShapeStyle;
use crate::widget::{Blur, CommonWidget, Widget, WidgetId, WidgetSync, ZStack};
use crate::CommonWidgetImpl;

/// A simple, non-interactive widget for drawing a single **Circle**.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Circle<S, F> where S: ReadState<T=Style>, F: ReadState<T=Style> {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] stroke_color: S,
    #[state] fill_color: F,
    style: ShapeStyle,
}

impl Circle<Style, Style> {
    pub fn new() -> Circle<impl ReadState<T=Style>, impl ReadState<T=Style>> {
        Circle {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            stroke_color: EnvironmentColor::Accent.style(),
            fill_color: EnvironmentColor::Accent.style(),
            style: ShapeStyle::Default,
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
        }
    }

    pub fn stroke<S: IntoReadState<Style>>(self, color: S) -> Circle<S::Output, F2> {
        Circle {
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

    pub fn material<M: IntoReadState<Color>>(self, material: M) -> ZStack<(Blur, Circle<S2, impl ReadState<T=Style> + Clone>)> {
        let comp = self.fill(material.into_read_state());
        ZStack::new((Blur::gaussian(10.0), comp))
    }
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> CommonWidget for Circle<S, F> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
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

        let primitive = DrawShape::Circle(self.center_point(), self.dimension.width / 2.0);

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

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> AnyShape for Circle<S, F> {
    fn cache_key(&self) -> Option<WidgetId> {
        todo!()
    }

    fn description(&self) -> CompositeDrawShape {
        todo!()
    }
}