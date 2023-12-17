use std::ops::Range;

use copypasta::{ClipboardContext, ClipboardProvider};
use unicode_segmentation::UnicodeSegmentation;
use carbide::draw::Rect;
use carbide::event::MouseEventContext;
use carbide::layout::LayoutContext;
use carbide::text::InnerTextContext;
use carbide_core::CommonWidgetImpl;

use carbide_core::draw::{Color, Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseEvent, MouseEventHandler, OtherEventHandler};
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::focus::Focusable;
use carbide_core::layout::{BasicLayouter, Layout, Layouter};
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map2, ReadState, ReadStateExtNew, State, TState};
use carbide_core::state::StateSync;
use carbide_core::utils::{binary_search};
use carbide_core::widget::{CommonWidget, Rectangle, Text, TextWidget, AnyWidget, WidgetExt, WidgetId, Widget};
use carbide_core::widget::Wrap;
use crate::{enabled_state, EnabledState};

use crate::plain::cursor::{Cursor, CursorIndex};

pub type TextInputState = TState<Result<String, String>>;

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
                text.graphemes(true).map(|_a| obscuring_char).collect::<String>() // TODO: Does this break when changing to editing chars instead of graphemes?
            } else {
                text.clone()
            }
        });

        let text_widget = Box::new(Text::new(display_text.clone())
            .font_size(font_size.clone())
            .color(text_color.clone())
            .wrap_mode(Wrap::None));

        //let last = len_in_graphemes(&*display_text.value());
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
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        //println!("Event: {:#?}", event);
        if self.get_focus() != Focus::Focused || !*self.enabled.value() {
            return;
        }

        let (current_movable_cursor_index, _is_selection) = match self.cursor {
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
            TextInputKeyCommand::JumpWordLeft => self.jump_word_left(current_movable_cursor_index),
            TextInputKeyCommand::JumpWordRight => self.jump_word_right(current_movable_cursor_index),
            TextInputKeyCommand::JumpSelectWordLeft => self.jump_select_word_left(current_movable_cursor_index),
            TextInputKeyCommand::JumpSelectWordRight => self.jump_select_word_right(current_movable_cursor_index),
            TextInputKeyCommand::RemoveWordLeft => self.remove_word_left(),
            TextInputKeyCommand::RemoveWordRight => self.remove_word_right(),
            TextInputKeyCommand::DuplicateLeft => self.duplicate_left(),
            TextInputKeyCommand::DuplicateRight => self.duplicate_right(),
            TextInputKeyCommand::Copy => self.copy(),
            TextInputKeyCommand::Paste => self.paste(),
            TextInputKeyCommand::Clip => self.clip(),
            TextInputKeyCommand::SelectAll => self.select_all(),
            TextInputKeyCommand::RemoveAll => self.remove_all(),
            TextInputKeyCommand::JumpToLeft => self.jump_to_left(),
            TextInputKeyCommand::JumpToRight => self.jump_to_right(),
            TextInputKeyCommand::JumpSelectToLeft => self.jump_select_to_left(),
            TextInputKeyCommand::JumpSelectToRight => self.jump_select_to_right(),
            TextInputKeyCommand::Enter => self.enter(env),
            TextInputKeyCommand::Undefined => {}
        }

        match event {
            KeyboardEvent::Text(string, modifiers) => {
                if string.len() == 0 || string.chars().next().unwrap().is_control() || modifiers.contains(ModifierKey::GUI) {
                    return;
                }

                match self.cursor {
                    Cursor::Single(index) => {
                        self.insert_str(index.index, string);

                        self.cursor = Cursor::Single(CursorIndex {
                            line: 0,
                            index: index.index + string.len(),
                        });
                    }
                    Cursor::Selection { start, end } => {
                        let min = start.index.min(end.index);
                        let max = start.index.max(end.index);
                        self.remove_range(min..max);
                        self.capture_state(env);
                        self.insert_str(min, string);
                        self.cursor = Cursor::Single(CursorIndex {
                            line: 0,
                            index: min + string.len(),
                        });
                    }
                }
            }
            _ => (),
        }

        //println!("cursor: {:?}", self.cursor);
    }
}

impl<F: State<T=Focus>, C: ReadState<T=Color>, O: ReadState<T=Option<char>>, S: ReadState<T=u32>, T: State<T=String>, E: ReadState<T=bool>> PlainTextInput<F, C, O, S, T, E> {
    fn enter(&mut self, env: &mut Environment) {
        self.set_focus_and_request(Focus::FocusReleased, env);
    }

    fn jump_select_to_right(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                self.cursor = Cursor::Selection {
                    start: index,
                    end: CursorIndex {
                        line: 0,
                        index: self.text.value().len(),
                    },
                }
            }
            Cursor::Selection { start, .. } => {
                self.cursor = Cursor::Selection {
                    start,
                    end: CursorIndex {
                        line: 0,
                        index: self.text.value().len(),
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
            index: self.text.value().len(),
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
        self.cursor = Cursor::Selection {
            start: CursorIndex { line: 0, index: 0 },
            end: CursorIndex {
                line: 0,
                index: self.text.value().len(),
            },
        }
    }

    /// Clip the selection text, or the full text if nothing was selected
    fn clip(&mut self) {
        let mut ctx = ClipboardContext::new().unwrap();

        match self.cursor {
            Cursor::Single(_) => {
                ctx.set_contents(self.display_text.value().to_string())
                    .unwrap();
                self.text.set_value("".to_string());

                self.cursor = Cursor::Single(CursorIndex { line: 0, index: 0 })
            }
            Cursor::Selection { start, end } => {
                let min = start.index.min(end.index);
                let max = start.index.max(end.index);

                let s = self.display_text.value()[min..max].to_string();
                ctx.set_contents(s).unwrap();
                self.remove_range(min..max);

                self.cursor = Cursor::Single(CursorIndex { line: 0, index: min })
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
                self.insert_str(index.index, &content);

                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: index.index + content.len(),
                });
            }
            Cursor::Selection { start, end } => {
                let min = start.index.min(end.index);
                let max = start.index.max(end.index);
                self.remove_range(min..max);

                self.insert_str(min, &content);
                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: min + content.len(),
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
                let min = start.index.min(end.index);
                let max = start.index.max(end.index);

                let s = self.display_text.value()[min..max].to_string();
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
                    index: self.text.value().len(),
                })
            }
            Cursor::Selection { start, end } => {
                let text = self.text.value().clone();
                let min = start.index.min(end.index);
                let max = start.index.max(end.index);

                self.insert_str(max, &text[min..max]);

                self.cursor = Cursor::Selection {
                    start: CursorIndex {
                        line: 0,
                        index: end.index,
                    },
                    end: CursorIndex {
                        line: 0,
                        index: end.index + (min..max).count(),
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
                let min = start.index.min(end.index);
                let max = start.index.max(end.index);

                self.insert_str(max, &text[min..max]);
            }
        }
    }

    /// If we have a cursor and not a selection, we remove rightwards until we see either a space or the end of the text
    fn remove_word_right(&mut self) {
        if let Cursor::Single(index) = self.cursor {
            let start_index = next_byte_offset(index.index, &*self.display_text.value(), true, |s| s == " ");

            self.remove_range(index.index..start_index);
        }
    }

    /// If we have a cursor and not a selection, we remove leftwards until we see either a space or the start of the text
    fn remove_word_left(&mut self) {
        if let Cursor::Single(index) = self.cursor {
            let start_index = prev_byte_offset(index.index, &*self.display_text.value(), true, |s| s == " ");

            self.remove_range(start_index..index.index);

            self.cursor = Cursor::Single(CursorIndex {
                line: 0,
                index: start_index,
            })
        }
    }

    /// Creates or extends a selection to the next space to the right or the end of the text
    fn jump_select_word_right(&mut self, current_movable_cursor_index: CursorIndex) {
        let index = next_byte_offset(current_movable_cursor_index.index, &*self.display_text.value(), true, |s| s == " ");

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
    fn jump_select_word_left(&mut self, current_movable_cursor_index: CursorIndex) {
        let index = prev_byte_offset(current_movable_cursor_index.index, &*self.display_text.value(), true, |s| s == " ");

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
    fn jump_word_right(&mut self, current_movable_cursor_index: CursorIndex) {
        self.cursor = Cursor::Single(CursorIndex {
            line: 0,
            index: next_byte_offset(current_movable_cursor_index.index, &*self.display_text.value(), true, |s| s == " ")
        });
    }

    /// Moves the cursor to the left to either we see a space or the start of the text
    fn jump_word_left(&mut self, current_movable_cursor_index: CursorIndex) {
        self.cursor = Cursor::Single(CursorIndex {
            line: 0,
            index: prev_byte_offset(current_movable_cursor_index.index, &*self.display_text.value(), true, |s| s == " ")
        });
    }

    /// Remove the grapheme to the right of the cursor if any exist
    fn remove_right(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                if index.index < self.text.value().len() {
                    let mut range = None;

                    for (i, _) in self.display_text.value().grapheme_indices(true) {
                        if i > index.index {
                            range = Some(index.index..i);
                            break;
                        }
                    }

                    if let Some(range) = range {
                        self.remove_range(range);
                    } else {
                        let len = self.text.value().len();
                        self.remove_range(index.index..len);
                    }
                }
            }
            Cursor::Selection { start, end } => {
                let min = start.index.min(end.index);
                let max = start.index.max(end.index);
                self.remove_range(min..max);

                self.cursor = Cursor::Single(CursorIndex { line: 0, index: min });
            }
        }
    }

    /// Remove the grapheme to the left of the cursor if any exist
    fn remove_left(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                if index.index > 0 {
                    let mut prev_byte_offset = 0;

                    for (i, _) in self.display_text.value().char_indices() { // .grapheme_indices(true)
                        if i < index.index {
                            prev_byte_offset = i;
                        } else {
                            break;
                        }
                    }

                    self.remove_range(prev_byte_offset..index.index);

                    self.cursor = Cursor::Single(CursorIndex {
                        line: 0,
                        index: prev_byte_offset,
                    });
                }
            }
            Cursor::Selection { start, end } => {
                let min = start.index.min(end.index);
                let max = start.index.max(end.index);

                self.remove_range(min..max);

                self.cursor = Cursor::Single(CursorIndex { line: 0, index: min });
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
                        index: next_grapheme_byte_offset(index.index, &self.text.value()),
                    },
                }
            }
            Cursor::Selection { start, end } => {
                let new_index = next_grapheme_byte_offset(end.index, &self.text.value());

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
                        index: prev_grapheme_byte_offset(index.index, &self.text.value()),
                    },
                }
            }
            Cursor::Selection { start, end } => {
                let new_index = prev_grapheme_byte_offset(end.index, &self.text.value());

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
                    index: next_grapheme_byte_offset(current_index.index, &self.text.value()),
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
                    index: prev_grapheme_byte_offset(current_index.index, &self.text.value()),
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
    fn insert_str(&mut self, byte_offset: usize, string: &str) {
        self.text.value_mut().insert_str(byte_offset, string);
    }

    /// Push a string to the end of the input
    fn push_str(&mut self, string: &str) {
        self.text.value_mut().push_str(string);
    }

    /// Remove all the graphemes inside the range,
    fn remove_range(&mut self, byte_range: Range<usize>) {
        self.text.value_mut().replace_range(byte_range, "");
    }

    /// Clamp the cursor to within the number of chars in the displayed text.
    /// The state should be up to date before calling this method, especially the display_text state.
    fn clamp_cursor(&mut self) {
        let count = self.display_text.value().len();

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

    fn update_offset_with_speed_to_make_cursor_visible(&mut self, speed: f64, ctx: &mut dyn InnerTextContext) {
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
            let cursor_offset_from_text_origin = ctx.position_of(text_id, 0, index.index).x();

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
                current_offset -= (cursor_offset_from_text_origin + self.cursor_widget.width() + current_offset - self.width());
            }

            //dbg!(cursor_offset_from_text_origin + current_offset);
            if cursor_offset_from_text_origin + current_offset < 0.0 {
                //println!("Current offset below 0");
                current_offset -= (cursor_offset_from_text_origin + current_offset);
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
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, ctx: &mut MouseEventContext) {
        let enabled = *self.enabled.value();
        let editable = enabled && self.get_focus() == Focus::Focused;

        match event {
            MouseEvent::Press(_, _, _) if !self.is_inside(event.get_current_mouse_position()) => {
                if self.get_focus() == Focus::Focused {
                    self.set_focus_and_request(Focus::FocusReleased, ctx.env);
                }
            }
            MouseEvent::Press(_, position, ModifierKey::NO_MODIFIER) if enabled => self.text_click(position, ctx),
            MouseEvent::Release(_, _, _) => {
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
        if to.x() - self.x() < 0.0 {
            self.current_offset_speed = Some(SCROLL_SUPER_FAST_SPEED);
        } else if (self.x() + self.width()) - to.x() < 0.0 {
            self.current_offset_speed = Some(-SCROLL_SUPER_FAST_SPEED);
        } else if to.x() - self.x() < SCROLL_FAST_WIDTH {
            self.current_offset_speed = Some(SCROLL_FAST_SPEED);
        } else if (self.x() + self.width()) - to.x() < SCROLL_FAST_WIDTH {
            self.current_offset_speed = Some(-SCROLL_FAST_SPEED);
        } else if to.x() - self.x() < SCROLL_SLOW_WIDTH {
            self.current_offset_speed = Some(SCROLL_SLOW_SPEED);
        } else if (self.x() + self.width()) - to.x() < SCROLL_SLOW_WIDTH {
            self.current_offset_speed = Some(-SCROLL_SLOW_SPEED);
        } else {
            self.current_offset_speed = None;
        }

        let x = to.x() - self.position.x() - *self.text_offset.value();
        let (line, index) = ctx.hit(self.text_widget.text_id(), Position::new(x, 0.0));


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
            self.set_focus_and_request(Focus::FocusRequested, ctx.env);
        }

        let x = position.x() - self.position.x() - *self.text_offset.value();
        let (line, index) = ctx.text.hit(self.text_widget.text_id(), Position::new(x, 0.0));

        self.cursor = Cursor::Single(CursorIndex {
            line,
            index,
        });
    }

    fn selection_click(&mut self, position: &Position, ctx: &mut MouseEventContext) {
        let x = position.x() - self.position.x() - *self.text_offset.value();
        let (line, clicked_index) = ctx.text.hit(self.text_widget.text_id(), Position::new(x, 0.0));

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
        let x = position.x() - self.position.x() - *self.text_offset.value();
        let (line, clicked_index) = ctx.text.hit(self.text_widget.text_id(), Position::new(x, 0.0));

        let range = word_range_surrounding_byte_offset(clicked_index, &self.display_text.value());

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
                let start_x = ctx.text.position_of(text_id, 0, start.index).x() + self.x();
                let end_x = ctx.text.position_of(text_id, 0, end.index).x() + self.x();

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

        let positioning = BasicLayouter::Leading.positioner();
        let position = self.position + Position::new(*self.text_offset.value(), 0.0);
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.text_widget);
        self.text_widget.position_children(ctx);

        //println!("Position children called");
        if let Some(speed) = self.current_offset_speed {
            self.update_offset_with_speed_to_make_cursor_visible(speed, ctx.text);
            ctx.env.request_animation_frame();
        } else {
            self.update_offset_to_make_cursor_visible(ctx.text);
        }


        let positioning = BasicLayouter::Leading.positioner();
        let position = self.position + Position::new(*self.text_offset.value(), 0.0);
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.text_widget);
        self.text_widget.position_children(ctx);

        if self.get_focus() == Focus::Focused && *self.enabled.value() {
            let text_id = self.text_widget.text_id();

            match self.cursor {
                Cursor::Single(index) => {
                    let x = ctx.text.position_of(text_id, 0, index.index).x() + self.x() + *self.text_offset.value();

                    self.cursor_widget.set_position(Position::new(x, self.text_widget.y()));
                    self.cursor_widget.position_children(ctx);
                }
                Cursor::Selection { start, end } => {
                    let end_x = ctx.text.position_of(text_id, 0, end.index).x() + self.x() + *self.text_offset.value();
                    let min_x = ctx.text.position_of(text_id, 0, start.index.min(end.index)).x() + self.x() + *self.text_offset.value();

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
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        if self.get_focus() == Focus::Focused && *self.enabled.value() {
            match self.cursor {
                Cursor::Single(_) => {
                    self.cursor_widget.render(context, env);
                }
                Cursor::Selection { .. } => {
                    self.selection_widget.render(context, env);
                    self.cursor_widget.render(context, env);
                }
            }
        }

        self.text_widget.render(context, env);
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
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 1, focus: self.focus);
}

impl<
    F: State<T=Focus>,
    C: ReadState<T=Color>,
    O: ReadState<T=Option<char>>,
    S: ReadState<T=u32>,
    T: State<T=String>,
    E: ReadState<T=bool>,
> WidgetExt for PlainTextInput<F, C, O, S, T, E> {}


// ---------------------------------------------------
//  Key commands
// ---------------------------------------------------
pub(super) enum TextInputKeyCommand {
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
    Clip,
    SelectAll,
    RemoveAll,
    JumpToLeft,
    JumpToRight,
    JumpSelectToLeft,
    JumpSelectToRight,
    Enter,
    Undefined,
}

impl From<&KeyboardEvent> for TextInputKeyCommand {
    fn from(value: &KeyboardEvent) -> Self {
        match value {
            KeyboardEvent::Press(Key::Left, ModifierKey::NO_MODIFIER) => TextInputKeyCommand::MoveLeft,
            KeyboardEvent::Press(Key::Left, ModifierKey::SHIFT) => TextInputKeyCommand::SelectLeft,
            KeyboardEvent::Press(Key::Left, ModifierKey::ALT) => TextInputKeyCommand::JumpWordLeft,
            KeyboardEvent::Press(Key::Left, ModifierKey::GUI) => TextInputKeyCommand::JumpToLeft,
            KeyboardEvent::Press(Key::Left, ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordLeft,
            KeyboardEvent::Press(Key::Left, ModifierKey::SHIFT_GUI) => TextInputKeyCommand::JumpSelectToLeft,

            KeyboardEvent::Press(Key::Right, ModifierKey::NO_MODIFIER) => TextInputKeyCommand::MoveRight,
            KeyboardEvent::Press(Key::Right, ModifierKey::SHIFT) => TextInputKeyCommand::SelectRight,
            KeyboardEvent::Press(Key::Right, ModifierKey::ALT) => TextInputKeyCommand::JumpWordRight,
            KeyboardEvent::Press(Key::Right, ModifierKey::GUI) => TextInputKeyCommand::JumpToRight,
            KeyboardEvent::Press(Key::Right, ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordRight,
            KeyboardEvent::Press(Key::Right, ModifierKey::SHIFT_GUI) => TextInputKeyCommand::JumpSelectToRight,

            KeyboardEvent::Press(Key::Backspace, ModifierKey::NO_MODIFIER) => TextInputKeyCommand::RemoveLeft,
            KeyboardEvent::Press(Key::Backspace, ModifierKey::SHIFT) => TextInputKeyCommand::RemoveLeft,
            KeyboardEvent::Press(Key::Backspace, ModifierKey::ALT) => TextInputKeyCommand::RemoveWordLeft,

            KeyboardEvent::Press(Key::Delete, ModifierKey::NO_MODIFIER) => TextInputKeyCommand::RemoveRight,
            KeyboardEvent::Press(Key::Delete, ModifierKey::SHIFT) => TextInputKeyCommand::RemoveAll,
            KeyboardEvent::Press(Key::Delete, ModifierKey::ALT) => TextInputKeyCommand::RemoveWordRight,

            KeyboardEvent::Press(Key::C, ModifierKey::GUI) => TextInputKeyCommand::Copy,
            KeyboardEvent::Press(Key::V, ModifierKey::GUI) => TextInputKeyCommand::Paste,
            KeyboardEvent::Press(Key::X, ModifierKey::GUI) => TextInputKeyCommand::Clip,
            KeyboardEvent::Press(Key::A, ModifierKey::GUI) => TextInputKeyCommand::SelectAll,
            KeyboardEvent::Press(Key::D, ModifierKey::GUI) => TextInputKeyCommand::DuplicateRight,
            KeyboardEvent::Press(Key::D, ModifierKey::SHIFT_GUI) => TextInputKeyCommand::DuplicateLeft,

            KeyboardEvent::Press(Key::Home, ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToLeft,
            KeyboardEvent::Press(Key::End, ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToRight,
            KeyboardEvent::Press(Key::Return, ModifierKey::NO_MODIFIER) => TextInputKeyCommand::Enter,
            KeyboardEvent::Press(Key::Return2, ModifierKey::NO_MODIFIER) => TextInputKeyCommand::Enter,

            _ => TextInputKeyCommand::Undefined,
        }
    }
}


// ---------------------------------------------------
//  Utilities
// ---------------------------------------------------
fn next_grapheme_byte_offset(current_offset: usize, text: &str) -> usize {
    for (i, c) in text.grapheme_indices(true) {
        if i == current_offset {
            return i + c.len();
        }
    }

    current_offset
}

fn prev_grapheme_byte_offset(current_offset: usize, text: &str) -> usize {
    let mut prev_byte_offset = 0;

    for (i, _) in text.grapheme_indices(true) {
        if i < current_offset {
            prev_byte_offset = i;
        } else {
            break;
        }
    }

    prev_byte_offset
}

fn len_in_graphemes(text: &str) -> usize {
    text.graphemes(true).count()
}

fn len_in_chars(text: &str) -> usize {
    text.char_indices().count()
}

/// Get the index of the first byte for a given grapheme index.
fn byte_index_from_graphemes(grapheme_index: usize, text: &str) -> usize {
    if text.len() == 0 {
        return 0;
    }
    let byte_offset = match text.grapheme_indices(true).skip(grapheme_index).next() {
        None => text.len(),
        Some((g, _)) => g,
    };

    byte_offset
}

fn byte_range_graphemes(grapheme_range: Range<usize>, text: &str) -> Range<usize> {
    let start = byte_index_from_graphemes(grapheme_range.start, text);
    let end = byte_index_from_graphemes(grapheme_range.end, text);

    start..end
}

fn prev_byte_offset(byte_offset: usize, text: &str, skip_initial: bool, f: fn(&str) ->bool) -> usize {
    let substring = &text[..byte_offset];

    for (index, c) in substring.grapheme_indices(true).rev().skip_while(|(_, s)| f(*s) && skip_initial) {
        if f(c) {
            return index + c.len();
        }
    }

    0
}

fn next_byte_offset(byte_offset: usize, text: &str, skip_initial: bool, f: fn(&str) ->bool) -> usize {
    let substring = &text[byte_offset..];

    for (index, c) in substring.grapheme_indices(true).skip_while(|(_, s)| f(*s) && skip_initial) {
        if f(c) {
            return index + byte_offset;
        }
    }

    text.len()
}

/// Returns a range of byte offsets for either the word at the current byte offset
/// or the range of spaces surrounding the current byte offset
fn word_range_surrounding_byte_offset(byte_offset: usize, text: &str) -> Range<usize> {
    let min = prev_byte_offset(byte_offset, text, false, |s| s == " ");
    let max = next_byte_offset(byte_offset, text, false, |s| s == " ");

    if min == max {
        let min = prev_byte_offset(byte_offset, text, false, |s| s != " ");
        let max = next_byte_offset(byte_offset, text, false, |s| s != " ");

        return min..max
    }

    min..max
}