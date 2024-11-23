use std::ops::Range;

use copypasta::{ClipboardContext, ClipboardProvider};
use unicode_segmentation::UnicodeSegmentation;
use carbide::animation::AnimationManager;
use carbide::cursor::MouseCursor;
use carbide::draw::Alignment;

use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Color, Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor, EnvironmentFontSize, IntoColorReadState};
use carbide_core::event::{Ime, Key, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, ModifierKey, MouseEvent, MouseEventContext, MouseEventHandler};
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::layout::{Layout, LayoutContext};
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map2, ReadState, ReadStateExtNew, State};
use carbide_core::text::InnerTextContext;
use carbide_core::widget::{AnyWidget, CommonWidget, Rectangle, Text, TextWidget, Widget, WidgetExt, WidgetId, Wrap};

use crate::{enabled_state, EnabledState};
use crate::plain::cursor::{Cursor, CursorIndex};

pub const PASSWORD_CHAR: char = '●';
pub const PASSWORD_CHAR_SMALL: char = '•';

pub const SCROLL_SUPER_FAST_SPEED: f64 = 4.0;
pub const SCROLL_FAST_SPEED: f64 = 2.0;
pub const SCROLL_SLOW_SPEED: f64 = 1.0;
pub const SCROLL_FAST_WIDTH: f64 = 6.0;
pub const SCROLL_SLOW_WIDTH: f64 = 12.0;

// How editors allows editing:
// Jetbrains IDE: Allow placing cursor at character bounds, even within grapheme clusters. Allows deleting characters even within grapheme clusters (both directions).
// Safari/MacOS: Allow placing cursor at grapheme cluster bounds, delete characters (when not emoji?) in leftward direction, delete graphemes in rightwards direction.
// Firefox: Allow placing cursor at grapheme cluster bounds. Delete characters in leftward direction, delete graphemes in rightwards direction
// Cosmic text editor: Allow placing cursor at grapheme cluster bounds. Delete individual characters.
// Rust playground: Allow placing cursor at character bounds. Allow deleting characters. All selection and offsets get messed up when having funny characters. Does not combine multi character things such as emojis.


// When editing text we have three concepts important to understand:
// * Bytes - a string consist of bytes at the lowest level. When asking for String::len, we get the number of bytes. We dont want to edit individual bytes because it will lead to invalid chars.
// * Chars - a char is a collection of bytes (up to 4 in rust), and represents a character.
// * Graphemes - a collection of chars. Represents visual unicode glyphs.

/// A plain text input widget. The widget contains no specific styling, other than text color,
/// cursor color/width and selection color. Most common logic has been implemented, such as
/// key shortcuts, mouse click and drag select along with copy and paste. For an example of
/// how to use this widget look at examples/plain_text_input
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent, Layout, Render)]
pub struct PlainTextInput<F, C, O, S, T, E> where
    F: State<T=Focus>,
    C: ReadState<T=Color>,
    O: ReadState<T=Option<char>>,
    S: ReadState<T=u32>,
    T: State<T=String>,
    E: ReadState<T=bool>,
{
    // Standard fields
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] focus: F,
    #[state] enabled: E,

    // Widgets
    text_widget: Box<dyn TextWidget>,
    cursor_widget: Box<dyn AnyWidget>,
    selection_widget: Box<dyn AnyWidget>,

    // Text styles
    #[state] text_color: C,
    #[state] obscure_text: O,
    #[state] font_size: S,

    // Text
    #[state] display_text: Box<dyn AnyReadState<T=String>>,
    #[state] text: T,
    #[state] text_offset: LocalState<f64>,

    // Cursor
    cursor: Cursor,
    last_drag_position: Option<Position>,
    current_offset_speed: Option<f64>,

    hovered: bool,
}

impl PlainTextInput<Focus, Color, Option<char>, u32, String, bool> {
    pub fn new<S: IntoState<String>>(text: S) -> PlainTextInput<LocalState<Focus>, impl ReadState<T=Color>, Option<char>, impl ReadState<T=u32>, S::Output, EnabledState> {
        let focus = LocalState::new(Focus::Unfocused);
        let color = EnvironmentColor::Label.color();
        let obscure = None;
        let font_size = EnvironmentFontSize::Body.u32();

        let cursor_widget = Rectangle::new().fill(EnvironmentColor::Green).boxed();
        let selection_widget = Rectangle::new().fill(EnvironmentColor::Purple).boxed();

        Self::new_internal(
            focus,
            color,
            obscure,
            font_size,
            text.into_state(),
            cursor_widget,
            selection_widget,
            enabled_state(),
        )
    }
}

impl<
    F: State<T=Focus>,
    C: ReadState<T=Color>,
    O: ReadState<T=Option<char>>,
    S: ReadState<T=u32>,
    T: State<T=String>,
    E: ReadState<T=bool>,
> PlainTextInput<F, C, O, S, T, E> {
    pub fn enabled<E2: IntoReadState<bool>>(self, enabled: E2) -> PlainTextInput<F, C, O, S, T, E2::Output> {
        Self::new_internal(
            self.focus,
            self.text_color,
            self.obscure_text,
            self.font_size,
            self.text,
            self.cursor_widget,
            self.selection_widget,
            enabled.into_read_state(),
        )
    }

    pub fn focused<F2: IntoState<Focus>>(self, focused: F2) -> PlainTextInput<F2::Output, C, O, S, T, E> {
        Self::new_internal(
            focused.into_state(),
            self.text_color,
            self.obscure_text,
            self.font_size,
            self.text,
            self.cursor_widget,
            self.selection_widget,
            self.enabled,
        )
    }

    pub fn font_size<S2: IntoReadState<u32>>(self, font_size: S2) -> PlainTextInput<F, C, O, S2::Output, T, E> {
        Self::new_internal(
            self.focus,
            self.text_color,
            self.obscure_text,
            font_size.into_read_state(),
            self.text,
            self.cursor_widget,
            self.selection_widget,
            self.enabled,
        )
    }

    pub fn text_color<C2: IntoReadState<Color>>(self, text_color: C2) -> PlainTextInput<F, C2::Output, O, S, T, E> {
        Self::new_internal(
            self.focus,
            text_color.into_read_state(),
            self.obscure_text,
            self.font_size,
            self.text,
            self.cursor_widget,
            self.selection_widget,
            self.enabled,
        )
    }

    pub fn obscure<O2: IntoReadState<Option<char>>>(self, obscure: O2) -> PlainTextInput<F, C, O2::Output, S, T, E> {
        Self::new_internal(
            self.focus,
            self.text_color,
            obscure.into_read_state(),
            self.font_size,
            self.text,
            self.cursor_widget,
            self.selection_widget,
            self.enabled,
        )
    }

    pub fn selection_widget(self, selection: Box<dyn AnyWidget>) -> PlainTextInput<F, C, O, S, T, E> {
        Self::new_internal(
            self.focus,
            self.text_color,
            self.obscure_text,
            self.font_size,
            self.text,
            self.cursor_widget,
            selection,
            self.enabled,
        )
    }

    pub fn cursor_widget(self, cursor: Box<dyn AnyWidget>) -> PlainTextInput<F, C, O, S, T, E> {
        Self::new_internal(
            self.focus,
            self.text_color,
            self.obscure_text,
            self.font_size,
            self.text,
            cursor,
            self.selection_widget,
            self.enabled,
        )
    }

    fn new_internal<
        F2: State<T=Focus>,
        C2: ReadState<T=Color>,
        O2: ReadState<T=Option<char>>,
        S2: ReadState<T=u32>,
        T2: State<T=String>,
        E2: ReadState<T=bool>,
    >(focus: F2, text_color: C2, obscure: O2, font_size: S2, text: T2, cursor_widget: Box<dyn AnyWidget>, selection_widget: Box<dyn AnyWidget>, enabled: E2) -> PlainTextInput<F2, C2, O2, S2, T2, E2> {

        let display_text = Map2::read_map(text.clone(), obscure.clone(), |text, obscure| {
            if let Some(obscuring_char) = obscure {
                text.graphemes(true).map(|_a| obscuring_char).collect::<String>()
            } else {
                text.clone()
            }
        });

        let text_widget = Box::new(Text::new(display_text.clone())
            .font_size(font_size.clone())
            .color(text_color.clone())
            .wrap(Wrap::None));

        let last = usize::MAX;

        PlainTextInput {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            focus,
            enabled,
            text_widget,
            cursor_widget,
            selection_widget,
            text_color,
            obscure_text: obscure,
            font_size,
            //display_text: Box::new(()),
            display_text: display_text.as_dyn_read(),
            text,
            text_offset: LocalState::new(0.0),
            cursor: Cursor::Single(CursorIndex { line: 0, index: last }),
            last_drag_position: None,
            current_offset_speed: None,
            hovered: false,
        }
    }
}

/*impl PlainTextInput {
    pub fn new(text: impl Into<TextInputState>) -> Box<Self> {
        let text = text.into();
        let focus_state = LocalState::new(Focus::Unfocused);
        let font_size: TState<u32> = EnvironmentFontSize::Body.into();

        let selection_color: TState<Color> = EnvironmentColor::Green.into();
        let cursor_color: TState<Color> = EnvironmentColor::Red.into();
        let text_color: TState<Color> = EnvironmentColor::Label.into();

        let obscure_text = None;

        Self::internal_new(
            text,
            font_size,
            focus_state,
            selection_color,
            cursor_color,
            text_color,
            obscure_text,
        )
    }

    pub fn obscure(mut self, obscure: char) -> Box<Self> {
        self.obscure_text = Some(obscure);
        Self::internal_new(
            self.input_state,
            self.font_size,
            self.focus,
            self.selection_color,
            self.cursor_color,
            self.text_color,
            self.obscure_text,
        )
    }

    pub fn focus(mut self, focus: impl Into<TState<Focus>>) -> Box<Self> {
        self.focus = focus.into();
        Self::internal_new(
            self.input_state,
            self.font_size,
            self.focus,
            self.selection_color,
            self.cursor_color,
            self.text_color,
            self.obscure_text,
        )
    }

    pub fn font_size(mut self, font_size: impl Into<TState<u32>>) -> Box<Self> {
        self.font_size = font_size.into();
        Self::internal_new(
            self.input_state,
            self.font_size,
            self.focus,
            self.selection_color,
            self.cursor_color,
            self.text_color,
            self.obscure_text,
        )
    }

    pub fn selection_color(mut self, color: impl Into<TState<Color>>) -> Box<Self> {
        self.selection_color = color.into();
        Self::internal_new(
            self.input_state,
            self.font_size,
            self.focus,
            self.selection_color,
            self.cursor_color,
            self.text_color,
            self.obscure_text,
        )
    }

    pub fn cursor_color(mut self, color: impl Into<TState<Color>>) -> Box<Self> {
        self.cursor_color = color.into();
        Self::internal_new(
            self.input_state,
            self.font_size,
            self.focus,
            self.selection_color,
            self.cursor_color,
            self.text_color,
            self.obscure_text,
        )
    }

    pub fn text_color(mut self, color: impl Into<TState<Color>>) -> Box<Self> {
        self.text_color = color.into();
        Self::internal_new(
            self.input_state,
            self.font_size,
            self.focus,
            self.selection_color,
            self.cursor_color,
            self.text_color,
            self.obscure_text,
        )
    }

    pub fn internal_new(
        input: TextInputState,
        font_size: TState<u32>,
        focus: TState<Focus>,
        selection_color: TState<Color>,
        cursor_color: TState<Color>,
        text_color: TState<Color>,
        obscure_text: Option<char>,
    ) -> Box<Self> {
        let cursor_x = LocalState::new(0.0);
        let selection_x = LocalState::new(0.0);
        let selection_width = LocalState::new(0.0);
        let text_offset = LocalState::new(0.0);

        let is_focused = focus.mapped(|focus: &Focus| focus == &Focus::Focused);

        let display_text: TState<String> = if let Some(obscuring_char) = obscure_text {
            input
                .mapped(move |val: &String| val.chars().map(|c| obscuring_char).collect::<String>())
        } else {
            input.clone().into()
        };

        let child = HStack::new(vec![
            ZStack::new(vec![
                IfElse::new(is_focused.clone()).when_true(
                    Rectangle::new()
                        .fill(selection_color.clone())
                        .frame(
                            selection_width.clone(),
                            font_size.map(|val: &u32| *val as f64).ignore_writes(),
                        )
                        .offset(selection_x.clone(), 0.0),
                ),
                Text::new(display_text.clone())
                    .font_size(font_size.clone())
                    .wrap_mode(Wrap::None)
                    .foreground_color(text_color.clone()),
                IfElse::new(is_focused).when_true(
                    Rectangle::new()
                        .fill(cursor_color.clone())
                        .frame(1.0, font_size.map(|val: &u32| *val as f64).ignore_writes())
                        .offset(cursor_x.clone(), 0.0),
                ),
            ])
            .with_alignment(BasicLayouter::Leading)
            .offset(text_offset.clone(), 0.0),
            Spacer::new(),
        ])
        .frame_fixed_height(30);

        Box::new(PlainTextInput {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            focus,
            cursor: Cursor::Single(CursorIndex { line: 0, char: 0 }),
            drag_start_cursor: None,
            obscure_text,
            selection_color,
            cursor_color,
            text: input.clone().into(),
            display_text,
            cursor_x,
            selection_x,
            selection_width,
            text_offset,
            font_size,
            text_color,
            input_state: input,
        })
    }
}*/

impl<
    F: State<T=Focus>,
    C: ReadState<T=Color>,
    O: ReadState<T=Option<char>>,
    S: ReadState<T=u32>,
    T: State<T=String>,
    E: ReadState<T=bool>,
> KeyboardEventHandler for PlainTextInput<F, C, O, S, T, E> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        //println!("Event: {:#?}", event);
        if self.get_focus() != Focus::Focused || !*self.enabled.value() {
            return;
        }

        let (current_index, _is_selection) = match self.cursor {
            Cursor::Single(cursor_index) => (cursor_index, false),
            Cursor::Selection { end, .. } => (end, true),
        };

        match TextInputKeyCommand::from(event) {
            TextInputKeyCommand::MoveLeft => self.move_left(),
            TextInputKeyCommand::MoveRight => self.move_right(),
            TextInputKeyCommand::SelectLeft => self.select_left(),
            TextInputKeyCommand::SelectRight => self.select_right(),
            TextInputKeyCommand::RemoveLeft => self.remove_left(),
            TextInputKeyCommand::RemoveRight => self.remove_right(),
            TextInputKeyCommand::JumpWordLeft => self.jump_word_left(current_index),
            TextInputKeyCommand::JumpWordRight => self.jump_word_right(current_index),
            TextInputKeyCommand::JumpSelectWordLeft => self.jump_select_word_left(current_index),
            TextInputKeyCommand::JumpSelectWordRight => self.jump_select_word_right(current_index),
            TextInputKeyCommand::RemoveWordLeft => self.remove_word_left(),
            TextInputKeyCommand::RemoveWordRight => self.remove_word_right(),
            TextInputKeyCommand::DuplicateLeft => self.duplicate_left(),
            TextInputKeyCommand::DuplicateRight => self.duplicate_right(),
            TextInputKeyCommand::Cut => self.cut(),
            TextInputKeyCommand::Copy => self.copy(),
            TextInputKeyCommand::Paste => self.paste(),
            TextInputKeyCommand::SelectAll => self.select_all(),
            TextInputKeyCommand::RemoveAll => self.remove_all(),
            TextInputKeyCommand::JumpToLeft => self.jump_to_left(),
            TextInputKeyCommand::JumpToRight => self.jump_to_right(),
            TextInputKeyCommand::JumpSelectToLeft => self.jump_select_to_left(),
            TextInputKeyCommand::JumpSelectToRight => self.jump_select_to_right(),
            TextInputKeyCommand::Enter => self.enter(ctx.env),
            TextInputKeyCommand::Space => self.text(" "),
            TextInputKeyCommand::Text(s, m) => {
                if s.len() == 0 || s.chars().next().unwrap().is_control() || m.contains(ModifierKey::SUPER) {
                    return;
                }

                self.text(s);
            }
            TextInputKeyCommand::Undefined => {}
        }

        //println!("cursor: {:?}", self.cursor);
    }
}

impl<F: State<T=Focus>, C: ReadState<T=Color>, O: ReadState<T=Option<char>>, S: ReadState<T=u32>, T: State<T=String>, E: ReadState<T=bool>> PlainTextInput<F, C, O, S, T, E> {
    fn text(&mut self, s: &str) {
        match self.cursor {
            Cursor::Single(index) => {
                let offset = byte_offset_from_grapheme_index(index.index, &*self.text.value());

                self.insert_str(offset, s);

                let new_offset = ByteOffset(offset.0 + s.len());

                let grapheme_index = grapheme_index_from_byte_offset(new_offset, &*self.text.value());

                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: grapheme_index,
                });
            }
            Cursor::Selection { start, end } => {
                let min = byte_offset_from_grapheme_index(start.index.min(end.index), &*self.text.value());
                let max = byte_offset_from_grapheme_index(start.index.max(end.index), &*self.text.value());

                self.remove_range(min..max);

                self.insert_str(min, s);

                let new_offset = ByteOffset(min.0 + s.len());
                let grapheme_index = grapheme_index_from_byte_offset(new_offset, &*self.text.value());


                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: grapheme_index,
                });
            }
        }
    }

    fn enter(&mut self, _env: &mut Environment) {
        self.set_focus(Focus::Unfocused);
    }

    fn jump_select_to_right(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                self.cursor = Cursor::Selection {
                    start: index,
                    end: CursorIndex {
                        line: 0,
                        index: count(&*self.display_text.value()),
                    },
                }
            }
            Cursor::Selection { start, .. } => {
                self.cursor = Cursor::Selection {
                    start,
                    end: CursorIndex {
                        line: 0,
                        index: count(&*self.display_text.value()),
                    },
                }
            }
        }
    }

    fn jump_select_to_left(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                self.cursor = Cursor::Selection {
                    start: index,
                    end: CursorIndex { line: 0, index: 0 },
                }
            }
            Cursor::Selection { start, .. } => {
                self.cursor = Cursor::Selection {
                    start,
                    end: CursorIndex { line: 0, index: 0 },
                }
            }
        }
    }

    fn jump_to_right(&mut self) {
        self.cursor = Cursor::Single(CursorIndex {
            line: 0,
            index: count(&*self.display_text.value()),
        })
    }

    fn jump_to_left(&mut self) {
        self.cursor = Cursor::Single(CursorIndex { line: 0, index: 0 })
    }

    /// Clears all text, and sets the cursor to index 0
    fn remove_all(&mut self) {
        self.text.set_value("".to_string());
        self.cursor = Cursor::Single(CursorIndex { line: 0, index: 0 })
    }

    /// Select all text, with the cursor ending at the end of the text
    fn select_all(&mut self) {
        let count = count(&*self.display_text.value());

        if count == 0 {
            self.cursor = Cursor::Single(CursorIndex { line: 0, index: 0 })
        } else {
            self.cursor = Cursor::Selection {
                start: CursorIndex { line: 0, index: 0 },
                end: CursorIndex {
                    line: 0,
                    index: count,
                },
            }
        }
    }

    /// Cut the selection text, or the full text if nothing was selected
    fn cut(&mut self) {
        let mut ctx = ClipboardContext::new().unwrap();

        match self.cursor {
            Cursor::Single(_) => {
                ctx.set_contents(self.display_text.value().to_string())
                    .unwrap();
                self.text.set_value("".to_string());

                self.cursor = Cursor::Single(CursorIndex { line: 0, index: 0 })
            }
            Cursor::Selection { start, end } => {
                let min = byte_offset_from_grapheme_index(start.index.min(end.index), &*self.display_text.value());
                let max = byte_offset_from_grapheme_index(start.index.max(end.index), &*self.display_text.value());

                let s = self.display_text.value()[min.0..max.0].to_string();
                ctx.set_contents(s).unwrap();
                self.remove_range(min..max);

                self.cursor = Cursor::Single(CursorIndex { line: 0, index: start.index.min(end.index) })
            }
        }
    }

    /// Paste the text from the clipboard, replacing the current selection if possible
    fn paste(&mut self) {
        let mut ctx = ClipboardContext::new().unwrap();

        let mut content = ctx.get_contents().unwrap();

        // Remove newlines from the pasted text
        content.retain(|c| c != '\n');

        match self.cursor {
            Cursor::Single(index) => {
                let byte_offset = byte_offset_from_grapheme_index(index.index, &*self.text.value());
                self.insert_str(byte_offset, &content);

                let new_offset = ByteOffset(byte_offset.0 + content.len());

                let grapheme_index = grapheme_index_from_byte_offset(new_offset, &*self.text.value());

                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: grapheme_index,
                });
            }
            Cursor::Selection { start, end } => {
                let min = byte_offset_from_grapheme_index(start.index.min(end.index), &*self.text.value());
                let max = byte_offset_from_grapheme_index(start.index.max(end.index), &*self.text.value());

                self.remove_range(min..max);

                self.insert_str(min, &content);

                let new_offset = ByteOffset(min.0 + content.len());

                let grapheme_index = grapheme_index_from_byte_offset(new_offset, &*self.text.value());

                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: grapheme_index,
                });
            }
        }
    }

    /// Copy the selected text, or the full text if nothing was selected
    fn copy(&mut self) {
        let mut ctx = ClipboardContext::new().unwrap();

        match self.cursor {
            Cursor::Single(_) => {
                ctx.set_contents(self.display_text.value().clone()).unwrap();
            }
            Cursor::Selection { start, end } => {
                let min = byte_offset_from_grapheme_index(start.index.min(end.index), &*self.display_text.value());
                let max = byte_offset_from_grapheme_index(start.index.max(end.index), &*self.display_text.value());

                let s = self.display_text.value()[min.0..max.0].to_string();
                ctx.set_contents(s).unwrap();
            }
        }
    }

    /// Duplicates the selection to the right, or the whole text if nothing was selected
    fn duplicate_right(&mut self) {
        match self.cursor {
            Cursor::Single(_) => {
                let text = self.text.value().clone();
                self.push_str(&text);

                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: count(&*self.text.value()),
                })
            }
            Cursor::Selection { start, end } => {
                let text = self.text.value().clone();

                let min = byte_offset_from_grapheme_index(start.index.min(end.index), &*self.text.value());
                let max = byte_offset_from_grapheme_index(start.index.max(end.index), &*self.text.value());

                self.insert_str(max, &text[min.0..max.0]);

                let index = ByteOffset(max.0 + (min.0..max.0).count());

                let grapheme_index = grapheme_index_from_byte_offset(index, &*self.text.value());

                self.cursor = Cursor::Selection {
                    start: CursorIndex {
                        line: 0,
                        index: end.index,
                    },
                    end: CursorIndex {
                        line: 0,
                        index: grapheme_index,
                    },
                }
            }
        }
    }

    /// Duplicates the selection to the left, or the whole text if nothing was selected
    fn duplicate_left(&mut self) {
        match self.cursor {
            Cursor::Single(_) => {
                let text = self.text.value().clone();
                self.push_str(&text);
            }
            Cursor::Selection { start, end } => {
                let text = self.text.value().clone();
                let min = byte_offset_from_grapheme_index(start.index.min(end.index), &*self.text.value());
                let max = byte_offset_from_grapheme_index(start.index.max(end.index), &*self.text.value());

                self.insert_str(max, &text[min.0..max.0]);
            }
        }
    }

    /// If we have a cursor and not a selection, we remove rightwards until we see either a space or the end of the text
    fn remove_word_right(&mut self) {
        if let Cursor::Single(index) = self.cursor {
            let end_index = next_grapheme_index(index.index, &*self.display_text.value(), true, |s| s == " ");

            let min = byte_offset_from_grapheme_index(index.index, &*self.text.value());
            let max = byte_offset_from_grapheme_index(end_index, &*self.text.value());

            self.remove_range(min..max);
        }
    }

    /// If we have a cursor and not a selection, we remove leftwards until we see either a space or the start of the text
    fn remove_word_left(&mut self) {
        if let Cursor::Single(index) = self.cursor {
            let start_index = prev_grapheme_index(index.index, &*self.display_text.value(), true, |s| s == " ");

            let min = byte_offset_from_grapheme_index(start_index, &*self.text.value());
            let max = byte_offset_from_grapheme_index(index.index, &*self.text.value());

            self.remove_range(min..max);

            self.cursor = Cursor::Single(CursorIndex {
                line: 0,
                index: start_index,
            })
        }
    }

    /// Creates or extends a selection to the next space to the right or the end of the text
    fn jump_select_word_right(&mut self, current_index: CursorIndex) {
        let index = next_grapheme_index(current_index.index, &*self.display_text.value(), true, |s| s == " ");

        let start = match self.cursor {
            Cursor::Single(index) => index,
            Cursor::Selection { start, .. } => start
        };

        if start.index == index {
            self.cursor = Cursor::Single(start);
        } else {
            self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, index} };
        }
    }

    /// Creates or extends a selection to the next space to the left, or the start of the text
    fn jump_select_word_left(&mut self, current_index: CursorIndex) {
        let index = prev_grapheme_index(current_index.index, &*self.display_text.value(), true, |s| s == " ");

        let start = match self.cursor {
            Cursor::Single(index) => index,
            Cursor::Selection { start, .. } => start
        };

        if start.index == index {
            self.cursor = Cursor::Single(start);
        } else {
            self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, index} };
        }
    }

    /// Moves the cursor to the right to either we see a space or the end of the text
    fn jump_word_right(&mut self, current_index: CursorIndex) {
        self.cursor = Cursor::Single(CursorIndex {
            line: 0,
            index: next_grapheme_index(current_index.index, &*self.display_text.value(), true, |s| s == " ")
        });
    }

    /// Moves the cursor to the left to either we see a space or the start of the text
    fn jump_word_left(&mut self, current_index: CursorIndex) {
        self.cursor = Cursor::Single(CursorIndex {
            line: 0,
            index: prev_grapheme_index(current_index.index, &*self.display_text.value(), true, |s| s == " ")
        });
    }

    /// Remove the grapheme to the right of the cursor if any exist
    fn remove_right(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                let min = byte_offset_from_grapheme_index(index.index, &*self.text.value());
                let max = byte_offset_from_grapheme_index(index.index + 1, &*self.text.value());

                self.remove_range(min..max);
            }
            Cursor::Selection { start, end } => {
                let min = byte_offset_from_grapheme_index(start.index.min(end.index), &*self.text.value());
                let max = byte_offset_from_grapheme_index(start.index.max(end.index), &*self.text.value());

                self.remove_range(min..max);

                self.cursor = Cursor::Single(CursorIndex { line: 0, index: start.index.min(end.index) });
            }
        }
    }

    /// Remove the grapheme to the left of the cursor if any exist
    fn remove_left(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                let index = byte_offset_from_grapheme_index(index.index, &*self.text.value());

                let mut prev_byte_offset = ByteOffset(0);

                for (i, _) in self.text.value().char_indices() { // .grapheme_indices(true)
                    if i < index.0 {
                        prev_byte_offset = ByteOffset(i);
                    } else {
                        break;
                    }
                }

                self.remove_range(prev_byte_offset..index);

                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: grapheme_index_from_byte_offset(prev_byte_offset, &*self.text.value()),
                });
            }
            Cursor::Selection { start, end } => {
                let min = byte_offset_from_grapheme_index(start.index.min(end.index), &*self.text.value());
                let max = byte_offset_from_grapheme_index(start.index.max(end.index), &*self.text.value());

                self.remove_range(min..max);

                self.cursor = Cursor::Single(CursorIndex { line: 0, index: start.index.min(end.index) });
            }
        }
    }

    /// Select with the start being at the current cursor position and the end being to the right
    /// or move the current end point of the selection being moved right one grapheme
    fn select_right(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                self.cursor = Cursor::Selection {
                    start: index,
                    end: CursorIndex {
                        line: 0,
                        index: index.index.saturating_add(1),
                    },
                }
            }
            Cursor::Selection { start, end } => {
                let new_index = end.index.saturating_add(1);

                if start.index == new_index {
                    self.cursor = Cursor::Single(start)
                } else {
                    self.cursor = Cursor::Selection {
                        start,
                        end: CursorIndex {
                            line: 0,
                            index: new_index,
                        },
                    }
                }
            }
        }
    }

    /// Select with the start being at the current cursor position and the end being to the left
    /// or move the current end point of the selection being moved left one grapheme
    fn select_left(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                self.cursor = Cursor::Selection {
                    start: index,
                    end: CursorIndex {
                        line: 0,
                        index: index.index.saturating_sub(1),
                    },
                }
            }
            Cursor::Selection { start, end } => {
                let new_index = end.index.saturating_sub(1);

                if start.index == new_index {
                    self.cursor = Cursor::Single(start)
                } else {
                    self.cursor = Cursor::Selection {
                        start,
                        end: CursorIndex {
                            line: 0,
                            index: new_index,
                        },
                    }
                }
            }
        }
    }

    /// Move the cursor right one grapheme
    fn move_right(&mut self) {
        match self.cursor {
            Cursor::Single(current_index) => {

                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: current_index.index.saturating_add(1).min(count(&*self.display_text.value())),
                });
            }
            Cursor::Selection { start, end } => {
                let max = start.index.max(end.index);
                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: max,
                });
            }
        }
    }

    /// Move the cursor left one grapheme
    fn move_left(&mut self) {
        match self.cursor {
            Cursor::Single(current_index) => {
                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: current_index.index.saturating_sub(1),
                });
            }
            Cursor::Selection { start, end } => {
                let min = start.index.min(end.index);
                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: min,
                });
            }
        }
    }
}

// General text modification actions
impl<F: State<T=Focus>, C: ReadState<T=Color>, O: ReadState<T=Option<char>>, S: ReadState<T=u32>, T: State<T=String>, E: ReadState<T=bool>> PlainTextInput<F, C, O, S, T, E> {
    /// Insert a string at a given byte offset.
    fn insert_str(&mut self, byte_offset: ByteOffset, string: &str) {
        self.text.value_mut().insert_str(byte_offset.0, string);
    }

    /// Push a string to the end of the input
    fn push_str(&mut self, string: &str) {
        self.text.value_mut().push_str(string);
    }

    /// Remove all the graphemes inside the range,
    fn remove_range(&mut self, byte_range: Range<ByteOffset>) {
        self.text.value_mut().replace_range(byte_range.start.0..byte_range.end.0, "");
    }

    /// Clamp the cursor to within the number of chars in the displayed text.
    /// The state should be up to date before calling this method, especially the display_text state.
    fn clamp_cursor(&mut self) {
        let count = count(&*self.display_text.value());

        match self.cursor {
            Cursor::Single(CursorIndex{ line, index }) => {
                self.cursor = Cursor::Single(CursorIndex { line, index: index.min(count)});
            }
            Cursor::Selection {
                start: CursorIndex { line: line_start, index: index_start },
                end: CursorIndex { line: line_end, index: index_end },
            } => {
                self.cursor = Cursor::Selection {
                    start: CursorIndex { line: line_start, index: index_start.min(count) },
                    end: CursorIndex { line: line_end, index: index_end.min(count) },
                }
            }
        }
    }

    fn update_offset_with_speed_to_make_cursor_visible(&mut self, speed: f64, _ctx: &mut dyn InnerTextContext) {
        let mut current_offset = *self.text_offset.value();

        current_offset += speed;

        current_offset = current_offset
            .max(self.width() - self.text_widget.width() - self.cursor_widget.width())
            .min(0.0);

        self.text_offset.set_value(current_offset);
    }

        /// Update the current scroll offset to make the cursor visible within the text field if possible
    fn update_offset_to_make_cursor_visible(&mut self, ctx: &mut dyn InnerTextContext) {
        let mut current_offset = *self.text_offset.value();

        if self.get_focus() == Focus::Focused {
            // We should try to keep this index within view as long as the field is focused
            let index = match self.cursor {
                Cursor::Single(index) => index,
                Cursor::Selection { end, .. } => end,
            };

            let text_id = self.text_widget.text_id();
            let cursor_offset_from_text_origin = ctx.position_of(text_id, 0, index.index).x;

            //println!("cursor_offset_from_text_origin: {:?}", cursor_offset_from_text_origin);
            //println!("tolerance: {:?}", tolerance_difference.x());
            //println!("width: {:?}", self.width());
            //println!("text_width: {:?}", self.text_widget.width());
            //println!("x: {:?}", self.x());
            //println!("text_x: {:?}", self.text_widget.x());

            //println!("current_offset: {:?}", current_offset);

            //dbg!(cursor_offset_from_text_origin + self.cursor_widget.width());
            //dbg!(cursor_offset_from_text_origin + self.cursor_widget.width() + current_offset - self.width());
            if cursor_offset_from_text_origin + self.cursor_widget.width() + current_offset > self.width() {
                //println!("Current offset above width");
                current_offset -= cursor_offset_from_text_origin + self.cursor_widget.width() + current_offset - self.width();
            }

            //dbg!(cursor_offset_from_text_origin + current_offset);
            if cursor_offset_from_text_origin + current_offset < 0.0 {
                //println!("Current offset below 0");
                current_offset -= cursor_offset_from_text_origin + current_offset;
            }
        }

        // Clamp the offset to be within the bounds of the visible area.
        current_offset = current_offset
            .max(self.width() - self.text_widget.width() - self.cursor_widget.width())
            .min(0.0);

        self.text_offset.set_value(current_offset);
        //println!("new_offset: {:?}\n", *self.text_offset.value());
    }
}

impl<
    F: State<T=Focus>,
    C: ReadState<T=Color>,
    O: ReadState<T=Option<char>>,
    S: ReadState<T=u32>,
    T: State<T=String>,
    E: ReadState<T=bool>,
> MouseEventHandler for PlainTextInput<F, C, O, S, T, E> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        let enabled = *self.enabled.value();
        let editable = enabled && self.get_focus() == Focus::Focused;

        match event {
            MouseEvent::Move { to, .. } => {
                self.hovered = self.is_inside(*to);
            }
            MouseEvent::Press { .. } if !self.is_inside(event.get_current_mouse_position()) => {
                if self.get_focus() == Focus::Focused {
                    self.set_focus(Focus::Unfocused);
                }
            }
            MouseEvent::Press { position, modifiers: ModifierKey::EMPTY, .. } if enabled => self.text_click(position, ctx),
            MouseEvent::Release { .. } => {
                self.current_offset_speed = None;
                self.last_drag_position = None;
            }
            //MouseEvent::Click(_, position, ModifierKey::NO_MODIFIER) => self.text_click(position, env),
            MouseEvent::Click(_, position, ModifierKey::SHIFT) if editable => self.selection_click(position, ctx),
            MouseEvent::NClick(_, _, _, n) if n % 2 == 1 && editable => self.select_all(),
            MouseEvent::NClick(_, position, _, n) if n % 2 == 0 && editable => self.select_word_at_click(position, ctx),
            MouseEvent::Drag { to, delta_xy, .. } => {
                if !enabled {
                    return;
                }

                if self.last_drag_position.is_some() || self.is_inside(event.get_current_mouse_position()) {
                    self.last_drag_position = Some(*to);
                    self.drag_selection(ctx.text, to, delta_xy);
                }
            }
            _ => (),
        }
    }
}

impl<F: State<T=Focus>, C: ReadState<T=Color>, O: ReadState<T=Option<char>>, S: ReadState<T=u32>, T: State<T=String>, E: ReadState<T=bool>> PlainTextInput<F, C, O, S, T, E> {
    fn drag_selection(&mut self, ctx: &mut dyn InnerTextContext, to: &Position, _delta_xy: &Position) {
        if to.x - self.x() < 0.0 {
            self.current_offset_speed = Some(SCROLL_SUPER_FAST_SPEED);
        } else if (self.x() + self.width()) - to.x < 0.0 {
            self.current_offset_speed = Some(-SCROLL_SUPER_FAST_SPEED);
        } else if to.x - self.x() < SCROLL_FAST_WIDTH {
            self.current_offset_speed = Some(SCROLL_FAST_SPEED);
        } else if (self.x() + self.width()) - to.x < SCROLL_FAST_WIDTH {
            self.current_offset_speed = Some(-SCROLL_FAST_SPEED);
        } else if to.x - self.x() < SCROLL_SLOW_WIDTH {
            self.current_offset_speed = Some(SCROLL_SLOW_SPEED);
        } else if (self.x() + self.width()) - to.x < SCROLL_SLOW_WIDTH {
            self.current_offset_speed = Some(-SCROLL_SLOW_SPEED);
        } else {
            self.current_offset_speed = None;
        }

        let x = to.x - self.position.x - *self.text_offset.value();
        let (_, index) = ctx.hit(self.text_widget.text_id(), Position::new(x, self.text_widget.height() / 2.0));


        match self.cursor {
            Cursor::Single(start) | Cursor::Selection { start, .. } => {
                if start.index == index {
                    self.cursor = Cursor::Single(start);
                } else {
                    self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, index } };
                }
            }
        }
    }

    fn text_click(&mut self, position: &Position, ctx: &mut MouseEventContext) {
        if self.get_focus() == Focus::Unfocused {
            self.request_focus(ctx.env_stack);
        }

        let x = position.x - self.position.x - *self.text_offset.value();
        let (line, index) = ctx.text.hit(self.text_widget.text_id(), Position::new(x, self.text_widget.height() / 2.0));

        self.cursor = Cursor::Single(CursorIndex {
            line,
            index,
        });
    }

    fn selection_click(&mut self, position: &Position, ctx: &mut MouseEventContext) {
        let x = position.x - self.position.x - *self.text_offset.value();
        let (line, clicked_index) = ctx.text.hit(self.text_widget.text_id(), Position::new(x, self.text_widget.height() / 2.0));

        match self.cursor {
            Cursor::Single(CursorIndex { line: _, index }) => {
                self.cursor = Cursor::Selection {
                    start: CursorIndex { line: 0, index },
                    end: CursorIndex {
                        line,
                        index: clicked_index,
                    },
                }
            }
            Cursor::Selection {
                start: CursorIndex { index, .. },
                ..
            } => {
                self.cursor = Cursor::Selection {
                    start: CursorIndex { line: 0, index },
                    end: CursorIndex {
                        line,
                        index: clicked_index,
                    },
                }
            }
        }
    }

    fn select_word_at_click(&mut self, position: &Position, ctx: &mut MouseEventContext) {
        let x = position.x - self.position.x - *self.text_offset.value();
        let (line, clicked_index) = ctx.text.hit(self.text_widget.text_id(), Position::new(x, self.text_widget.height() / 2.0));

        let range = word_range_surrounding_grapheme_index(clicked_index, &self.display_text.value());

        self.cursor = Cursor::Selection {
            start: CursorIndex {
                line,
                index: range.start,
            },
            end: CursorIndex {
                line,
                index: range.end,
            },
        }
    }
}

impl<
    F: State<T=Focus>,
    C: ReadState<T=Color>,
    O: ReadState<T=Option<char>>,
    S: ReadState<T=u32>,
    T: State<T=String>,
    E: ReadState<T=bool>,
> Layout for PlainTextInput<F, C, O, S, T, E> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.clamp_cursor();

        //println!("calculate size");
        if let Some(position) = self.last_drag_position {
            self.drag_selection(ctx.text, &position, &position);
        }

        let text_dimensions = self.text_widget.calculate_size(requested_size, ctx);

        // Calculate size for selection indicator
        match self.cursor {
            Cursor::Single(_) => {
                self.cursor_widget.calculate_size(Dimension::new(1.0, text_dimensions.height), ctx);
            }
            Cursor::Selection { start, end } => {
                let text_id = self.text_widget.text_id();
                let start_x = ctx.text.position_of(text_id, 0, start.index).x + self.x();
                let end_x = ctx.text.position_of(text_id, 0, end.index).x + self.x();

                let min = start_x.min(end_x);
                let max = start_x.max(end_x);

                self.cursor_widget.calculate_size(Dimension::new(1.0, text_dimensions.height), ctx);
                self.selection_widget.calculate_size(Dimension::new(max - min, text_dimensions.height), ctx);
            }
        }

        self.set_dimension(requested_size);
        self.dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        // The process of positioning
        // 1. We need to position the text as is, to be able to use the positions of the glyphs.
        //    This also helps if new letters are added since last positioning. All this is
        //    calculated using the current offset.
        // 2. We calculate the new offset, by looking if the cursor as calculated before is
        //    outside the visible area on either side.

        let position = self.position + Position::new(*self.text_offset.value(), 0.0);
        let dimension = self.dimension;

        self.text_widget.set_position(Alignment::Leading.position(position, dimension, self.text_widget.dimension()));
        self.text_widget.position_children(ctx);

        //println!("Position children called");
        if let Some(speed) = self.current_offset_speed {
            self.update_offset_with_speed_to_make_cursor_visible(speed, ctx.text);
            if let Some(manager) = ctx.env_stack.get_mut::<AnimationManager>() {
                manager.request_animation_frame();
            }
        } else {
            self.update_offset_to_make_cursor_visible(ctx.text);
        }


        let position = self.position + Position::new(*self.text_offset.value(), 0.0);
        let dimension = self.dimension;

        self.text_widget.set_position(Alignment::Leading.position(position, dimension, self.text_widget.dimension()));
        self.text_widget.position_children(ctx);

        if self.get_focus() == Focus::Focused && *self.enabled.value() {
            let text_id = self.text_widget.text_id();

            match self.cursor {
                Cursor::Single(index) => {
                    let x = ctx.text.position_of(text_id, 0, index.index).x + self.x() + *self.text_offset.value();

                    self.cursor_widget.set_position(Position::new(x, self.text_widget.y()));
                    self.cursor_widget.position_children(ctx);
                }
                Cursor::Selection { start, end } => {
                    let end_x = ctx.text.position_of(text_id, 0, end.index).x + self.x() + *self.text_offset.value();
                    let min_x = ctx.text.position_of(text_id, 0, start.index.min(end.index)).x + self.x() + *self.text_offset.value();

                    self.cursor_widget.set_position(Position::new(end_x, self.text_widget.y()));
                    self.cursor_widget.position_children(ctx);

                    self.selection_widget.set_position(Position::new(min_x, self.text_widget.y()));
                    self.selection_widget.position_children(ctx);
                }
            }
        }
    }
}

impl<
    F: State<T=Focus>,
    C: ReadState<T=Color>,
    O: ReadState<T=Option<char>>,
    S: ReadState<T=u32>,
    T: State<T=String>,
    E: ReadState<T=bool>,
> Render for PlainTextInput<F, C, O, S, T, E> {
    fn render(&mut self, context: &mut RenderContext) {
        if let Some(cursor) = self.cursor() {
            context.env.set_cursor(cursor);
        }

        if self.get_focus() == Focus::Focused && *self.enabled.value() {
            match self.cursor {
                Cursor::Single(_) => {
                    self.text_widget.render(context);
                    self.cursor_widget.render(context);
                }
                Cursor::Selection { .. } => {
                    self.selection_widget.render(context);
                    self.text_widget.render(context);
                    self.cursor_widget.render(context);
                }
            }
        } else {
            self.text_widget.render(context);
        }
    }
}

impl<
    F: State<T=Focus>,
    C: ReadState<T=Color>,
    O: ReadState<T=Option<char>>,
    S: ReadState<T=u32>,
    T: State<T=String>,
    E: ReadState<T=bool>,
> CommonWidget for PlainTextInput<F, C, O, S, T, E> {
    fn cursor(&self) -> Option<MouseCursor> {
        if self.hovered {
            Some(MouseCursor::Text)
        } else {
            None
        }
    }

    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 1, focus: self.focus);
}


// ---------------------------------------------------
//  Key commands
// ---------------------------------------------------
pub(super) enum TextInputKeyCommand<'a> {
    Text(&'a String, ModifierKey),
    MoveLeft,
    MoveRight,
    SelectLeft,
    SelectRight,
    RemoveLeft,
    RemoveRight,
    JumpWordLeft,
    JumpWordRight,
    JumpSelectWordLeft,
    JumpSelectWordRight,
    RemoveWordLeft,
    RemoveWordRight,
    DuplicateLeft,
    DuplicateRight,
    Copy,
    Paste,
    Cut,
    SelectAll,
    RemoveAll,
    JumpToLeft,
    JumpToRight,
    JumpSelectToLeft,
    JumpSelectToRight,
    Enter,
    Space,
    Undefined,
}

impl<'a> From<&'a KeyboardEvent> for TextInputKeyCommand<'a> {
    fn from(value: &'a KeyboardEvent) -> Self {
        match value {
            KeyboardEvent::Press(Key::ArrowLeft, ModifierKey::EMPTY) => TextInputKeyCommand::MoveLeft,
            KeyboardEvent::Press(Key::ArrowLeft, ModifierKey::SHIFT) => TextInputKeyCommand::SelectLeft,
            KeyboardEvent::Press(Key::ArrowLeft, ModifierKey::ALT) => TextInputKeyCommand::JumpWordLeft,
            KeyboardEvent::Press(Key::ArrowLeft, ModifierKey::SUPER) => TextInputKeyCommand::JumpToLeft,
            KeyboardEvent::Press(Key::ArrowLeft, ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordLeft,
            KeyboardEvent::Press(Key::ArrowLeft, ModifierKey::SHIFT_SUPER) => TextInputKeyCommand::JumpSelectToLeft,

            KeyboardEvent::Press(Key::ArrowRight, ModifierKey::EMPTY) => TextInputKeyCommand::MoveRight,
            KeyboardEvent::Press(Key::ArrowRight, ModifierKey::SHIFT) => TextInputKeyCommand::SelectRight,
            KeyboardEvent::Press(Key::ArrowRight, ModifierKey::ALT) => TextInputKeyCommand::JumpWordRight,
            KeyboardEvent::Press(Key::ArrowRight, ModifierKey::SUPER) => TextInputKeyCommand::JumpToRight,
            KeyboardEvent::Press(Key::ArrowRight, ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordRight,
            KeyboardEvent::Press(Key::ArrowRight, ModifierKey::SHIFT_SUPER) => TextInputKeyCommand::JumpSelectToRight,

            KeyboardEvent::Press(Key::Backspace, ModifierKey::EMPTY) => TextInputKeyCommand::RemoveLeft,
            KeyboardEvent::Press(Key::Backspace, ModifierKey::SHIFT) => TextInputKeyCommand::RemoveLeft,
            KeyboardEvent::Press(Key::Backspace, ModifierKey::ALT) => TextInputKeyCommand::RemoveWordLeft,

            KeyboardEvent::Press(Key::Delete, ModifierKey::EMPTY) => TextInputKeyCommand::RemoveRight,
            KeyboardEvent::Press(Key::Delete, ModifierKey::SHIFT) => TextInputKeyCommand::RemoveAll,
            KeyboardEvent::Press(Key::Delete, ModifierKey::ALT) => TextInputKeyCommand::RemoveWordRight,

            KeyboardEvent::Press(Key::Character(c), ModifierKey::SUPER) if c == "c" => TextInputKeyCommand::Copy,
            KeyboardEvent::Press(Key::Character(c), ModifierKey::SUPER) if c == "v" => TextInputKeyCommand::Paste,
            KeyboardEvent::Press(Key::Character(c), ModifierKey::SUPER) if c == "x" => TextInputKeyCommand::Cut,
            KeyboardEvent::Press(Key::Character(c), ModifierKey::SUPER) if c == "a" => TextInputKeyCommand::SelectAll,
            KeyboardEvent::Press(Key::Character(c), ModifierKey::SUPER) if c == "d" => TextInputKeyCommand::DuplicateRight,
            KeyboardEvent::Press(Key::Character(c), ModifierKey::SHIFT_SUPER) if c == "d" => TextInputKeyCommand::DuplicateLeft,

            KeyboardEvent::Press(Key::Home, ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToLeft,
            KeyboardEvent::Press(Key::End, ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToRight,
            KeyboardEvent::Press(Key::Enter, ModifierKey::EMPTY) => TextInputKeyCommand::Enter,
            KeyboardEvent::Press(Key::Space, ModifierKey::EMPTY) => TextInputKeyCommand::Space,

            KeyboardEvent::Press(Key::Character(s), m) => TextInputKeyCommand::Text(s, *m),
            KeyboardEvent::Ime(Ime::Commit(s)) => TextInputKeyCommand::Text(s, ModifierKey::EMPTY),

            _ => TextInputKeyCommand::Undefined,
        }
    }
}


// ---------------------------------------------------
//  Utilities
// ---------------------------------------------------
#[derive(Copy, Clone, Debug)]
struct ByteOffset(usize);

fn prev_grapheme_index(grapheme_index: usize, text: &str, skip_initial: bool, f: fn(&str) ->bool) -> usize {
    text
        .grapheme_indices(true)
        .rev()
        .enumerate()
        .skip(count(text) - grapheme_index)
        .skip_while(|(_, (_, s))| f(*s) && skip_initial)
        .find(|(_, (_, c))| f(*c))
        .map_or(0, |(e, _)| count(text) - e)
}

fn next_grapheme_index(grapheme_index: usize, text: &str, skip_initial: bool, f: fn(&str) ->bool) -> usize {
    text
        .grapheme_indices(true)
        .enumerate()
        .skip(grapheme_index)
        .skip_while(|(_, (_, s))| f(*s) && skip_initial)
        .find(|(_, (_, c))| f(*c))
        .map_or(count(text), |(e, _)| e)
}

/// Returns a range of grapheme indexes for either the word at the current grapheme index
/// or the range of spaces surrounding the current grapheme index
fn word_range_surrounding_grapheme_index(grapheme_index: usize, text: &str) -> Range<usize> {
    let min = prev_grapheme_index(grapheme_index, text, false, |s| s == " ");
    let max = next_grapheme_index(grapheme_index, text, false, |s| s == " ");

    if min == max {
        let min = prev_grapheme_index(grapheme_index, text, false, |s| s != " ");
        let max = next_grapheme_index(grapheme_index, text, false, |s| s != " ");

        return min..max
    }

    min..max
}

fn count(s: &str) -> usize {
    s.graphemes(true).count()
}

fn byte_offset_from_grapheme_index(index: usize, string: &str) -> ByteOffset {
    string
        .grapheme_indices(true)
        .skip(index)
        .map(|(i, _)| ByteOffset(i))
        .next()
        .unwrap_or(ByteOffset(string.len()))
}

fn grapheme_index_from_byte_offset(index: ByteOffset, string: &str) -> usize {
    for (i, (g, _)) in string.grapheme_indices(true).enumerate() {
        if g >= index.0 {
            return i
        }
    }

    count(string)
}


// ---------------------------------------------------
//  Tests
// ---------------------------------------------------
#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use carbide::state::{LocalState, ReadState};

    use crate::plain::cursor::Cursor;
    use crate::plain::cursor::CursorIndex;
    use crate::PlainTextInput;

    #[test]
    fn hello_world() {
        let input = PlainTextInput::new("Hello world!".to_string());
    }

    #[test]
    fn select_all_non_empty1() {
        let mut input = PlainTextInput::new("Hello world!".to_string());
        input.select_all();
        assert_eq!(input.text, "Hello world!".to_string());
        assert_matches!(input.cursor, Cursor::Selection {
            start: CursorIndex {
                line: 0,
                index: 0
            },
            end: CursorIndex {
                line: 0,
                index: 12,
            },
        });
    }

    #[test]
    fn select_all_non_empty2() {
        let mut input = PlainTextInput::new("ধারা ১ সমস্ত মানুষ".to_string());
        input.select_all();
        assert_eq!(input.text, "ধারা ১ সমস্ত মানুষ".to_string());
        assert_matches!(input.cursor, Cursor::Selection {
            start: CursorIndex {
                line: 0,
                index: 0
            },
            end: CursorIndex {
                line: 0,
                index: 13,
            },
        });
    }

    #[test]
    fn select_all_empty() {
        let mut input = PlainTextInput::new("".to_string());
        input.select_all();
        assert_eq!(input.text, String::new());
        assert_matches!(input.cursor, Cursor::Single(CursorIndex { line: 0, index: 0 }));

        let mut input = PlainTextInput::new("".to_string())
            .obscure('0');
        input.select_all();
        assert_eq!(input.text, String::new());
        assert_matches!(input.cursor, Cursor::Single(CursorIndex { line: 0, index: 0 }));
    }

    #[test]
    fn remove_all_becomes_empty() {
        let mut input = PlainTextInput::new("".to_string());
        input.remove_all();
        assert_eq!(input.text, String::new());
        assert_matches!(input.cursor, Cursor::Single(CursorIndex { line: 0, index: 0 }));

        let mut input = PlainTextInput::new("Hello world!".to_string());
        input.remove_all();
        assert_eq!(input.text, String::new());
        assert_matches!(input.cursor, Cursor::Single(CursorIndex { line: 0, index: 0 }));

        let mut input = PlainTextInput::new("ধারা ১ সমস্ত মানুষ".to_string());
        input.remove_all();
        assert_eq!(input.text, String::new());
        assert_matches!(input.cursor, Cursor::Single(CursorIndex { line: 0, index: 0 }));

        let mut input = PlainTextInput::new(LocalState::new("Hello world!".to_string()))
            .obscure('0');
        assert_eq!(&*input.display_text.value(), "000000000000");
        input.remove_all();
        assert_eq!(&*input.display_text.value(), "");
        assert_eq!(&*input.text.value(), "");
        assert_matches!(input.cursor, Cursor::Single(CursorIndex { line: 0, index: 0 }));
    }

    #[test]
    fn move_left_then_right_identity() {
        let mut input = PlainTextInput::new("Hello world!".to_string());
        input.cursor = Cursor::Single(CursorIndex { line: 0, index: 12 });

        let cursor = input.cursor;
        input.move_left();
        input.move_right();

        assert_eq!(input.cursor, cursor);



        let mut input = PlainTextInput::new("ধারা ১ সমস্ত মানুষ".to_string());
        input.cursor = Cursor::Single(CursorIndex { line: 0, index: 13 });

        let cursor = input.cursor;
        input.move_left();
        input.move_right();

        assert_eq!(input.cursor, cursor);



        // Empty will also return to original position
        let mut input = PlainTextInput::new("".to_string());
        input.cursor = Cursor::Single(CursorIndex { line: 0, index: 0 });

        let cursor = input.cursor;
        input.move_left();
        input.move_right();

        assert_eq!(input.cursor, cursor);



        let mut input = PlainTextInput::new(LocalState::new("Hello world!".to_string()))
            .obscure('0');
        input.cursor = Cursor::Single(CursorIndex { line: 0, index: 12 });

        let cursor = input.cursor;
        input.move_left();
        input.move_right();

        assert_eq!(input.cursor, cursor);
    }
}