use std::borrow::Borrow;
use std::fmt::Debug;

use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
//use crate::render::text::Text as RenderText;
use crate::render::new_primitive;
use crate::render::PrimitiveKind;
use crate::text::Text as InternalText;
use crate::text::{
    FontStyle, FontWeight, Glyph, NoStyleTextSpanGenerator, TextDecoration, TextSpanGenerator,
    TextStyle,
};
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
pub struct Text {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    wrap_mode: Wrap,
    #[state]
    pub text: StringState,
    #[state]
    font_size: U32State,
    #[state]
    color: TState<Color>,
    font_family: String,
    font_style: FontStyle,
    font_weight: FontWeight,
    text_decoration: TextDecoration,
    internal_text: Option<InternalText>,
    text_span_generator: Box<dyn TextSpanGenerator>,
}

impl Text {
    pub fn new<K: Into<StringState>>(text: K) -> Box<Self> {
        let text = text.into();

        Box::new(Text {
            id: WidgetId::new(),
            text,
            font_size: EnvironmentFontSize::Body.into(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            wrap_mode: Wrap::Whitespace,
            color: EnvironmentColor::Label.into(),
            font_family: "system-font".to_string(),
            font_style: FontStyle::Normal,
            font_weight: FontWeight::Normal,
            text_decoration: TextDecoration::None,
            internal_text: None,
            text_span_generator: Box::new(NoStyleTextSpanGenerator {}),
        })
    }

    pub fn new_with_generator<K: Into<StringState>, G: Into<Box<dyn TextSpanGenerator>>>(
        text: K,
        generator: G,
    ) -> Box<Self> {
        let text = text.into();

        Box::new(Text {
            id: WidgetId::new(),
            text,
            font_size: EnvironmentFontSize::Body.into(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            wrap_mode: Wrap::Whitespace,
            color: EnvironmentColor::Label.into(),
            font_family: "system-font".to_string(),
            font_style: FontStyle::Normal,
            font_weight: FontWeight::Normal,
            text_decoration: TextDecoration::None,
            internal_text: None,
            text_span_generator: generator.into(),
        })
    }

    pub fn color<C: Into<TState<Color>>>(mut self, color: C) -> Box<Self> {
        self.color = color.into();
        Box::new(self)
    }

    pub fn font_size<K: Into<U32State>>(mut self, size: K) -> Box<Self> {
        self.font_size = size.into();
        Box::new(self)
    }

    pub fn font_weight<K: Into<FontWeight>>(mut self, weight: K) -> Box<Self> {
        self.font_weight = weight.into();
        Box::new(self)
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
}

impl Layout for Text {
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

    fn position_children(&mut self) {
        let position = Position::new(self.x(), self.y());
        if let Some(internal) = &mut self.internal_text {
            internal.position(position)
        }
    }
}

impl Render for Text {
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        let default_color = *self.color.value();

        if let Some(internal) = &mut self.internal_text {
            internal.ensure_glyphs_added_to_atlas(env);

            for (glyphs, color, additional_rects) in internal.span_glyphs() {
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

impl CommonWidget for Text {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = Position::new(position.x.round(), position.y.round());
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

impl WidgetExt for Text {}
