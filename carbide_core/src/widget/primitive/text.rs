use instant::Instant;
use rusttype::Scale;

use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;
use crate::render::text::Text as RenderText;
use crate::render::util::new_primitive;
use crate::text::FontId;
use crate::text::Text as InternalText;
use crate::text_old::PositionedGlyph;
use crate::utils;
use crate::widget::types::justify;
use crate::widget::types::justify::Justify;
use crate::widget::types::text_wrap::Wrap;

/// Displays some given text centered within a rectangular area.
///
/// By default, the rectangular dimensions are fit to the area occupied by the text.
///
/// If some horizontal dimension is given, the text will automatically wrap to the width and align
/// in accordance with the produced **Alignment**.
#[derive(Debug, Clone, Widget)]
pub struct Text<GS> where GS: GlobalState {
    id: Uuid,
    position: Point,
    dimension: Dimensions,
    wrap_mode: Wrap,
    #[state] pub text: StringState<GS>,
    #[state] font_size: U32State<GS>,
    #[state] color: ColorState<GS>,
    internal_text: InternalText,
}

impl<GS: GlobalState> Layout<GS> for Text<GS> {
    fn flexibility(&self) -> u32 {
        2
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        let now = Instant::now();
        let text = self.text.get_latest_value();
        let font_size = self.font_size.get_latest_value();
        self.internal_text.update(text.as_str(), *font_size, env);

        let pref_width = requested_size[0].min(self.internal_text.max_width());
        self.dimension = [pref_width, self.dimension[1]];

        let pref_height = requested_size[1].min(self.internal_text.height(self.dimension[0], *font_size));
        self.dimension = [self.dimension[0], pref_height];

        println!("Time for calculate size: {}us", now.elapsed().as_micros());

        self.dimension
    }

    fn position_children(&mut self) {}
}

impl<GS: GlobalState> Render<GS> for Text<GS> {
    fn get_primitives(&mut self, env: &Environment<GS>, _: &GS) -> Vec<Primitive> {
        let (text, font_id) = self.get_render_text(env);

        let kind = PrimitiveKind::Text {
            color: self.color.get_latest_value().clone(),
            text,
            font_id,
        };

        let mut prims: Vec<Primitive> = vec![new_primitive(kind, Rect::new(self.position, self.dimension))];
        prims.extend(Rectangle::<GS>::debug_outline(Rect::new(self.position, self.dimension), 1.0));

        return prims;
    }
}

impl<S: GlobalState> CommonWidget<S> for Text<S> {
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

impl<GS: GlobalState> Text<GS> {
    pub fn new<K: Into<StringState<GS>>>(text: K) -> Box<Self> {
        Box::new(Text {
            id: Uuid::new_v4(),
            text: text.into(),
            font_size: EnvironmentFontSize::Body.into(),
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
            wrap_mode: Wrap::Whitespace,
            color: EnvironmentColor::Label.into(),
            internal_text: InternalText::new(0, Wrap::Whitespace),
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

    /// Calculate the max width of the text, bounded by the proposed max width
    fn max_width(&self, proposed_max_width: f64, env: &mut Environment<GS>) -> Scalar {
        let font = env.get_font_mut(0); // Fixme: add multiple fonts
        let font_size = *self.font_size.get_latest_value();
        let mut max_width = 0.0;

        for line in self.text.get_latest_value().lines() {
            let width = font.calculate_width(line, font_size);

            // We can end early, because we are bounded by the proposed max
            if width >= proposed_max_width {
                return proposed_max_width;
            }
            max_width = utils::partial_max(max_width, width);
        }
        max_width
    }

    /// Calculate the max height for the text, including newlines and wrapping
    fn max_height(&self, env: &Environment<GS>) -> Scalar {
        let font = env.get_font(0); // Fixme: add multiple fonts

        let text = &self.text;
        let font_size = *self.font_size.get_latest_value();
        let wrap = self.wrap_mode;
        let num_lines = match wrap {
            Wrap::Character =>
                text_old::line::infos(text.get_latest_value(), font, font_size)
                    .wrap_by_character(self.dimension[0])
                    .count(),
            Wrap::Whitespace =>
                text_old::line::infos(text.get_latest_value(), font, font_size)
                    .wrap_by_whitespace(self.dimension[0])
                    .count(),
            _ => {
                text.get_latest_value().lines().count()
            }
        };
        let line_spacing = 1.0;
        let height = text_old::height(num_lines.max(1), font_size, line_spacing);
        height
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


    pub fn get_positioned_glyphs(&self, env: &Environment<GS>, scale_factor: f32) -> Vec<PositionedGlyph> {
        let (render_text, _) = self.get_render_text(env);
        render_text.positioned_glyphs(env, scale_factor)
    }

    pub fn get_render_text(&self, env: &Environment<GS>) -> (RenderText, FontId) {
        let font_id = 0 as FontId;
        let font = env.get_font(font_id);

        let rect = Rect::new(self.position, self.dimension);

        let new_line_infos = match self.wrap_mode {
            Wrap::None =>
                text_old::line::infos(&self.text.get_latest_value(), font, *self.font_size.get_latest_value()),
            Wrap::Character =>
                text_old::line::infos(&self.text.get_latest_value(), font, *self.font_size.get_latest_value())
                    .wrap_by_character(rect.w()),
            Wrap::Whitespace =>
                text_old::line::infos(&self.text.get_latest_value(), font, *self.font_size.get_latest_value())
                    .wrap_by_whitespace(rect.w()),
        };

        let base_line_offset = font.get_inner().v_metrics(Scale::uniform(*self.font_size.get_latest_value() as f32)).descent;

        let t = RenderText {
            positioned_glyphs: Vec::new(),
            text: self.text.get_latest_value().clone(),
            line_infos: new_line_infos.collect(),
            font_id,
            font_size: *self.font_size.get_latest_value(),
            rect,
            justify: Justify::Left,
            line_spacing: 1.0,
            base_line_offset,
        };

        (t, font_id)
    }
}

impl<GS: GlobalState> WidgetExt<GS> for Text<GS> {}