use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;
use carbide_core::render::{RenderContext, Style};
use carbide_core::state::IntoReadState;


use carbide_macro::{carbide_default_builder, carbide_default_builder2};

use crate::Color;
use crate::color::BLACK;
use crate::draw::{Dimension, Position, Rect};
use crate::draw::draw_style::DrawStyle;
use crate::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use crate::layout::Layout;
//use crate::render::text::Text as RenderText;
use crate::render::{new_primitive, Primitive, Render};
use crate::render::PrimitiveKind;
use crate::state::{ReadState, RState, StateSync, TState};
use crate::text::{
    FontStyle, FontWeight, Glyph, NoStyleTextSpanGenerator, TextDecoration, TextSpanGenerator,
    TextStyle,
};
use crate::text::Text as InternalText;
use crate::widget::{CommonWidget, Justify, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};
//use crate::text_old::PositionedGlyph;
use crate::widget::types::Wrap;

/// Displays some given text centered within a rectangular area.
///
/// By default, the rectangular dimensions are fit to the area occupied by the text.
///
/// If some horizontal dimension is given, the text will automatically wrap to the width and align
/// in accordance with the produced **Alignment**.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Text<T, S, C> where T: ReadState<T=String> + Clone, S: ReadState<T=u32> + Clone, C: ReadState<T=Color> + Clone {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    wrap_mode: Wrap,
    #[state]
    pub text: T,
    #[state]
    font_size: S,
    #[state]
    color: C,
    font_family: String,
    font_style: FontStyle,
    font_weight: FontWeight,
    text_decoration: TextDecoration,
    internal_text: Option<InternalText>,
    text_span_generator: Box<dyn TextSpanGenerator>,
}

impl Text<String, u32, Color> {
    #[carbide_default_builder2]
    pub fn new<T: IntoReadState<String>>(text: T) -> Box<Text<T::Output, impl ReadState<T=u32>, impl ReadState<T=Color>>> {
        let text = text.into_read_state();

        Box::new(Text {
            id: WidgetId::new(),
            text,
            font_size: EnvironmentFontSize::Body.u32(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            wrap_mode: Wrap::Whitespace,
            color: EnvironmentColor::Label.color(),
            font_family: "system-font".to_string(),
            font_style: FontStyle::Normal,
            font_weight: FontWeight::Normal,
            text_decoration: TextDecoration::None,
            internal_text: None,
            text_span_generator: Box::new(NoStyleTextSpanGenerator {}),
        })
    }

    /*pub fn new_with_generator<T: IntoReadState<String>>(
        text: T,
        generator: impl Into<Box<dyn TextSpanGenerator>>,
    ) -> Box<Text<T::Output, EnvironmentFontSizeState, <EnvironmentColor as IntoReadState<Color>>::Output>> {
        let text = text.into_read_state();

        Box::new(Text {
            id: WidgetId::new(),
            text,
            font_size: EnvironmentFontSize::Body.into_read_state(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            wrap_mode: Wrap::Whitespace,
            color: <EnvironmentColor as IntoReadState<Color>>::into_read_state(EnvironmentColor::Label),
            font_family: "system-font".to_string(),
            font_style: FontStyle::Normal,
            font_weight: FontWeight::Normal,
            text_decoration: TextDecoration::None,
            internal_text: None,
            text_span_generator: generator.into(),
        })
    }*/
}

impl<T2: ReadState<T=String> + Clone, S2: ReadState<T=u32> + Clone, C2: ReadState<T=Color> + Clone> Text<T2, S2, C2> {
    pub fn color<C: IntoReadState<Color>>(mut self, color: C) -> Box<Text<T2, S2, C::Output>> {
        Box::new(Text {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            wrap_mode: self.wrap_mode,
            text: self.text,
            font_size: self.font_size,
            color: color.into_read_state(),
            font_family: self.font_family,
            font_style: self.font_style,
            font_weight: self.font_weight,
            text_decoration: self.text_decoration,
            internal_text: self.internal_text,
            text_span_generator: self.text_span_generator,
        })
    }

    pub fn font_size<S: IntoReadState<u32>>(mut self, size: S) -> Box<Text<T2, S::Output, C2>> {
        Box::new(Text {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            wrap_mode: self.wrap_mode,
            text: self.text,
            font_size: size.into_read_state(),
            color: self.color,
            font_family: self.font_family,
            font_style: self.font_style,
            font_weight: self.font_weight,
            text_decoration: self.text_decoration,
            internal_text: self.internal_text,
            text_span_generator: self.text_span_generator,
        })
    }

    pub fn font_weight(mut self, weight: impl Into<FontWeight>) -> Box<Self> {
        self.font_weight = weight.into();
        Box::new(self)
    }

    /// Take a given text element and make it render with the font weight: Bold
    pub fn bold(mut self) -> Box<Self> {
        self.font_weight(FontWeight::Bold)
    }

    pub fn wrap_mode(mut self, wrap: Wrap) -> Box<Self> {
        self.wrap_mode = wrap;
        Box::new(self)
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        self.justify(Justify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.justify(Justify::Center)
    }

    pub fn justify(self, _j: Justify) -> Self {
        self
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.justify(Justify::Right)
    }

    pub fn glyphs(&self) -> Vec<Glyph> {
        if let Some(internal) = &self.internal_text {
            internal.first_glyphs()
        } else {
            vec![]
        }
    }

    pub fn get_style(&self) -> TextStyle {
        let color = if self.text_span_generator.store_color() {
            Some(self.color.value().deref().clone())
        } else {
            None
        };
        TextStyle {
            font_family: self.font_family.clone(),
            font_size: *self.font_size.value(),
            font_style: self.font_style,
            font_weight: self.font_weight,
            text_decoration: self.text_decoration.clone(),
            color,
        }
    }

    /// Take a given text element and make it render with an underline
    pub fn underline(mut self) -> Box<Self> {
        self.text_decoration = TextDecoration::Underline(vec![]);
        Box::new(self)
    }

    pub fn with_optional_decoration(mut self, decoration: TextDecoration) -> Box<Self> {
        self.text_decoration = decoration;
        Box::new(self)
    }

    pub fn with_optional_weight(mut self, weight: FontWeight) -> Box<Self> {
        self.font_weight = weight;
        Box::new(self)
    }
}

impl<T: ReadState<T=String> + Clone, S: ReadState<T=u32> + Clone, C: ReadState<T=Color> + Clone> Layout for Text<T, S, C> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.capture_state(env);
        if let None = self.internal_text {
            let text = self.text.value().deref().clone();
            let style = self.get_style();
            //dbg!(&style);
            self.internal_text = Some(InternalText::new(
                text,
                style,
                self.wrap_mode,
                self.text_span_generator.borrow(),
                env,
            ))
        }

        let style = self.get_style();
        if let Some(internal) = &mut self.internal_text {
            let text = self.text.value().deref().clone();
            if internal.string_that_generated_this() != &text
                || internal.style_that_generated_this() != &style
            {
                *internal = InternalText::new(
                    text,
                    style,
                    self.wrap_mode,
                    self.text_span_generator.borrow(),
                    env,
                );
            }
            self.dimension = internal.calculate_size(requested_size, env);
        }

        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
        if let Some(internal) = &mut self.internal_text {
            internal.position(self.position.tolerance(1.0/env.scale_factor()));

            internal.ensure_glyphs_added_to_atlas(env);
        }
    }
}

impl<T: ReadState<T=String> + Clone, S: ReadState<T=u32> + Clone, C: ReadState<T=Color> + Clone> Render for Text<T, S, C> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        let default_color = *self.color.value();

        if let Some(internal) = &mut self.internal_text {
            context.style(DrawStyle::Color(default_color), |context| {
                for (glyphs, color, additional_rects) in &internal.span_glyphs(env.scale_factor()) {

                    context.text(glyphs);


                    /*let color = if let Some(color) = color {
                        color
                    } else {
                        default_color
                    };

                    let kind = PrimitiveKind::Text {
                        color,
                        text: glyphs,
                    };
                    primitives.push(new_primitive(
                        kind,
                        Rect::new(self.position, self.dimension),
                    ));

                    for additional_rect in additional_rects {
                        let position =
                            Position::new(additional_rect.position.x, additional_rect.position.y);
                        let dimension = Dimension::new(
                            additional_rect.dimension.width,
                            additional_rect.dimension.height,
                        );
                        primitives.push(Primitive {
                            kind: PrimitiveKind::RectanglePrim { color },
                            bounding_box: Rect::new(position, dimension),
                        });
                    }*/
                }
            });
        }
    }

    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        let default_color = *self.color.value();

        if let Some(internal) = &mut self.internal_text {
            internal.position(self.position.tolerance(1.0/env.scale_factor()));
        }

        if let Some(internal) = &mut self.internal_text {
            internal.ensure_glyphs_added_to_atlas(env);

            for (glyphs, color, additional_rects) in internal.span_glyphs(env.scale_factor()) {
                let color = if let Some(color) = color {
                    color
                } else {
                    default_color
                };

                let kind = PrimitiveKind::Text {
                    color,
                    text: glyphs,
                };
                primitives.push(new_primitive(
                    kind,
                    Rect::new(self.position, self.dimension),
                ));

                for additional_rect in additional_rects {
                    let position =
                        Position::new(additional_rect.position.x, additional_rect.position.y);
                    let dimension = Dimension::new(
                        additional_rect.dimension.width,
                        additional_rect.dimension.height,
                    );
                    primitives.push(Primitive {
                        kind: PrimitiveKind::RectanglePrim { color },
                        bounding_box: Rect::new(position, dimension),
                    });
                }
            }
        }
    }
}

impl<T: ReadState<T=String> + Clone, S: ReadState<T=u32> + Clone, C: ReadState<T=Color> + Clone> CommonWidget for Text<T, S, C> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget)) {}

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {}

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {}

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {}

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {}


    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn flexibility(&self) -> u32 {
        2
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<T: ReadState<T=String> + Clone, S: ReadState<T=u32> + Clone, C: ReadState<T=Color> + Clone> WidgetExt for Text<T, S, C> {}
