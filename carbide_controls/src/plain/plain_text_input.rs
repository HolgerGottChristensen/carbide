use std::ops::{DerefMut, Range};

use copypasta::{ClipboardContext, ClipboardProvider};
use unicode_segmentation::UnicodeSegmentation;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentFontSize};
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseButton, MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent, WindowEvent};
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::focus::Focusable;
use carbide_core::layout::{BasicLayouter, Layouter};
use carbide_core::prelude::{EnvironmentColor, Layout, Primitive};
use carbide_core::prelude::StateSync;
use carbide_core::render::Render;
use carbide_core::Scalar;
use carbide_core::state::{AnimatedState, ColorState, F64State, FocusState, LocalState, ReadState, ResStringState, State, StateExt, StringState, TState, U32State};
use carbide_core::text::{FontSize, Glyph};
use carbide_core::widget::{CommonWidget, HStack, WidgetId, IfElse, Rectangle, SCALE, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut, ZStack};
use carbide_core::widget::Wrap;

use crate::plain::cursor::{Cursor, CursorIndex};
use crate::plain::text_input_key_commands::TextInputKeyCommand;

pub type TextInputState = ResStringState;

pub const PASSWORD_CHAR: char = '●';
pub const PASSWORD_CHAR_SMALL: char = '•';

/// A plain text input widget. The widget contains no specific styling, other than text color,
/// cursor color/width and selection color. Most common logic has been implemented, such as
/// key shortcuts, mouse click and drag select along with copy and paste. For an example of
/// how to use this widget look at examples/plain_text_input
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent, OtherEvent)]
pub struct PlainTextInput {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    cursor: Cursor,
    drag_start_cursor: Option<Cursor>,
    obscure_text: Option<char>,

    // Colors
    #[state] selection_color: ColorState,
    #[state] cursor_color: ColorState,
    #[state] text_color: ColorState,

    #[state] focus: FocusState,
    #[state] input_state: TextInputState,
    #[state] text: StringState,
    #[state] display_text: StringState,
    #[state] cursor_x: F64State,
    #[state] selection_x: F64State,
    #[state] selection_width: F64State,
    #[state] text_offset: F64State,
    #[state] font_size: U32State,
}

impl PlainTextInput {
    pub fn new<T: Into<TextInputState>>(text: T) -> Box<Self> {
        let text = text.into();
        let focus_state: FocusState = LocalState::new(Focus::Unfocused).into();
        let font_size: U32State = EnvironmentFontSize::Body.into();

        let selection_color: ColorState = EnvironmentColor::Green.into();
        let cursor_color: ColorState = EnvironmentColor::Red.into();
        let text_color: ColorState = EnvironmentColor::Label.into();

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

    pub fn focus<F: Into<FocusState>>(mut self, focus: F) -> Box<Self> {
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

    pub fn font_size<I: Into<U32State>>(mut self, font_size: I) -> Box<Self> {
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

    pub fn selection_color<C: Into<ColorState>>(mut self, color: C) -> Box<Self> {
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

    pub fn cursor_color<C: Into<ColorState>>(mut self, color: C) -> Box<Self> {
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

    pub fn text_color<C: Into<ColorState>>(mut self, color: C) -> Box<Self> {
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
        font_size: U32State,
        focus: FocusState,
        selection_color: ColorState,
        cursor_color: ColorState,
        text_color: ColorState,
        obscure_text: Option<char>,
    ) -> Box<Self> {
        let cursor_x: F64State = LocalState::new(0.0).into();
        let selection_x: F64State = LocalState::new(0.0).into();
        let selection_width: F64State = LocalState::new(0.0).into();
        let text_offset: F64State = LocalState::new(0.0).into();

        let is_focused = focus.mapped(|focus: &Focus| {
            focus == &Focus::Focused
        });

        let display_text: StringState = if let Some(obscuring_char) = obscure_text {
            input.mapped(move |val: &String| {
                val.chars().map(|c| obscuring_char).collect::<String>()
            })
        } else {
            input.clone().into()
        };

        let child =
            HStack::new(vec![
                ZStack::new(vec![
                    IfElse::new(is_focused.clone())
                        .when_true(
                            Rectangle::new()
                                .fill(selection_color.clone())
                                .frame(selection_width.clone(), font_size.map(|val: &u32| *val as f64).ignore_writes())
                                .offset(selection_x.clone(), 0.0)
                        ),
                    Text::new(display_text.clone())
                        .font_size(font_size.clone())
                        .wrap_mode(Wrap::None)
                        .foreground_color(text_color.clone()),
                    IfElse::new(is_focused)
                        .when_true(
                            Rectangle::new()
                                .fill(cursor_color.clone())
                                .frame(1.0, font_size.map(|val: &u32| *val as f64).ignore_writes())
                                .offset(cursor_x.clone(), 0.0),
                        ),
                ]).with_alignment(BasicLayouter::Leading)
                    .offset(text_offset.clone(), 0.0),
                Spacer::new(),
            ]).frame_fixed_height(30);

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
}

impl KeyboardEventHandler for PlainTextInput {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        if self.get_focus() == Focus::Unfocused {
            return;
        }

        match event {
            KeyboardEvent::Press(key, modifier) => {
                let (current_movable_cursor_index, _is_selection) = match self.cursor {
                    Cursor::Single(cursor_index) => {
                        (cursor_index, false)
                    }
                    Cursor::Selection { end, .. } => {
                        (end, true)
                    }
                };

                match (key, modifier).into() {
                    TextInputKeyCommand::MoveLeft => {
                        let current_char = current_movable_cursor_index.char;
                        let moved_char = if current_char == 0 { 0 } else { current_char - 1 };
                        let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(&*self.text.value()));

                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: clamped });
                    }
                    TextInputKeyCommand::MoveRight => {
                        let current_char = current_movable_cursor_index.char;
                        let moved_char = current_char + 1;
                        let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(&*self.text.value()));

                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: clamped });
                    }
                    TextInputKeyCommand::RemoveLeft => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                if index.char > 0 {
                                    self.remove(index.char - 1);
                                    self.cursor = Cursor::Single(CursorIndex { line: 0, char: index.char - 1 });
                                }
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);

                                self.remove_range(min..max);

                                self.cursor = Cursor::Single(CursorIndex { line: 0, char: min });
                            }
                        }
                    }
                    TextInputKeyCommand::RemoveRight => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                if index.char < Self::len_in_graphemes(&*self.text.value()) {
                                    self.remove(index.char);
                                    self.cursor = Cursor::Single(CursorIndex { line: 0, char: index.char });
                                }
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);
                                self.remove_range(min..max);

                                self.cursor = Cursor::Single(CursorIndex { line: 0, char: min });
                            }
                        }
                    }
                    TextInputKeyCommand::Undefined => {}
                    TextInputKeyCommand::Copy => {
                        let mut ctx = ClipboardContext::new().unwrap();

                        match self.cursor {
                            Cursor::Single(_) => {
                                ctx.set_contents(self.display_text.value().clone()).unwrap();
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);

                                let min_byte = Self::byte_index_from_graphemes(min, &*self.display_text.value());
                                let max_byte = Self::byte_index_from_graphemes(max, &*self.display_text.value());

                                let s = self.display_text.value()[min_byte..max_byte].to_string();
                                ctx.set_contents(s).unwrap();
                            }
                        }
                    }
                    TextInputKeyCommand::Paste => {
                        let mut ctx = ClipboardContext::new().unwrap();

                        let mut content = ctx.get_contents().unwrap();

                        // Remove newlines from the pasted text
                        content.retain(|c| { c != '\n' });

                        match self.cursor {
                            Cursor::Single(index) => {
                                self.insert_str(index.char, &content);
                                self.cursor = Cursor::Single(CursorIndex { line: 0, char: index.char + Self::len_in_graphemes(&content) });
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);
                                self.remove_range(min..max);

                                self.insert_str(min, &content);
                                self.cursor = Cursor::Single(CursorIndex { line: 0, char: min + Self::len_in_graphemes(&content) });
                            }
                        }
                    }
                    TextInputKeyCommand::Clip => {
                        let mut ctx = ClipboardContext::new().unwrap();

                        match self.cursor {
                            Cursor::Single(_) => {
                                ctx.set_contents(self.display_text.value().to_string()).unwrap();
                                self.text.set_value("".to_string());

                                self.cursor = Cursor::Single(CursorIndex { line: 0, char: 0 })
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);

                                let min_byte = Self::byte_index_from_graphemes(min, &*self.display_text.value());
                                let max_byte = Self::byte_index_from_graphemes(max, &*self.display_text.value());

                                let s = self.display_text.value()[min_byte..max_byte].to_string();
                                ctx.set_contents(s).unwrap();
                                self.remove_range(min..max);

                                self.cursor = Cursor::Single(CursorIndex { line: 0, char: min })
                            }
                        }
                    }
                    TextInputKeyCommand::SelectLeft => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                let moved_char = if index.char == 0 { 0 } else { index.char - 1 };
                                let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(&self.text.value()));

                                self.cursor = Cursor::Selection { start: index, end: CursorIndex { line: 0, char: clamped } }
                            }
                            Cursor::Selection { start, end } => {
                                let moved_char = if end.char == 0 { 0 } else { end.char - 1 };
                                let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(&self.text.value()));

                                if start.char == clamped {
                                    self.cursor = Cursor::Single(start)
                                } else {
                                    self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: clamped } }
                                }
                            }
                        }
                    }
                    TextInputKeyCommand::SelectRight => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                let moved_char = index.char + 1;
                                let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(&self.text.value()));

                                self.cursor = Cursor::Selection { start: index, end: CursorIndex { line: 0, char: clamped } }
                            }
                            Cursor::Selection { start, end } => {
                                let moved_char = end.char + 1;
                                let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(&self.text.value()));

                                if start.char == clamped {
                                    self.cursor = Cursor::Single(start)
                                } else {
                                    self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: clamped } }
                                }
                            }
                        }
                    }
                    TextInputKeyCommand::SelectAll => {
                        self.cursor = Cursor::Selection { start: CursorIndex { line: 0, char: 0 }, end: CursorIndex { line: 0, char: Self::len_in_graphemes(&self.text.value()) } }
                    }
                    TextInputKeyCommand::JumpWordLeft => {
                        let start_index = current_movable_cursor_index.char;

                        let range = Self::prev_word_range(&*self.display_text.value(), start_index);

                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: range.start })
                    }
                    TextInputKeyCommand::JumpWordRight => {
                        let start_index = current_movable_cursor_index.char;

                        let range = Self::next_word_range(&*self.display_text.value(), start_index);

                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: range.end })
                    }
                    TextInputKeyCommand::JumpSelectWordLeft => {
                        let start_index = current_movable_cursor_index.char;

                        let range = Self::prev_word_range(&*self.display_text.value(), start_index);

                        match self.cursor {
                            Cursor::Single(_) => {
                                self.cursor = Cursor::Selection { start: CursorIndex { line: 0, char: start_index }, end: CursorIndex { line: 0, char: range.start } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: range.start } }
                            }
                        }
                    }
                    TextInputKeyCommand::JumpSelectWordRight => {
                        let start_index = current_movable_cursor_index.char;

                        let range = Self::next_word_range(&*self.display_text.value(), start_index);

                        match self.cursor {
                            Cursor::Single(_) => {
                                self.cursor = Cursor::Selection { start: CursorIndex { line: 0, char: start_index }, end: CursorIndex { line: 0, char: range.end } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: range.end } }
                            }
                        }
                    }
                    TextInputKeyCommand::RemoveAll => {
                        self.text.set_value("".to_string());
                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: 0 })
                    }
                    TextInputKeyCommand::RemoveWordLeft => {
                        if let Cursor::Single(index) = self.cursor {
                            let start_index = index.char;

                            let range = Self::prev_word_range(&*self.display_text.value(), start_index);
                            let start = range.start;

                            self.remove_range(range);

                            self.cursor = Cursor::Single(CursorIndex { line: 0, char: start })
                        }
                    }
                    TextInputKeyCommand::RemoveWordRight => {
                        if let Cursor::Single(index) = self.cursor {
                            let start_index = index.char;

                            let range = Self::next_word_range(&*self.display_text.value(), start_index);
                            let start = range.start;

                            self.remove_range(range);

                            self.cursor = Cursor::Single(CursorIndex { line: 0, char: start })
                        }
                    }
                    TextInputKeyCommand::DuplicateLeft => {
                        match self.cursor {
                            Cursor::Single(_) => {
                                let text = self.text.value().clone();
                                self.push_str(&text);
                            }
                            Cursor::Selection { start, end } => {
                                let text = self.text.value().clone();
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);

                                self.insert_str(max, &text[min..max]);
                            }
                        }
                    }
                    TextInputKeyCommand::DuplicateRight => {
                        match self.cursor {
                            Cursor::Single(_) => {
                                let text = self.text.value().clone();
                                self.push_str(&text);

                                self.cursor = Cursor::Single(CursorIndex { line: 0, char: Self::len_in_graphemes(&text) * 2 })
                            }
                            Cursor::Selection { start, end } => {
                                let text = self.text.value().clone();
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);

                                self.insert_str(max, &text[min..max]);

                                self.cursor = Cursor::Selection { start: CursorIndex { line: 0, char: end.char }, end: CursorIndex { line: 0, char: end.char + (min..max).count() } }
                            }
                        }
                    }
                    TextInputKeyCommand::JumpToLeft => {
                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: 0 })
                    }
                    TextInputKeyCommand::JumpToRight => {
                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: Self::len_in_graphemes(&self.text.value()) })
                    }
                    TextInputKeyCommand::JumpSelectToLeft => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                self.cursor = Cursor::Selection { start: index, end: CursorIndex { line: 0, char: 0 } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: 0 } }
                            }
                        }
                    }
                    TextInputKeyCommand::JumpSelectToRight => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                self.cursor = Cursor::Selection { start: index, end: CursorIndex { line: 0, char: Self::len_in_graphemes(&self.text.value()) } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: Self::len_in_graphemes(&self.text.value()) } }
                            }
                        }
                    }
                    TextInputKeyCommand::Enter => {
                        self.set_focus_and_request(Focus::FocusReleased, env);
                    }
                }
            }
            KeyboardEvent::Text(string, modifiers) => {
                if Self::len_in_graphemes(&string) == 0 || string.chars().next().unwrap().is_control() { return; }
                if modifiers.contains(ModifierKey::GUI) { return; }

                match self.cursor {
                    Cursor::Single(index) => {
                        self.insert_str(index.char, string);

                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: index.char + Self::len_in_graphemes(&string) });
                    }
                    Cursor::Selection { start, end } => {
                        let min = start.char.min(end.char);
                        let max = start.char.max(end.char);
                        self.remove_range(min..max);
                        self.capture_state(env);
                        self.insert_str(min, string);
                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: min + Self::len_in_graphemes(&string) });
                    }
                }
            }
            _ => ()
        }

        self.capture_state(env);
        self.release_state(env);
        self.reposition_cursor(env);
        self.recalculate_offset_to_make_cursor_visible(env);
    }
}

impl MouseEventHandler for PlainTextInput {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, env: &mut Environment) {
        if !self.is_inside(event.get_current_mouse_position()) {
            match event {
                MouseEvent::Press(_, _, _) => {
                    if self.get_focus() == Focus::Focused {
                        self.set_focus_and_request(Focus::FocusReleased, env);
                    }
                }
                _ => ()
            }

            return;
        }

        let text_offset = *self.text_offset.value();

        match event {
            MouseEvent::Press(_, position, modifier) => {
                //self.request_focus(env);
                if !modifier.contains(ModifierKey::SHIFT) {
                    let relative_offset = position.x() - self.position.x() - text_offset;
                    let char_index = Cursor::char_index(relative_offset, &self.glyphs(env));

                    self.drag_start_cursor = Some(Cursor::Single(CursorIndex { line: 0, char: char_index }));
                    if let Cursor::Single(_) = self.cursor {
                        self.cursor = Cursor::Single(CursorIndex { line: 0, char: char_index });
                    }
                }
                self.reposition_cursor(env);
            }
            MouseEvent::Click(_, position, modifier) => {
                self.request_focus(env);
                if modifier == &ModifierKey::SHIFT {
                    let relative_offset = position.x() - self.position.x() - text_offset;
                    let clicked_index = Cursor::char_index(relative_offset, &self.glyphs(env));

                    match self.cursor {
                        Cursor::Single(CursorIndex { line, char }) => {
                            self.cursor = Cursor::Selection {
                                start: CursorIndex { line: 0, char },
                                end: CursorIndex { line: 0, char: clicked_index },
                            }
                        }
                        Cursor::Selection { start: CursorIndex { char, .. }, .. } => {
                            self.cursor = Cursor::Selection {
                                start: CursorIndex { line: 0, char },
                                end: CursorIndex { line: 0, char: clicked_index },
                            }
                        }
                    }
                } else {
                    let relative_offset = position.x() - self.position.x() - text_offset;
                    let char_index = Cursor::char_index(relative_offset, &self.glyphs(env));

                    self.cursor = Cursor::Single(CursorIndex { line: 0, char: char_index });
                }
                self.reposition_cursor(env);
            }
            MouseEvent::NClick(_, position, _, n) => {
                //self.request_focus(env);

                // If the click number is even, select all, otherwise select the clicked word.
                if n % 2 == 1 {
                    self.cursor = Cursor::Selection { start: CursorIndex { line: 0, char: 0 }, end: CursorIndex { line: 0, char: Self::len_in_graphemes(&self.text.value()) } };
                } else {
                    let relative_offset = position.x() - self.position.x() - text_offset;

                    let char_index = Cursor::char_index(relative_offset, &self.glyphs(env));

                    let range = Self::word_index_range(&self.display_text.value(), char_index);

                    self.cursor = Cursor::Selection { start: CursorIndex { line: 0, char: range.start }, end: CursorIndex { line: 0, char: range.end } }
                }
                self.reposition_cursor(env);
            }
            MouseEvent::Drag { to, delta_xy, .. } => {
                // If we do not have focus, just return
                // if self.get_focus() != Focus::Focused { return; }

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
                    let end = CursorIndex { line: 0, char: Self::len_in_graphemes(&self.text.value()) };

                    let max_offset = Cursor::Selection { start, end }.width(&text, &positioned_glyphs);

                    // Since the offset is negative we have to chose the max value
                    *self.text_offset.value_mut() = offset.max(-(max_offset - self.width())).min(0.0);
                }

                let current_relative_offset = to.x() - self.position.x() - text_offset;

                let current_char_index = Cursor::char_index(current_relative_offset, &self.glyphs(env));

                match self.drag_start_cursor {
                    None => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                self.cursor = Cursor::Selection { start: index, end: CursorIndex { line: 0, char: current_char_index } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: current_char_index } }
                            }
                        }
                    }
                    Some(cursor) => {
                        match cursor {
                            Cursor::Single(index) => {
                                self.cursor = Cursor::Selection { start: index, end: CursorIndex { line: 0, char: current_char_index } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: current_char_index } }
                            }
                        }
                        self.drag_start_cursor = None;
                    }
                }
                self.reposition_cursor(env);
            }
            _ => ()
        }
    }
}

impl PlainTextInput {
    /// Insert a string at a given grapheme index.
    fn insert_str(&mut self, index: usize, string: &str) {
        let offset = Self::byte_index_from_graphemes(index, &self.text.value());
        //TODO: This might be rather inefficient
        let mut next_string = self.text.value().clone();
        next_string.insert_str(offset, string);
        self.text.set_value(next_string);
    }

    /// Insert a string at a given grapheme index.
    fn push_str(&mut self, string: &str) {
        //TODO: This might be rather inefficient
        let mut next_string = self.text.value().clone();
        next_string.push_str(string);
        self.text.set_value(next_string);
    }

    /// Get the positioned glyphs of a given string. This is useful when needing to calculate cursor
    /// position, or the width of a given string.
    fn glyphs(&mut self, env: &mut Environment) -> Vec<Glyph> {
        let mut text_scaler: Box<Text> = Text::new(self.display_text.clone())
            .font_size(self.font_size.clone()).wrap_mode(Wrap::None);

        text_scaler.set_position(Position::new(0.0, 0.0));
        let normal_scale = env.get_scale_factor();
        env.set_scale_factor(1.0);
        text_scaler.calculate_size(Dimension::new(100.0, 100.0), env);
        env.set_scale_factor(normal_scale);

        let positioned_glyphs = text_scaler.glyphs();
        positioned_glyphs
    }

    /// Remove a single grapheme at an index.
    fn remove(&mut self, index: usize) {
        let offset = Self::byte_index_from_graphemes(index, &*self.text.value());
        let mut new_string = self.text.value().clone();
        new_string.remove(offset);
        self.text.set_value(new_string);
    }

    /// Remove all the graphemes inside the range,
    fn remove_range(&mut self, index: Range<usize>) {
        let offset_start = Self::byte_index_from_graphemes(index.start, &*self.text.value());
        let offset_end = Self::byte_index_from_graphemes(index.end, &*self.text.value());
        let mut new_string = self.text.value().clone();
        new_string.replace_range(offset_start..offset_end, "");
        self.text.set_value(new_string);
    }

    /// Get the range from the leftmost character in a word, to the current index.
    /// When calculating this, all spaces to the left of the word is included as well.
    fn prev_word_range(text: &String, start_index: usize) -> Range<usize> {
        let mut has_hit_space = false;

        let number_left = text.chars().rev().skip(Self::len_in_graphemes(&text) - start_index).skip_while(|cur| {
            if *cur == ' ' {
                has_hit_space = true;
                true
            } else {
                !has_hit_space
            }
        }).count();

        number_left..start_index
    }

    /// Get the range from the current index to the rightmost character in a word.
    /// When calculating this, all spaces to the right of the word is included as well.
    fn next_word_range(text: &String, start_index: usize) -> Range<usize> {
        let mut has_hit_space = false;

        let number_left = text.chars().skip(start_index).skip_while(|cur| {
            if *cur == ' ' {
                has_hit_space = true;
                true
            } else {
                !has_hit_space
            }
        }).count();

        let new_index = Self::len_in_graphemes(&text) - number_left;

        start_index..new_index
    }

    /// Get a range of the graphemes in the word surrounded by spaces,
    /// where the current index is within. The spaces is not included.
    fn word_index_range(text: &String, start_index: usize) -> Range<usize> {
        let mut max_iter = text.chars().enumerate().skip(start_index).skip_while(|(_, cur)| {
            *cur != ' '
        });

        let mut min_iter = text.chars().rev().enumerate().skip(Self::len_in_graphemes(text) - start_index).skip_while(|(_, cur)| {
            *cur != ' '
        });

        let max = match max_iter.next() {
            None => { Self::len_in_graphemes(text) }
            Some((u, _)) => u
        };

        let min = match min_iter.next() {
            None => 0,
            Some((u, _)) => Self::len_in_graphemes(text) - u
        };

        min..max
    }

    /// Recalculate the position of the cursor and the selection. This will not move the cursor
    /// index, but move the visual positioning of the cursor and the selection box (if selection mode).
    fn reposition_cursor(&mut self, env: &mut Environment) {
        let glyph = self.glyphs(env);
        let text = &*self.display_text.value();

        let index = match &mut self.cursor {
            Cursor::Single(index) => {
                let len_in_graphemes = Self::len_in_graphemes(text);
                *index = CursorIndex { line: 0, char: index.char.min(len_in_graphemes) };
                index
            }
            Cursor::Selection { end, .. } => end
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
        }
    }

    /// This will change the text offset to make the cursor visible. It will result in the text
    /// getting scrolled, such that the entire cursor is visible.
    fn recalculate_offset_to_make_cursor_visible(&mut self, env: &mut Environment) {
        let cursor_x = *self.cursor_x.value();
        let cursor_width = 4.0;
        let current_text_offset = *self.text_offset.value();

        if cursor_x + cursor_width > self.width() && -current_text_offset < cursor_x + cursor_width - self.width() {
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
        }
    }

    fn len_in_graphemes(text: &String) -> usize {
        text.graphemes(true).count()
    }

    /// Get the index of the first byte for a given grapheme index.
    fn byte_index_from_graphemes(index: usize, text: &String) -> usize {
        if text.len() == 0 { return 0; }
        let grapheme_byte_offset = match text.grapheme_indices(true).skip(index).next() {
            None => text.len(),
            Some((g, _)) => g
        };
        grapheme_byte_offset
    }
}

impl OtherEventHandler for PlainTextInput {
    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        match event {
            WidgetEvent::Window(w) => {
                match w {
                    WindowEvent::Resize(_) => {
                        let offset = *self.text_offset.value();
                        let text = self.text.value().clone();
                        let positioned_glyphs = self.glyphs(env);

                        let start = CursorIndex { line: 0, char: 0 };
                        let end = CursorIndex { line: 0, char: Self::len_in_graphemes(&text) };

                        let max_offset = Cursor::Selection { start, end }.width(&text, &positioned_glyphs);

                        // Since the offset is negative we have to chose the max value
                        *self.text_offset.value_mut() = offset.max(-(max_offset - self.width())).min(0.0);
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }
}

impl CommonWidget for PlainTextInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn get_focus(&self) -> Focus {
        self.focus.value().clone()
    }

    fn set_focus(&mut self, focus: Focus) {
        println!("Set focus of plain field to: {:?}", focus);
        *self.focus.value_mut() = focus;
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
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

    fn flexibility(&self) -> u32 {
        1
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for PlainTextInput {}