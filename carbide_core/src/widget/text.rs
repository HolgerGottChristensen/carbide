use std::fmt::Debug;

use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position};
use crate::environment::{EnvironmentColor, EnvironmentFontSize};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext, Style};
use crate::state::{IntoReadState, ReadState};
use crate::text::{FontStyle, FontWeight, TextDecoration, TextId, TextStyle};
use crate::widget::{AnyWidget, CommonWidget, Justify, Widget, WidgetExt, WidgetId, WidgetSync};
use crate::widget::types::Wrap;

/// Displays some given text centered within a rectangular area.
///
/// By default, the rectangular dimensions are fit to the area occupied by the text.
///
/// If some horizontal dimension is given, the text will automatically wrap to the width and align
/// in accordance with the produced **Alignment**.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Text<T, S, C, FS, FW> where T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight> {
    id: WidgetId,
    text_id: TextId,
    position: Position,
    dimension: Dimension,
    wrap_mode: Wrap,
    #[state] pub text: T,
    #[state] font_size: S,
    #[state] color: C,
    family: String,
    font_style: FS,
    font_weight: FW,
    text_decoration: TextDecoration,
    //internal_text: Option<InternalText>,
    //text_span_generator: Box<dyn TextSpanGenerator>,
}

impl Text<String, u32, Style, FontStyle, FontWeight> {
    #[carbide_default_builder2]
    pub fn new<T: IntoReadState<String>>(text: T) -> Text<T::Output, impl ReadState<T=u32>, impl ReadState<T=Style>, FontStyle, FontWeight> {
        let text = text.into_read_state();

        Text {
            id: WidgetId::new(),
            text_id: TextId::new(),
            text,
            font_size: EnvironmentFontSize::Body.u32(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            wrap_mode: Wrap::Whitespace,
            color: EnvironmentColor::Label.style(),
            family: "Noto Sans".to_string(),
            font_style: FontStyle::Normal,
            font_weight: FontWeight::Normal,
            text_decoration: TextDecoration::None,
            //internal_text: None,
            //text_span_generator: Box::new(NoStyleTextSpanGenerator {}),
        }
    }

    // pub fn new_with_generator<T: IntoReadState<String>>(
    //     text: T,
    //     generator: impl Into<Box<dyn TextSpanGenerator>>,
    // ) -> Text<T::Output, impl ReadState<T=u32>, impl ReadState<T=Color>> {
    //     let text = text.into_read_state();
    //
    //     Text {
    //         id: WidgetId::new(),
    //         text,
    //         font_size: EnvironmentFontSize::Body.u32(),
    //         position: Position::new(0.0, 0.0),
    //         dimension: Dimension::new(100.0, 100.0),
    //         wrap_mode: Wrap::Whitespace,
    //         color: EnvironmentColor::Label.color(),
    //         font_family: "system-font".to_string(),
    //         font_style: FontStyle::Normal,
    //         font_weight: FontWeight::Normal,
    //         text_decoration: TextDecoration::None,
    //         internal_text: None,
    //         text_span_generator: generator.into(),
    //         text_id: TextId::new(),
    //     }
    // }
}

impl<T2: ReadState<T=String>, S2: ReadState<T=u32>, C2: ReadState<T=Style>, FS2: ReadState<T=FontStyle>, FW2: ReadState<T=FontWeight>> Text<T2, S2, C2, FS2, FW2> {
    pub fn color<C: IntoReadState<Style>>(self, color: C) -> Text<T2, S2, C::Output, FS2, FW2> {
        Text {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            wrap_mode: self.wrap_mode,
            text: self.text,
            font_size: self.font_size,
            color: color.into_read_state(),
            family: self.family,
            font_style: self.font_style,
            font_weight: self.font_weight,
            text_decoration: self.text_decoration,
            //internal_text: self.internal_text,
            //text_span_generator: self.text_span_generator,
            text_id: self.text_id
        }
    }

    pub fn font_size<S: IntoReadState<u32>>(self, size: S) -> Text<T2, S::Output, C2, FS2, FW2> {
        Text {
            id: self.id,
            text_id: self.text_id,
            position: self.position,
            dimension: self.dimension,
            wrap_mode: self.wrap_mode,
            text: self.text,
            font_size: size.into_read_state(),
            color: self.color,
            family: self.family,
            font_style: self.font_style,
            font_weight: self.font_weight,
            text_decoration: self.text_decoration,
        }
    }

    pub fn family(mut self, family: String) -> Self {
        self.family = family;
        self
    }

    pub fn font_weight<FW: IntoReadState<FontWeight>>(mut self, weight: FW) -> Text<T2, S2, C2, FS2, FW::Output> {
        Text {
            id: self.id,
            text_id: self.text_id,
            position: self.position,
            dimension: self.dimension,
            wrap_mode: self.wrap_mode,
            text: self.text,
            font_size: self.font_size,
            color: self.color,
            family: self.family,
            font_style: self.font_style,
            font_weight: weight.into_read_state(),
            text_decoration: self.text_decoration,
        }
    }

    pub fn font_style<FS: IntoReadState<FontStyle>>(mut self, style: FS) -> Text<T2, S2, C2, FS::Output, FW2> {
        Text {
            id: self.id,
            text_id: self.text_id,
            position: self.position,
            dimension: self.dimension,
            wrap_mode: self.wrap_mode,
            text: self.text,
            font_size: self.font_size,
            color: self.color,
            family: self.family,
            font_style: style.into_read_state(),
            font_weight: self.font_weight,
            text_decoration: self.text_decoration,
        }
    }

    /// Take a given text element and make it render with the font weight: Bold
    pub fn bold(self) -> Text<T2, S2, C2, FS2, FontWeight> {
        self.font_weight(FontWeight::Bold)
    }

    pub fn italic(mut self) -> Text<T2, S2, C2, FontStyle, FW2> {
        self.font_style(FontStyle::Italic)
    }

    pub fn wrap_mode(mut self, wrap: Wrap) -> Self {
        self.wrap_mode = wrap;
        self
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn justify_left(self) -> Self {
        self.justify(Justify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn justify_center(self) -> Self {
        self.justify(Justify::Center)
    }

    pub fn justify(self, _j: Justify) -> Self {
        self
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn justify_right(self) -> Self {
        self.justify(Justify::Right)
    }

    pub fn get_style(&self) -> TextStyle {
        TextStyle {
            family: self.family.clone(),
            font_size: *self.font_size.value(),
            line_height: 1.2,
            font_style: *self.font_style.value(),
            font_weight: *self.font_weight.value(),
            text_decoration: self.text_decoration.clone(),
            color: None,
            wrap: self.wrap_mode,
        }
    }

    /// Take a given text element and make it render with an underline
    pub fn underline(mut self) -> Self {
        self.text_decoration = TextDecoration::Underline(vec![]);
        self
    }

    pub fn with_optional_decoration(mut self, decoration: TextDecoration) -> Self {
        self.text_decoration = decoration;
        self
    }

    /*pub fn with_optional_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }*/
}

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>> Layout for Text<T, S, C, FS, FW> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.sync(ctx.env);

        ctx.text.update(self.text_id, &self.text.value(), &self.get_style());
        self.dimension = ctx.text.calculate_size(self.text_id, requested_size, ctx.env);

        // if let None = self.internal_text {
        //     let text = self.text.value().deref().clone();
        //     let style = self.get_style();
        //     //dbg!(&style);
        //     self.internal_text = Some(InternalText::new(
        //         text,
        //         style,
        //         self.wrap_mode,
        //         self.text_span_generator.borrow(),
        //         env,
        //     ))
        // }
        //
        // let style = self.get_style();
        // if let Some(internal) = &mut self.internal_text {
        //     let text = self.text.value().deref().clone();
        //     if internal.string_that_generated_this() != &text
        //         || internal.style_that_generated_this() != &style
        //     {
        //         *internal = InternalText::new(
        //             text,
        //             style,
        //             self.wrap_mode,
        //             self.text_span_generator.borrow(),
        //             env,
        //         );
        //     }
        //     self.dimension = internal.calculate_size(requested_size, env);
        // }

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        ctx.text.calculate_position(self.text_id, self.position.tolerance(1.0/ctx.env.scale_factor()), ctx.env)


        //env.text_context.calculate_position(self.text_id, self.position.tolerance(1.0/env.scale_factor()));

        // if let Some(internal) = &mut self.internal_text {
        //     internal.position(self.position.tolerance(1.0/env.scale_factor()));
        //     //internal.position(self.position);
        //
        //     internal.ensure_glyphs_added_to_atlas(env);
        // }
    }
}

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>> Render for Text<T, S, C, FS, FW> {
    fn render(&mut self, context: &mut RenderContext) {
        self.sync(context.env);

        let default_color = self.color.value();

        context.style(default_color.convert(self.position, self.dimension), |context| {
            context.text(self.text_id);
        });
    }
}

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>> CommonWidget for Text<T, S, C, FS, FW> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, _f: &mut dyn FnMut(&'a dyn AnyWidget)) {}

    fn foreach_child_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}

    fn foreach_child_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}

    fn foreach_child_direct<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}

    fn foreach_child_direct_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}

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

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>> WidgetExt for Text<T, S, C, FS, FW> {}

pub trait TextWidget: AnyWidget {
    fn text_id(&self) -> TextId;
}

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>> TextWidget for Text<T, S, C, FS, FW> {
    fn text_id(&self) -> TextId {
        self.text_id
    }
}

impl AnyWidget for Box<dyn TextWidget> {}

dyn_clone::clone_trait_object!(TextWidget);