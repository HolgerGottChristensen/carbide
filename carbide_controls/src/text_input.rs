use std::ops::{DerefMut, Range};

use copypasta::{ClipboardContext, ClipboardProvider};
use unicode_segmentation::UnicodeSegmentation;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentFontSize};
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseButton, MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent, WindowEvent};
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::layout::{BasicLayouter, Layouter};
use carbide_core::prelude::{EnvironmentColor, Layout};
use carbide_core::{Color, Scalar};
use carbide_core::state::{BoolState, ColorState, F64State, FocusState, LocalState, Map5, ReadState, State, StateExt, StringState, TState, U32State};
use carbide_core::text::Glyph;
use carbide_core::widget::{CommonWidget, CornerRadii, EdgeInsets, HStack, WidgetId, Rectangle, RoundedRectangle, SCALE, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut, ZStack};
use carbide_core::widget::Wrap;

use crate::{PASSWORD_CHAR, PlainTextInput, TextInputState};

/// A plain text input widget. The widget contains no specific styling, other than text color,
/// cursor color/width and selection color. Most common logic has been implemented, such as
/// key shortcuts, mouse click and drag select along with copy and paste. For an example of
/// how to use this widget look at examples/plain_text_input
#[derive(Debug, Clone, Widget)]
pub struct TextInput {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    obscure: Option<char>,
    #[state] text: TextInputState,
    #[state] focus: FocusState,
    #[state] is_error: BoolState,
}

impl TextInput {
    pub fn new<T: Into<TextInputState>>(text: T) -> Box<Self> {
        let text = text.into();
        let focus: FocusState = LocalState::new(Focus::Unfocused).into();

        Self::internal_new(text, None, focus)
    }

    pub fn obscure(mut self) -> Box<Self> {
        self.obscure = Some(PASSWORD_CHAR);
        Self::internal_new(
            self.text,
            self.obscure,
            self.focus,
        )
    }

    pub fn obscure_with_char(mut self, c: char) -> Box<Self> {
        self.obscure = Some(c);
        Self::internal_new(
            self.text,
            self.obscure,
            self.focus,
        )
    }

    fn internal_new(
        text: TextInputState,
        obscure: Option<char>,
        focus: FocusState,
    ) -> Box<Self> {
        let cursor_color: ColorState = EnvironmentColor::Label.into();

        let selection_color: ColorState = EnvironmentColor::Accent.into();
        let darkened_selection_color = selection_color.darkened(0.2);

        let is_error: BoolState = text.clone().into();
        let is_error_stroke: BoolState = is_error.clone();

        let stroke_color = Map5::read_map(
            focus.clone(),
            is_error_stroke.clone(),
            EnvironmentColor::Red.state(),
            EnvironmentColor::Accent.state(),
            EnvironmentColor::OpaqueSeparator.state(),
            |focus: &Focus, is_error: &bool, error_color: &Color, accent_color: &Color, default_color: &Color| {
                if *is_error {
                    *error_color
                } else {
                    match focus {
                        Focus::Focused => *accent_color,
                        _ => *default_color,
                    }
                }
            }).ignore_writes();

        let text_field = if let Some(obscure) = obscure {
            PlainTextInput::new(text.clone())
                .obscure(obscure)
        } else {
            PlainTextInput::new(text.clone())
        };

        let child = ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(EnvironmentColor::SecondarySystemBackground)
                .stroke(stroke_color)
                .stroke_style(1.0),
            text_field
                .focus(focus.clone())
                .cursor_color(cursor_color)
                .selection_color(darkened_selection_color)
                .clip()
                .padding(EdgeInsets::single(0.0, 0.0, 5.0, 5.0)),
        ]).frame(0.0, 22)
            .expand_width();

        Box::new(TextInput {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            obscure,
            text,
            focus,
            is_error,
        })
    }
}

impl CommonWidget for TextInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::single(&self.child)
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn flexibility(&self) -> u32 {
        1
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for TextInput {}