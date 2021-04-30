use rusttype::Scale;

use crate::position::Align;
use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;
use crate::render::text::Text as RenderText;
use crate::render::util::new_primitive;
use crate::state::environment_color::EnvironmentColor;
use crate::state::environment_font_size::EnvironmentFontSize;
use crate::text::{font, Justify, PositionedGlyph};
use crate::utils;
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
    #[state] pub text: Box<dyn State<String, GS>>,
    #[state] font_size: U32State<GS>,
    #[state] color: ColorState<GS>,
}

impl<GS: GlobalState> WidgetExt<GS> for Text<GS> {}

impl<S: GlobalState> Layout<S> for Text<S> {
    fn flexibility(&self) -> u32 {
        2
    }

    fn calculate_size(&mut self, proposed_size: Dimensions, env: &Environment<S>) -> Dimensions {
        let pref_width = self.default_x(env.get_fonts_map());

        if pref_width > proposed_size[0] {
            self.dimension = [proposed_size[0], self.dimension[1]];
        } else {
            self.dimension = [pref_width, self.dimension[1]];
        }

        let pref_height = self.default_y(env.get_fonts_map());

        if pref_height > proposed_size[1] {
            self.dimension = [self.dimension[0], proposed_size[1]];
        } else {
            self.dimension = [self.dimension[0], pref_height];
        }

        self.dimension

    }

    fn position_children(&mut self) {

    }
}

impl<GS: GlobalState> Render<GS> for Text<GS> {
    fn get_primitives(&mut self, env: &Environment<GS>, _: &GS) -> Vec<Primitive> {

        let (text, font_id) = self.get_render_text(env.get_fonts_map());

        let kind = PrimitiveKind::Text {
            color: self.color.get_latest_value().clone(),
            text,
            font_id,
        };

        let mut prims: Vec<Primitive> = vec![new_primitive(kind, Rect::new(self.position, self.dimension))];
        prims.extend(Rectangle::<GS>::debug_outline_special(Rect::new(self.position, self.dimension), 1.0));

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



// /// Line styling for the **Text**.
// pub enum Line {
//     /// Underline the text.
//     Under,
//     /// Overline the text.
//     Over,
//     /// Strikethrough the text.
//     Through,
// }



impl<GS: GlobalState> Text<GS> {
    pub fn new<K: Into<Box<dyn State<String, GS>>>>(text: K) -> Box<Self> {
        Box::new(Text {
            id: Uuid::new_v4(),
            text: text.into(),
            font_size: EnvironmentFontSize::Body.into(),
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
            wrap_mode: Wrap::Whitespace,
            color: EnvironmentColor::Label.into()
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

    /// If no specific width was given, we'll use the width of the widest line as a default.
    ///
    /// The `Font` used by the `Text` is retrieved in order to determine the width of each line. If
    /// the font used by the `Text` cannot be found, a dimension of `Absolute(0.0)` is returned.
    fn default_x(&self, fonts: &text::font::Map) -> Scalar {
        let font = fonts.ids().next()
            .and_then(|id| fonts.get(id));
        let font = match font {
            Some(font) => font,
            None => return 0.0,
        };

        let font_size = *self.font_size.get_latest_value();
        let mut max_width = 0.0;
        for line in self.text.get_latest_value().lines() {
            let width = text::line::width(line, font, font_size);
            max_width = utils::partial_max(max_width, width);
        }
        max_width
    }

    /// If no specific height was given, we'll use the total height of the text as a default.
    ///
    /// The `Font` used by the `Text` is retrieved in order to determine the width of each line. If
    /// the font used by the `Text` cannot be found, a dimension of `Absolute(0.0)` is returned.
    fn default_y(&self, fonts: &text::font::Map) -> Scalar {
        let font = fonts.ids().next()
            .and_then(|id| fonts.get(id));

        let font = match font {
            Some(font) => font,
            None => return 0.0,
        };

        let text = &self.text;
        let font_size = *self.font_size.get_latest_value();
        let wrap = self.wrap_mode;
        let num_lines = match wrap {
            Wrap::Character =>
                text::line::infos(text.get_latest_value(), font, font_size)
                    .wrap_by_character(self.dimension[0])
                    .count(),
            Wrap::Whitespace =>
                text::line::infos(text.get_latest_value(), font, font_size)
                    .wrap_by_whitespace(self.dimension[0])
                    .count(),
            _ => {
                text.get_latest_value().lines().count()
            }
        };
        let line_spacing =  1.0;
        let height = text::height(std::cmp::max(num_lines, 1), font_size, line_spacing);
        height
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        self.justify(text::Justify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.justify(text::Justify::Center)
    }

    pub fn justify(self, _j: text::Justify) -> Self {
        self
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.justify(text::Justify::Right)
    }


    pub fn get_positioned_glyphs(&self, fonts: &text::font::Map, dpi: f32) -> Vec<PositionedGlyph> {
        let (render_text, _) = self.get_render_text(fonts);
        render_text.positioned_glyphs(dpi)
    }

    pub fn get_render_text(&self, fonts: &text::font::Map) -> (RenderText, font::Id) {
        let font_id: font::Id = match fonts.ids().next() {
            Some(id) => id,
            None => panic!("No font ids available"),
        };
        let font = match fonts.get(font_id) {
            Some(font) => font,
            None => panic!("Not able to retrieve font"),
        };

        let rect = Rect::new(self.position, self.dimension);

        let new_line_infos = match self.wrap_mode {
            Wrap::None =>
                text::line::infos(&self.text.get_latest_value(), font, *self.font_size.get_latest_value()),
            Wrap::Character =>
                text::line::infos(&self.text.get_latest_value(), font, *self.font_size.get_latest_value())
                    .wrap_by_character(rect.w()),
            Wrap::Whitespace =>
                text::line::infos(&self.text.get_latest_value(), font, *self.font_size.get_latest_value())
                    .wrap_by_whitespace(rect.w()),
        };

        let base_line_offset = font.v_metrics(Scale::uniform(*self.font_size.get_latest_value() as f32)).descent;

        let t = RenderText {
            positioned_glyphs: Vec::new(),
            window_dim: self.dimension,
            text: self.text.get_latest_value().clone(),
            line_infos: new_line_infos.collect(),
            font: font.clone(),
            font_size: *self.font_size.get_latest_value(),
            rect,
            justify: Justify::Left,
            y_align: Align::End,
            line_spacing: 1.0,
            base_line_offset
        };

        (t, font_id)
    }

}
