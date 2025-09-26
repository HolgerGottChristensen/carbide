use crate::accessibility::Accessibility;
use crate::accessibility::AccessibilityContext;
use crate::draw::{Dimension, Position};
use crate::environment::{EnvironmentColor, EnvironmentFontSize};
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext, Style};
use crate::scene::SceneManager;
use crate::state::{IntoReadState, ReadState};
use crate::text::text_justify::TextJustify;
use crate::text::text_wrap::{wrap_state, Wrap, WrapState};
use crate::text::{FontStyle, FontWeight, TextDecoration, TextId, TextStyle};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetId, WidgetSync};
use accesskit::{Node, Point, Rect, Role, Size};
use carbide_macro::carbide_default_builder2;
use std::fmt::Debug;
use crate::identifiable::Identifiable;

/// Displays some given text centered within a rectangular area.
///
/// By default, the rectangular dimensions are fit to the area occupied by the text.
///
/// If some horizontal dimension is given, the text will automatically wrap to the width and align
/// in accordance with the produced **Alignment**.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Layout, Accessibility)]
pub struct Text<T, S, C, FS, FW, W>
where
    T: ReadState<T=String>,
    S: ReadState<T=u32>,
    C: ReadState<T=Style>,
    FS: ReadState<T=FontStyle>,
    FW: ReadState<T=FontWeight>,
    W: ReadState<T=Wrap>
{
    #[id] id: WidgetId,
    text_id: TextId,
    position: Position,
    dimension: Dimension,
    #[state] pub text: T,
    #[state] font_size: S,
    #[state] color: C,
    #[state] wrap_mode: W,
    family: String,
    font_style: FS,
    font_weight: FW,
    text_decoration: TextDecoration,
}

impl Text<String, u32, Style, FontStyle, FontWeight, Wrap> {
    #[carbide_default_builder2]
    pub fn new<T: IntoReadState<String>>(text: T) -> Text<T::Output, impl ReadState<T=u32>, impl ReadState<T=Style>, FontStyle, FontWeight, WrapState> {
        let text = text.into_read_state();

        Text {
            id: WidgetId::new(),
            text_id: TextId::new(),
            text,
            font_size: EnvironmentFontSize::Body.u32(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            wrap_mode: wrap_state(),
            color: EnvironmentColor::Label.style(),
            family: "Noto Sans".to_string(),
            font_style: FontStyle::Normal,
            font_weight: FontWeight::Normal,
            text_decoration: TextDecoration::None,
        }
    }
}

impl<T2: ReadState<T=String>, S2: ReadState<T=u32>, C2: ReadState<T=Style>, FS2: ReadState<T=FontStyle>, FW2: ReadState<T=FontWeight>, W: ReadState<T=Wrap>> Text<T2, S2, C2, FS2, FW2, W> {
    pub fn color<C: IntoReadState<Style>>(self, color: C) -> Text<T2, S2, C::Output, FS2, FW2, W> {
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

    pub fn font_size<S: IntoReadState<u32>>(self, size: S) -> Text<T2, S::Output, C2, FS2, FW2, W> {
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

    pub fn font_weight<FW: IntoReadState<FontWeight>>(self, weight: FW) -> Text<T2, S2, C2, FS2, FW::Output, W> {
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

    pub fn font_style<FS: IntoReadState<FontStyle>>(self, style: FS) -> Text<T2, S2, C2, FS::Output, FW2, W> {
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
    pub fn bold(self) -> Text<T2, S2, C2, FS2, FontWeight, W> {
        self.font_weight(FontWeight::Bold)
    }

    pub fn italic(self) -> Text<T2, S2, C2, FontStyle, FW2, W> {
        self.font_style(FontStyle::Italic)
    }

    pub fn wrap<W2: IntoReadState<Wrap>>(self, wrap: W2) -> Text<T2, S2, C2, FS2, FW2, W2::Output> {
        Text {
            id: self.id,
            text_id: self.text_id,
            position: self.position,
            dimension: self.dimension,
            wrap_mode: wrap.into_read_state(),
            text: self.text,
            font_size: self.font_size,
            color: self.color,
            family: self.family,
            font_style: self.font_style,
            font_weight: self.font_weight,
            text_decoration: self.text_decoration,
        }
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn justify_left(self) -> Self {
        self.justify(TextJustify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn justify_center(self) -> Self {
        self.justify(TextJustify::Center)
    }

    pub fn justify(self, _j: TextJustify) -> Self {
        self
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn justify_right(self) -> Self {
        self.justify(TextJustify::Right)
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
            wrap: *self.wrap_mode.value(),
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
}

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>, W: ReadState<T=Wrap>> Layout for Text<T, S, C, FS, FW, W> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.sync(ctx.env);

        ctx.text.update(self.text_id, &self.text.value(), &self.get_style());
        self.dimension = ctx.text.calculate_size(self.text_id, requested_size, ctx.env);

        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let scale_factor = ctx.env.get_mut::<SceneManager>()
            .map(|a| a.scale_factor())
            .unwrap_or(1.0);

        ctx.text.calculate_position(self.text_id, self.position.tolerance(1.0 / scale_factor), ctx.env)
    }
}

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>, W: ReadState<T=Wrap>> Render for Text<T, S, C, FS, FW, W> {
    fn render(&mut self, context: &mut RenderContext) {
        self.sync(context.env);

        let default_color = self.color.value();

        context.style(default_color.convert(self.position, self.dimension), |context| {
            context.text(self.text_id);
        });
    }
}

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>, W: ReadState<T=Wrap>> Accessibility for Text<T, S, C, FS, FW, W> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env);

        let scale_factor = ctx.env.get_mut::<SceneManager>()
            .map(|a| a.scale_factor())
            .unwrap_or(1.0);

        let mut builder = Node::new(Role::Label);

        builder.set_bounds(Rect::from_origin_size(
            Point::new(self.x() * scale_factor, self.y() * scale_factor),
            Size::new(self.width() * scale_factor, self.height() * scale_factor),
        ));

        if ctx.hidden {
            builder.set_hidden();
        }

        if let Some(label) = ctx.inherited_label {
            builder.set_label(label);
        } else {
            builder.set_label(&*self.text.value().clone());
        }

        if let Some(hint) = ctx.inherited_hint {
            builder.set_description(hint);
        }

        if let Some(value) = ctx.inherited_value {
            builder.set_value(value);
        }

        builder.set_author_id(format!("{:?}", self.id()));

        ctx.nodes.push(self.id(), builder);

        ctx.children.push(self.id());
    }
}

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>, W: ReadState<T=Wrap>> CommonWidget for Text<T, S, C, FS, FW, W> {
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

pub trait TextWidget: AnyWidget {
    fn text_id(&self) -> TextId;
}

impl<T: ReadState<T=String>, S: ReadState<T=u32>, C: ReadState<T=Style>, FS: ReadState<T=FontStyle>, FW: ReadState<T=FontWeight>, W: ReadState<T=Wrap>> TextWidget for Text<T, S, C, FS, FW, W> {
    fn text_id(&self) -> TextId {
        self.text_id
    }
}

impl AnyWidget for Box<dyn TextWidget> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}

dyn_clone::clone_trait_object!(TextWidget);