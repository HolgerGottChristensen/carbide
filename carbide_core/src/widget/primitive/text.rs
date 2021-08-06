use std::borrow::Borrow;
use std::fmt::Debug;

use instant::Instant;

use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;
//use crate::render::text::Text as RenderText;
use crate::render::util::new_primitive;
use crate::text::{FontStyle, FontWeight, Glyph, NoStyleTextSpanGenerator, TextDecoration, TextSpanGenerator, TextStyle};
use crate::text::Text as InternalText;
//use crate::text_old::PositionedGlyph;
use crate::widget::types::justify;
use crate::widget::types::text_wrap::Wrap;

/// Displays some given text centered within a rectangular area.
///
/// By default, the rectangular dimensions are fit to the area occupied by the text.
///
/// If some horizontal dimension is given, the text will automatically wrap to the width and align
/// in accordance with the produced **Alignment**.
#[derive(Debug, Clone, Widget)]
pub struct Text<GS> where GS: GlobalStateContract {
    id: Uuid,
    position: Point,
    dimension: Dimensions,
    wrap_mode: Wrap,
    #[state] pub text: StringState<GS>,
    #[state] font_size: U32State<GS>,
    #[state] color: ColorState<GS>,
    font_family: String,
    font_style: FontStyle,
    font_weight: FontWeight,
    text_decoration: TextDecoration,
    internal_text: Option<InternalText<GS>>,
    text_span_generator: Box<dyn TextSpanGenerator<GS>>,
}

impl<GS: GlobalStateContract> Text<GS> {
    pub fn new<K: Into<StringState<GS>>>(text: K) -> Box<Self> {
        let text = text.into();

        Box::new(Text {
            id: Uuid::new_v4(),
            text,
            font_size: EnvironmentFontSize::Body.into(),
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
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

    pub fn new_with_generator<K: Into<StringState<GS>>, G: Into<Box<dyn TextSpanGenerator<GS>>>>(text: K, generator: G) -> Box<Self> {
        let text = text.into();

        Box::new(Text {
            id: Uuid::new_v4(),
            text,
            font_size: EnvironmentFontSize::Body.into(),
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
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

    pub fn color<C: Into<ColorState<GS>>>(mut self, color: C) -> Box<Self> {
        self.color = color.into();
        Box::new(self)
    }

    pub fn font_size<K: Into<U32State<GS>>>(mut self, size: K) -> Box<Self> {
        self.font_size = size.into();
        Box::new(self)
    }

    pub fn wrap_mode(mut self, wrap: Wrap) -> Box<Self> {
        self.wrap_mode = wrap;
        Box::new(self)
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        self.justify(justify::Justify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.justify(justify::Justify::Center)
    }

    pub fn justify(self, _j: justify::Justify) -> Self {
        self
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.justify(justify::Justify::Right)
    }

    pub fn get_positioned_glyphs(&self, env: &Environment<GS>, scale_factor: f32) -> Vec<Glyph> {
        if let Some(internal) = &self.internal_text {
            internal.first_glyphs()
        } else {
            vec![]
        }
    }

    pub fn get_style(&self) -> TextStyle {
        TextStyle {
            font_family: self.font_family.clone(),
            font_size: *self.font_size,
            font_style: self.font_style,
            font_weight: self.font_weight,
            text_decoration: self.text_decoration.clone(),
            color: Some(*self.color.clone()),
        }
    }
}

impl<GS: GlobalStateContract> Layout<GS> for Text<GS> {
    fn flexibility(&self) -> u32 {
        2
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        let now = Instant::now();
        let style = self.get_style();

        if let None = self.internal_text {
            let text = (&*self.text).clone();
            let style = self.get_style();
            self.internal_text = Some(InternalText::new(text, style, self.text_span_generator.borrow(), env))
        }


        if let Some(internal) = &mut self.internal_text {
            let text = (&*self.text).clone();
            if internal.string_that_generated_this() != &text {
                *internal = InternalText::new(text, style, self.text_span_generator.borrow(), env);
            }
            let size = internal.calculate_size(Dimension::new(requested_size[0], requested_size[1]), env);

            self.dimension = [size.width, size.height]
        }

        println!("Time for calculate size: {}us", now.elapsed().as_micros());

        self.dimension
    }

    fn position_children(&mut self) {
        let position = Position::new(self.get_x(), self.get_y());
        if let Some(internal) = &mut self.internal_text {
            internal.position(position)
        }
    }
}

impl<GS: GlobalStateContract> Render<GS> for Text<GS> {
    fn get_primitives(&mut self, env: &mut Environment<GS>) -> Vec<Primitive> {
        let mut prims: Vec<Primitive> = vec![];
        let default_color = *self.color.clone();

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
                prims.push(new_primitive(kind, OldRect::new(self.position, self.dimension)));

                for additional_rect in additional_rects {
                    let position = [additional_rect.position.x, additional_rect.position.y];
                    let dimension = [additional_rect.dimension.width, additional_rect.dimension.height];
                    prims.push(Primitive {
                        kind: PrimitiveKind::Rectangle { color },
                        rect: OldRect::new(position, dimension),
                    });
                }
            }
        }

        prims.extend(Rectangle::<GS>::debug_outline(OldRect::new(self.position, self.dimension), 1.0));

        return prims;
    }
}

impl<S: GlobalStateContract> CommonWidget<S> for Text<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        WidgetIter::Empty
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::Empty
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::Empty
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::Empty
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<GS: GlobalStateContract> WidgetExt<GS> for Text<GS> {}