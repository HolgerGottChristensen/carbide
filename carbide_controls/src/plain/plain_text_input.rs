use std::ops::Range;

use copypasta::{ClipboardContext, ClipboardProvider};
use unicode_segmentation::UnicodeSegmentation;
use carbide_core::CommonWidgetImpl;

use carbide_core::draw::{Color, Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseEvent, MouseEventHandler, OtherEventHandler};
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::focus::Focusable;
use carbide_core::layout::{BasicLayouter, Layout, Layouter};
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map2, NewStateSync, ReadState, ReadStateExtNew, State, TState};
use carbide_core::state::StateSync;
use carbide_core::text::Glyph;
use carbide_core::utils::{binary_search, clamp};
use carbide_core::widget::{CommonWidget, Rectangle, Text, TextWidget, Widget, WidgetExt, WidgetId};
use carbide_core::widget::Wrap;

use crate::plain::cursor::{Cursor, CursorIndex};

pub type TextInputState = TState<Result<String, String>>;

pub const PASSWORD_CHAR: char = '●';
pub const PASSWORD_CHAR_SMALL: char = '•';

pub const SCROLL_FAST_SPEED: f64 = 1.0;
pub const SCROLL_SLOW_SPEED: f64 = 0.5;
pub const SCROLL_FAST_WIDTH: f64 = 6.0;
pub const SCROLL_SLOW_WIDTH: f64 = 12.0;

/// A plain text input widget. The widget contains no specific styling, other than text color,
/// cursor color/width and selection color. Most common logic has been implemented, such as
/// key shortcuts, mouse click and drag select along with copy and paste. For an example of
/// how to use this widget look at examples/plain_text_input
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent, OtherEvent, Layout, Render)]
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
    cursor_widget: Box<dyn Widget>,
    selection_widget: Box<dyn Widget>,

    // Text styles
    #[state] text_color: C,
    #[state] obscure_text: O,
    #[state] font_size: S,

    // Text
    #[state] display_text: Box<dyn AnyReadState<T=String>>,
    #[state] text: T,
    #[state] text_offset: TState<f64>,

    // Cursor
    cursor: Cursor,
    last_drag_position: Option<Position>,
    current_offset_speed: Option<f64>,
}

impl PlainTextInput<Focus, Color, Option<char>, u32, String, bool> {
    pub fn new<S: IntoState<String>>(text: S) -> PlainTextInput<TState<Focus>, impl ReadState<T=Color>, Option<char>, impl ReadState<T=u32>, S::Output, bool> {
        let focus = LocalState::new(Focus::Unfocused);
        let color = EnvironmentColor::Label.color();
        let obscure = None;
        let font_size = EnvironmentFontSize::Body.u32();

        let cursor_widget = Rectangle::new().fill(EnvironmentColor::Green);
        let selection_widget = Rectangle::new().fill(EnvironmentColor::Purple);

        Self::new_internal(
            focus,
            color,
            obscure,
            font_size,
            text.into_state(),
            cursor_widget,
            selection_widget,
            true,
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

    pub fn selection_widget(self, selection: Box<dyn Widget>) -> PlainTextInput<F, C, O, S, T, E> {
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

    pub fn cursor_widget(self, cursor: Box<dyn Widget>) -> PlainTextInput<F, C, O, S, T, E> {
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
    >(focus: F2, text_color: C2, obscure: O2, font_size: S2, text: T2, cursor_widget: Box<dyn Widget>, selection_widget: Box<dyn Widget>, enabled: E2) -> PlainTextInput<F2, C2, O2, S2, T2, E2> {

        let display_text = Map2::read_map(text.clone(), obscure.clone(), |text, obscure| {
            if let Some(obscuring_char) = obscure {
                text.graphemes(true).map(|a| obscuring_char).collect::<String>()
            } else {
                text.clone()
            }
        });

        let text_widget = Text::new(display_text.clone())
            .font_size(font_size.clone())
            .color(text_color.clone())
            .wrap_mode(Wrap::None);

        let last = len_in_graphemes(&*display_text.value());

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
                if len_in_graphemes(string) == 0 || string.chars().next().unwrap().is_control()
                {
                    return;
                }
                if modifiers.contains(ModifierKey::GUI) {
                    return;
                }

                match self.cursor {
                    Cursor::Single(index) => {
                        self.insert_str(index.index, string);

                        self.cursor = Cursor::Single(CursorIndex {
                            line: 0,
                            index: index.index + len_in_graphemes(&string),
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
                            index: min + len_in_graphemes(&string),
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
                        index: len_in_graphemes(&self.text.value()),
                    },
                }
            }
            Cursor::Selection { start, .. } => {
                self.cursor = Cursor::Selection {
                    start,
                    end: CursorIndex {
                        line: 0,
                        index: len_in_graphemes(&self.text.value()),
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
            index: len_in_graphemes(&self.text.value()),
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
                index: len_in_graphemes(&self.text.value()),
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

                let s = self.display_text.value()[byte_range_graphemes(min..max, &*self.display_text.value())].to_string();
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
                    index: index.index + len_in_graphemes(&content),
                });
            }
            Cursor::Selection { start, end } => {
                let min = start.index.min(end.index);
                let max = start.index.max(end.index);
                self.remove_range(min..max);

                self.insert_str(min, &content);
                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: min + len_in_graphemes(&content),
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

                let s = self.display_text.value()[byte_range_graphemes(min..max, &*self.display_text.value())].to_string();
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
                    index: len_in_graphemes(&text) * 2,
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
            let start_index = next_space_grapheme_index(index.index, &*self.display_text.value());

            self.remove_range(index.index..start_index);
        }
    }

    /// If we have a cursor and not a selection, we remove leftwards until we see either a space or the start of the text
    fn remove_word_left(&mut self) {
        if let Cursor::Single(index) = self.cursor {
            let start_index = prev_space_grapheme_index(index.index, &*self.display_text.value());

            self.remove_range(start_index..index.index);

            self.cursor = Cursor::Single(CursorIndex {
                line: 0,
                index: start_index,
            })
        }
    }

    /// Creates or extends a selection to the next space to the right or the end of the text
    fn jump_select_word_right(&mut self, current_movable_cursor_index: CursorIndex) {
        let index = next_space_grapheme_index(current_movable_cursor_index.index, &*self.display_text.value());

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
        let index = prev_space_grapheme_index(current_movable_cursor_index.index, &*self.display_text.value());

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
            index: next_space_grapheme_index(current_movable_cursor_index.index, &*self.display_text.value())
        });
    }

    /// Moves the cursor to the left to either we see a space or the start of the text
    fn jump_word_left(&mut self, current_movable_cursor_index: CursorIndex) {
        self.cursor = Cursor::Single(CursorIndex {
            line: 0,
            index: prev_space_grapheme_index(current_movable_cursor_index.index, &*self.display_text.value())
        });
    }

    /// Remove the grapheme to the right of the cursor if any exist
    fn remove_right(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                if index.index < len_in_graphemes(&*self.text.value()) {
                    self.remove(index.index);

                    self.cursor = Cursor::Single(CursorIndex {
                        line: 0,
                        index: index.index,
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

    /// Remove the grapheme to the left of the cursor if any exist
    fn remove_left(&mut self) {
        match self.cursor {
            Cursor::Single(index) => {
                if index.index > 0 {
                    self.remove(index.index - 1);

                    self.cursor = Cursor::Single(CursorIndex {
                        line: 0,
                        index: index.index - 1,
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
                let new_index = carbide_core::utils::clamp(
                    index.index + 1,
                    0,
                    len_in_graphemes(&self.text.value()),
                );

                self.cursor = Cursor::Selection {
                    start: index,
                    end: CursorIndex {
                        line: 0,
                        index: new_index,
                    },
                }
            }
            Cursor::Selection { start, end } => {
                let new_index = carbide_core::utils::clamp(
                    end.index + 1,
                    0,
                    len_in_graphemes(&self.text.value()),
                );

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
                let moved_index = if index.index == 0 { 0 } else { index.index - 1 };
                let new_index = carbide_core::utils::clamp(
                    moved_index,
                    0,
                    len_in_graphemes(&self.text.value()),
                );

                self.cursor = Cursor::Selection {
                    start: index,
                    end: CursorIndex {
                        line: 0,
                        index: new_index,
                    },
                }
            }
            Cursor::Selection { start, end } => {
                let moved_index = if end.index == 0 { 0 } else { end.index - 1 };
                let new_index = carbide_core::utils::clamp(
                    moved_index,
                    0,
                    len_in_graphemes(&self.text.value()),
                );

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
                let moved_index = (current_index.index + 1).min(len_in_graphemes(&*self.text.value()));

                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: moved_index,
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
                let current_index = current_index.index;

                let moved_index = if current_index == 0 {
                    0
                } else {
                    (current_index - 1).max(0)
                };

                self.cursor = Cursor::Single(CursorIndex {
                    line: 0,
                    index: moved_index,
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


    /// Insert a string at a given grapheme index.
    fn insert_str(&mut self, grapheme_index: usize, string: &str) {
        let offset = byte_index_from_graphemes(grapheme_index, &self.text.value());
        //TODO: This might be rather inefficient
        let mut next_string = self.text.value().clone();
        next_string.insert_str(offset, string);
        self.text.set_value(next_string);
    }

    /// Push a string to the end of the input
    fn push_str(&mut self, string: &str) {
        let mut next_string = self.text.value().clone();
        next_string.push_str(string);
        self.text.set_value(next_string);
    }

    /// Remove a single grapheme at an index.
    fn remove(&mut self, grapheme_index: usize) {
        self.remove_range(grapheme_index..grapheme_index+1);
    }

    /// Remove all the graphemes inside the range,
    fn remove_range(&mut self, grapheme_range: Range<usize>) {
        let mut new_string: String = self.text.value().clone();
        let byte_range = byte_range_graphemes(grapheme_range, &new_string);

        new_string.replace_range(byte_range, "");
        self.text.set_value(new_string);
    }

    /// Recalculate the position of the cursor and the selection. This will not move the cursor
    /// index, but move the visual positioning of the cursor and the selection box (if selection mode).
    fn reposition_cursor(&mut self, env: &mut Environment) {
        /*let glyph = self.glyphs(env);
        let text = &*self.display_text.value();

        let index = match &mut self.cursor {
            Cursor::Single(index) => {
                let len_in_graphemes = len_in_graphemes(text);
                *index = CursorIndex {
                    line: 0,
                    char: index.char.min(len_in_graphemes),
                };
                index
            }
            Cursor::Selection { end, .. } => end,
        };

        let point = index.position(text, &glyph);

        *self.cursor_x.value_mut() = point.x();
        *self.selection_x.value_mut() = point.x();

        let selection_width = self.cursor.width(text, &glyph);

        if selection_width < 0.0 {
            *self.selection_width.value_mut() = selection_width.abs();
        } else {
            *self.selection_x.value_mut() -= selection_width;
            *self.selection_width.value_mut() = selection_width;
        }*/

        todo!("reposition_cursor")
    }

    /// This will change the text offset to make the cursor visible. It will result in the text
    /// getting scrolled, such that the entire cursor is visible.
    fn recalculate_offset_to_make_cursor_visible(&mut self, env: &mut Environment) {
        /*let cursor_x = *self.cursor_x.value();
        let cursor_width = 4.0;
        let current_text_offset = *self.text_offset.value();

        if cursor_x + cursor_width > self.width()
            && -current_text_offset < cursor_x + cursor_width - self.width()
        {
            let new_text_offset = -(cursor_x + cursor_width - self.width());

            *self.text_offset.value_mut() = new_text_offset;
        } else if cursor_x + current_text_offset < 0.0 {
            let new_text_offset = -(cursor_x);

            *self.text_offset.value_mut() = new_text_offset;
        }

        let positioned_glyphs = self.glyphs(env);

        if positioned_glyphs.len() != 0 {
            let last_glyph = &positioned_glyphs[positioned_glyphs.len() - 1];

            let point = last_glyph.position();

            let width = last_glyph.advance_width();

            let width_of_text = point.x() + width;

            if width_of_text < self.width() {
                *self.text_offset.value_mut() = 0.0;
            } else if current_text_offset.abs() > width_of_text {
                *self.text_offset.value_mut() = 0.0;
                self.recalculate_offset_to_make_cursor_visible(env)
            }
        } else {
            *self.text_offset.value_mut() = 0.0;
        }*/

        todo!("recalculate_offset_to_make_cursor_visible")
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
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, env: &mut Environment) {
        // If clicked outside, we should release focus
        /*if !self.is_inside(event.get_current_mouse_position()) {
            match event {

                MouseEvent::Release(_, _, _) => {
                    self.current_offset_speed = None;
                    self.last_drag_position = None;
                }
                MouseEvent::Drag { to, delta_xy, .. } => {
                    if self.get_focus() == Focus::Focused {
                        self.last_drag_position = Some(*to);
                        self.drag_selection(env, to, delta_xy);
                    }
                }
                _ => (),
            }

            return;
        }*/

        let enabled = *self.enabled.value();
        let editable = enabled && self.get_focus() == Focus::Focused;

        match event {
            MouseEvent::Press(_, _, _) if !self.is_inside(event.get_current_mouse_position()) => {
                if self.get_focus() == Focus::Focused {
                    self.set_focus_and_request(Focus::FocusReleased, env);
                }
            }
            MouseEvent::Press(_, position, ModifierKey::NO_MODIFIER) if enabled => self.text_click(position, env),
            MouseEvent::Release(_, _, _) => {
                self.current_offset_speed = None;
                self.last_drag_position = None;
            }
            //MouseEvent::Click(_, position, ModifierKey::NO_MODIFIER) => self.text_click(position, env),
            MouseEvent::Click(_, position, ModifierKey::SHIFT) if editable => self.selection_click(position, env),
            MouseEvent::NClick(_, _, _, n) if n % 2 == 1 && editable => self.select_all(),
            MouseEvent::NClick(_, position, _, n) if n % 2 == 0 && editable => self.select_word_at_click(position, env),
            MouseEvent::Drag { to, delta_xy, .. } => {
                if !enabled {
                    return;
                }

                if self.last_drag_position.is_some() || self.is_inside(event.get_current_mouse_position()) {
                    self.last_drag_position = Some(*to);
                    self.drag_selection(env, to, delta_xy);
                }
            }
            _ => (),
        }
    }
}

impl<F: State<T=Focus>, C: ReadState<T=Color>, O: ReadState<T=Option<char>>, S: ReadState<T=u32>, T: State<T=String>, E: ReadState<T=bool>> PlainTextInput<F, C, O, S, T, E> {
    fn drag_selection(&mut self, env: &mut Environment, to: &Position, delta_xy: &Position) {
        if self.width() < SCROLL_FAST_WIDTH * 2.0 {
            self.current_offset_speed = None;
        } else if to.x() - self.x() < SCROLL_FAST_WIDTH {
            self.current_offset_speed = Some(SCROLL_FAST_SPEED);
        } else if (self.x() + self.width()) - to.x() < SCROLL_FAST_WIDTH {
            self.current_offset_speed = Some(-SCROLL_FAST_SPEED);
        } else if self.width() < SCROLL_SLOW_WIDTH * 2.0 {
            self.current_offset_speed = None;
        } else if to.x() - self.x() < SCROLL_SLOW_WIDTH {
            self.current_offset_speed = Some(SCROLL_SLOW_SPEED);
        } else if (self.x() + self.width()) - to.x() < SCROLL_SLOW_WIDTH {
            self.current_offset_speed = Some(-SCROLL_SLOW_SPEED);
        } else {
            self.current_offset_speed = None;
        }

        let mut relative_offset = to.x() * env.scale_factor();
        if let Some(speed) = self.current_offset_speed {
            relative_offset += speed;
        }

        let current_index = find_index_from_offset_and_glyphs(relative_offset, &self.text_widget.glyphs());


        match self.cursor {
            Cursor::Single(start) | Cursor::Selection { start, .. } => {
                if start.index == current_index {
                    self.cursor = Cursor::Single(start);
                } else {
                    self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, index: current_index } };
                }
            }
        }

        /*
        // Get the current text for the text_input
        let text = self.text.value().clone();

        // Get the delta x for the drag
        let delta_x = delta_xy.x().abs();

        // The threshold for when to scroll when the mouse is at the edge
        let mouse_scroll_threshold = 30.0;

        // If the cursor is at the right edge within the threshold
        if to.x() < self.x() + mouse_scroll_threshold {
            let offset = *self.text_offset.value() + 10.0 * delta_x;
            *self.text_offset.value_mut() = offset.min(0.0);

            // If the cursor is at the left edge within the threshold
        } else if to.x() > self.x() + self.width() - mouse_scroll_threshold {
            let offset = *self.text_offset.value() - 10.0 * delta_x;
            let positioned_glyphs = self.glyphs(env);

            let start = CursorIndex { line: 0, char: 0 };
            let end = CursorIndex {
                line: 0,
                char: len_in_graphemes(&self.text.value()),
            };

            let max_offset =
                Cursor::Selection { start, end }.width(&text, &positioned_glyphs);

            // Since the offset is negative we have to chose the max value
            *self.text_offset.value_mut() =
                offset.max(-(max_offset - self.width())).min(0.0);
        }

        let current_relative_offset = to.x() - self.position.x() - text_offset;

        let current_char_index =
            Cursor::char_index(current_relative_offset, &self.glyphs(env));

        match self.drag_start_cursor {
            None => match self.cursor {
                Cursor::Single(index) => {
                    self.cursor = Cursor::Selection {
                        start: index,
                        end: CursorIndex {
                            line: 0,
                            char: current_char_index,
                        },
                    }
                }
                Cursor::Selection { start, .. } => {
                    self.cursor = Cursor::Selection {
                        start,
                        end: CursorIndex {
                            line: 0,
                            char: current_char_index,
                        },
                    }
                }
            },
            Some(cursor) => {
                match cursor {
                    Cursor::Single(index) => {
                        self.cursor = Cursor::Selection {
                            start: index,
                            end: CursorIndex {
                                line: 0,
                                char: current_char_index,
                            },
                        }
                    }
                    Cursor::Selection { start, .. } => {
                        self.cursor = Cursor::Selection {
                            start,
                            end: CursorIndex {
                                line: 0,
                                char: current_char_index,
                            },
                        }
                    }
                }
                self.drag_start_cursor = None;
            }
        }*/
    }

    fn text_click(&mut self, position: &Position, env: &mut Environment) {
        if self.get_focus() == Focus::Unfocused {
            self.set_focus_and_request(Focus::FocusRequested, env);
        }

        let relative_offset = position.x() * env.scale_factor();
        let char_index = find_index_from_offset_and_glyphs(relative_offset, &self.text_widget.glyphs());

        self.cursor = Cursor::Single(CursorIndex {
            line: 0,
            index: char_index,
        });
    }

    fn selection_click(&mut self, position: &Position, env: &mut Environment) {
        let relative_offset = position.x() * env.scale_factor();
        let clicked_index = find_index_from_offset_and_glyphs(relative_offset, &self.text_widget.glyphs());

        match self.cursor {
            Cursor::Single(CursorIndex { line, index }) => {
                self.cursor = Cursor::Selection {
                    start: CursorIndex { line: 0, index },
                    end: CursorIndex {
                        line: 0,
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
                        line: 0,
                        index: clicked_index,
                    },
                }
            }
        }
    }

    fn select_word_at_click(&mut self, position: &Position, env: &mut Environment) {
        let relative_offset = position.x() * env.scale_factor();
        let clicked_index = find_index_from_offset_and_glyphs(relative_offset, &self.text_widget.glyphs());

        let range = word_range_surrounding_grapheme_index(clicked_index, &self.display_text.value());

        self.cursor = Cursor::Selection {
            start: CursorIndex {
                line: 0,
                index: range.start,
            },
            end: CursorIndex {
                line: 0,
                index: range.end,
            },
        }
    }

    /// Update the current scroll offset to make the cursor visible within the text field if possible
    fn update_offset_to_make_cursor_visible(&mut self, env: &mut Environment) {
        let mut current_offset = *self.text_offset.value();

        if self.get_focus() == Focus::Focused {
            // We should try to keep this index within view as long as the field is focused
            let index = match self.cursor {
                Cursor::Single(index) => index,
                Cursor::Selection { end, .. } => end,
            };

            let glyphs = self.text_widget.glyphs();

            // Since text is positioned from the nearest pixel, we need to take into account the
            // tolerance, to calculate the cursor offset from the text origin, since all glyphs
            // will be offset by this.
            let tolerance_difference = self.text_widget.position().tolerance(1.0/env.scale_factor()) - self.text_widget.position();

            let cursor_offset_from_text_origin = if glyphs.len() == 0 {
                0.0
            } else if index.index == 0 {
                glyphs[index.index].position().x() / env.scale_factor() - self.x() - tolerance_difference.x()
            } else {
                (glyphs[index.index - 1].position().x() + glyphs[index.index - 1].advance_width()) / env.scale_factor() - self.x() - tolerance_difference.x()
            };

            //println!("cursor_offset_from_text_origin: {:?}", cursor_offset_from_text_origin);
            //println!("tolerance: {:?}", tolerance_difference.x());
            //println!("width: {:?}", self.width());
            //println!("text_width: {:?}", self.text_widget.width());
            //println!("x: {:?}", self.x());
            //println!("text_x: {:?}", self.text_widget.x());

            //println!("current_offset: {:?}", current_offset);

            if cursor_offset_from_text_origin + self.cursor_widget.width() > self.width() {
                current_offset -= (cursor_offset_from_text_origin + self.cursor_widget.width() - self.width());
            }

            if cursor_offset_from_text_origin < 0.0 {
                current_offset -= (cursor_offset_from_text_origin);
            }
        }

        // Clamp the offset to be within the bounds of the visible area.
        current_offset = current_offset
            .max(self.width() - self.text_widget.width() - self.cursor_widget.width())
            .min(0.0);

        self.text_offset.set_value(current_offset);
        //println!("new_offset: {:?}\n", *self.text_offset.value());
    }

    /// Clamp the cursor to within the number of graphemes in the displayed text.
    /// The state should be up to date before calling this method, especially the display_text state.
    fn clamp_cursor(&mut self) {
        let len_in_graphemes = len_in_graphemes(&self.display_text.value());

        match self.cursor {
            Cursor::Single(CursorIndex{ line, index }) => {
                self.cursor = Cursor::Single(CursorIndex { line, index: index.min(len_in_graphemes)});
            }
            Cursor::Selection {
                start: CursorIndex { line: line_start, index: index_start },
                end: CursorIndex { line: line_end, index: index_end },
            } => {
                self.cursor = Cursor::Selection {
                    start: CursorIndex { line: line_start, index: index_start.min(len_in_graphemes) },
                    end: CursorIndex { line: line_end, index: index_end.min(len_in_graphemes) },
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
> OtherEventHandler for PlainTextInput<F, C, O, S, T, E> {
    /*fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        match event {
            WidgetEvent::Window(w) => {
                match w {
                    WindowEvent::Resize(_) => {
                        let offset = *self.text_offset.value();
                        let text = self.text.value().clone();
                        let positioned_glyphs = self.glyphs(env);

                        let start = CursorIndex { line: 0, char: 0 };
                        let end = CursorIndex {
                            line: 0,
                            char: len_in_graphemes(&text),
                        };

                        let max_offset =
                            Cursor::Selection { start, end }.width(&text, &positioned_glyphs);

                        // Since the offset is negative we have to chose the max value
                        *self.text_offset.value_mut() =
                            offset.max(-(max_offset - self.width())).min(0.0);
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }*/
}

impl<
    F: State<T=Focus>,
    C: ReadState<T=Color>,
    O: ReadState<T=Option<char>>,
    S: ReadState<T=u32>,
    T: State<T=String>,
    E: ReadState<T=bool>,
> Layout for PlainTextInput<F, C, O, S, T, E> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.clamp_cursor();

        //println!("calculate size");
        if let Some(position) = self.last_drag_position {
            self.drag_selection(env, &position, &position);
        }

        let text_dimensions = self.text_widget.calculate_size(requested_size, env);

        self.cursor_widget.calculate_size(Dimension::new(1.0, text_dimensions.height), env);

        // Calculate size for selection indicator
        let glyphs = self.text_widget.glyphs();
        match self.cursor {
            Cursor::Single(_) => {
                self.selection_widget.calculate_size(Dimension::new(0.0, 0.0), env);
            }
            Cursor::Selection { start, end } => {
                let min = start.index.min(end.index);
                let max = start.index.max(end.index);

                let min_x = if min == 0 {
                    self.text_widget.x()
                } else {
                    (glyphs[min - 1].position().x() + glyphs[min - 1].advance_width()) / env.scale_factor()
                };

                let max_x = if max == 0 {
                    self.text_widget.x()
                } else {
                    (glyphs[max - 1].position().x() + glyphs[max - 1].advance_width()) / env.scale_factor()
                };

                let selection_width = max_x - min_x;
                self.selection_widget.calculate_size(Dimension::new(selection_width, text_dimensions.height), env);

            }
        }

        self.set_dimension(requested_size);
        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
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
        self.text_widget.position_children(env);

        //println!("Position children called");
        self.update_offset_to_make_cursor_visible(env);

        let positioning = BasicLayouter::Leading.positioner();
        let position = self.position + Position::new(*self.text_offset.value(), 0.0);
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.text_widget);
        self.text_widget.position_children(env);

        if self.get_focus() == Focus::Focused && *self.enabled.value() {
            let glyphs = self.text_widget.glyphs();

            match self.cursor {
                Cursor::Single(index) => {

                    let new_x = if index.index == 0 {
                        self.text_widget.x()
                    } else {
                        (glyphs[index.index - 1].position().x() + glyphs[index.index - 1].advance_width()) / env.scale_factor()
                    };

                    self.cursor_widget.set_position(Position::new(new_x, self.text_widget.y()));
                    self.cursor_widget.position_children(env);
                }
                Cursor::Selection { start, end } => {
                    let min = start.index.min(end.index);

                    let min_x = if min == 0 {
                        self.text_widget.x()
                    } else {
                        (glyphs[min - 1].position().x() + glyphs[min - 1].advance_width()) / env.scale_factor()
                    };

                    let new_x = if end.index == 0 {
                        self.text_widget.x()
                    } else {
                        (glyphs[end.index - 1].position().x() + glyphs[end.index - 1].advance_width()) / env.scale_factor()
                    };

                    self.cursor_widget.set_position(Position::new(new_x, self.text_widget.y()));
                    self.cursor_widget.position_children(env);

                    self.selection_widget.set_position(Position::new(min_x, self.text_widget.y()));
                    self.selection_widget.position_children(env);
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
    CommonWidgetImpl!(self, id: self.id, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 1, focus: self.focus);
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
fn len_in_graphemes(text: &str) -> usize {
    text.graphemes(true).count()
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

fn prev_space_grapheme_index(current_grapheme_index: usize, text: &str) -> usize {
    let index = text.grapheme_indices(true)
        .rev()
        .skip(len_in_graphemes(text) - current_grapheme_index)
        .take_while(|(index, val)| *val != " ")
        .last()
        .map_or(0, |a| a.0);

    index
}

fn next_space_grapheme_index(current_grapheme_index: usize, text: &str) -> usize {
    let index = text.grapheme_indices(true)
        .skip(current_grapheme_index+1)
        .skip_while(|(index, val)| *val != " ")
        .next()
        .map_or(len_in_graphemes(text), |a| a.0);

    index
}

fn word_range_surrounding_grapheme_index(current_grapheme_index: usize, text: &str) -> Range<usize> {
    let min = prev_space_grapheme_index(current_grapheme_index, text);
    let max = next_space_grapheme_index(current_grapheme_index, text);

    min..max
}

pub fn find_index_from_offset_and_glyphs(relative_offset: f64, glyphs: &Vec<Glyph>) -> usize {
    let splits = vec![glyphs.first().map_or(0.0, |a| a.position().x() as f32)].into_iter().chain(glyphs.iter().map(|glyph| {
        let middle = glyph.position().x() + glyph.advance_width();
        middle as f32
    }));
    let splits = splits.collect::<Vec<_>>();
    let rightmost_closest = binary_search(relative_offset as f32, &splits);

    let new_closest = if rightmost_closest < splits.len() - 1
        && ((relative_offset as f32) - splits[rightmost_closest + 1]).abs()
        < ((relative_offset as f32) - splits[rightmost_closest]).abs()
    {
        rightmost_closest + 1
    } else {
        rightmost_closest
    };

    new_closest
}