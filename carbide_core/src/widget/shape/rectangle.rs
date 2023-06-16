use lyon::algorithms::path::builder::PathBuilder;
use lyon::algorithms::path::Winding;
use lyon::geom::euclid::rect;
use carbide_core::render::{RenderContext, Style};
use carbide_core::state::StateSync;


use carbide_macro::{carbide_default_builder2};

use crate::{CommonWidgetImpl};

use crate::draw::{Dimension, Position, Rect, Color, Scalar};
use crate::environment::{Environment};
use crate::environment::EnvironmentColor;
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::state::{ReadState, IntoReadState};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId};
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
    pub fn new() -> Box<Rectangle<impl ReadState<T=Style>, impl ReadState<T=Style>>> {
        Box::new(Rectangle {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            stroke_color: EnvironmentColor::Blue.style(),
            fill_color: EnvironmentColor::Blue.style(),
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: PrimitiveStore::new(),
        })
    }
}

impl<S2: ReadState<T=Style> + Clone, F2: ReadState<T=Style> + Clone> Rectangle<S2, F2> {
    pub fn fill<F: IntoReadState<Style>>(self, color: F) -> Box<Rectangle<S2, F::Output>> {
        Box::new(Rectangle {
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

    pub fn stroke<S: IntoReadState<Style>>(self, color: S) -> Box<Rectangle<S::Output, F2>> {
        Box::new(Rectangle {
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

        ZStack::new(vec![Blur::gaussian(10.0), Box::new(self)])
    }*/

    pub fn position(mut self, position: Position) -> Box<Self> {
        self.position = position;
        Box::new(self)
    }

    //#[cfg(not(feature = "debug-outline"))]
    pub fn debug_outline(_rect: Rect, _width: Scalar) -> Vec<Primitive> {
        vec![]
    }

    pub fn debug_outline_special(rect: Rect, border_width: Scalar) -> Vec<Primitive> {
        let (l, r, b, t) = rect.l_r_b_t();

        let left_border = Rect::new(
            Position::new(l, b),
            Dimension::new(border_width, rect.height()),
        );
        let right_border = Rect::new(
            Position::new(r - border_width, b),
            Dimension::new(border_width, rect.height()),
        );

        let top_border = Rect::new(
            Position::new(l + border_width, b),
            Dimension::new(rect.width() - border_width * 2.0, border_width),
        );
        let bottom_border = Rect::new(
            Position::new(l + border_width, t - border_width),
            Dimension::new(rect.width() - border_width * 2.0, border_width),
        );

        let border_color = Color::Rgba(0.0 / 255.0, 255.0 / 255.0, 251.0 / 255.0, 1.0); //Color::random();
        vec![
            Primitive {
                kind: PrimitiveKind::RectanglePrim {
                    color: border_color.clone(),
                },
                bounding_box: left_border,
            },
            Primitive {
                kind: PrimitiveKind::RectanglePrim {
                    color: border_color.clone(),
                },
                bounding_box: right_border,
            },
            Primitive {
                kind: PrimitiveKind::RectanglePrim {
                    color: border_color.clone(),
                },
                bounding_box: top_border,
            },
            Primitive {
                kind: PrimitiveKind::RectanglePrim {
                    color: border_color.clone(),
                },
                bounding_box: bottom_border,
            },
        ]
    }

    //#[cfg(feature = "debug-outline")]
    /*pub fn debug_outline(rect: Rect, width: Scalar) -> Vec<Primitive> {
        let (l, r, b, t) = rect.l_r_b_t();

        let left_border = Rect::new([l,b], [width, rect.h()]);
        let right_border = Rect::new([r-width,b], [width, rect.h()]);
        let top_border = Rect::new([l+width,b], [rect.w()-width*2.0, width]);
        let bottom_border = Rect::new([l+width,t-width], [rect.w()-width*2.0, width]);

        let border_color = Color::Rgba(0.0 / 255.0, 255.0 / 255.0, 251.0 / 255.0, 1.0);//Color::random();
        vec![
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: left_border
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: right_border
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: top_border
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: bottom_border
            },
        ]
    }*/
}

impl<S: ReadState<T=Style> + Clone, F: ReadState<T=Style> + Clone> CommonWidget for Rectangle<S, F> {
    CommonWidgetImpl!(self, id: self.id, position: self.position, dimension: self.dimension);
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

    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, _env: &mut Environment) {
        let rect = rect(
            self.x() as f32,
            self.y() as f32,
            self.width() as f32,
            self.height() as f32,
        );
        tessellate(self, &rect.to_box2d(), &|builder, rectangle| {
            builder.add_rectangle(rectangle, Winding::Positive)
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
