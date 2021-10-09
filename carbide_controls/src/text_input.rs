use std::ops::{DerefMut, Range};

use copypasta::{ClipboardContext, ClipboardProvider};
use unicode_segmentation::UnicodeSegmentation;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentFontSize};
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseButton, MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent, WindowEvent};
use carbide_core::focus::Focus;
use carbide_core::layout::BasicLayouter;
use carbide_core::prelude::{EnvironmentColor, Layout};
use carbide_core::state::{BoolState, ColorState, F64State, FocusState, LocalState, State, StringState, TState, U32State};
use carbide_core::text::Glyph;
use carbide_core::widget::{CommonWidget, CornerRadii, EdgeInsets, HStack, Id, Rectangle, RoundedRectangle, SCALE, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut, ZStack};
use carbide_core::widget::Wrap;

use crate::{PlainTextInput, TextInputState};

/// A plain text input widget. The widget contains no specific styling, other than text color,
/// cursor color/width and selection color. Most common logic has been implemented, such as
/// key shortcuts, mouse click and drag select along with copy and paste. For an example of
/// how to use this widget look at examples/plain_text_input
#[derive(Debug, Clone, Widget)]
pub struct TextInput {
    id: Id,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state] focus: FocusState,
    #[state] is_error: BoolState,
}

impl TextInput {
    pub fn new<T: Into<TextInputState>>(text: T) -> Box<Self> {
        let text = text.into();
        let focus_state: FocusState = LocalState::new(Focus::Unfocused).into();

        let cursor_color: ColorState = EnvironmentColor::Label.into();

        let selection_color: ColorState = EnvironmentColor::Accent.into();
        let darkened_selection_color = selection_color.darkened(0.2);

        let is_error: BoolState = text.clone().into();
        let is_error_stroke: BoolState = is_error.clone();

        let stroke_color = focus_state.mapped_env(move |focus: &Focus, _: &_, env: &Environment| {
            let e = is_error_stroke.clone();
            if *e.value() {
                env.env_color(EnvironmentColor::Red).unwrap()
            } else {
                match focus {
                    Focus::Focused => {
                        env.env_color(EnvironmentColor::Accent).unwrap()
                    }
                    _ => {
                        env.env_color(EnvironmentColor::OpaqueSeparator).unwrap()
                    }
                }
            }
        });

        let child = ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(EnvironmentColor::SecondarySystemBackground)
                .stroke(stroke_color)
                .stroke_style(1.0),
            PlainTextInput::new(text)
                .focus(focus_state.clone())
                .cursor_color(cursor_color)
                .selection_color(darkened_selection_color)
                .clip()
                .padding(EdgeInsets::single(0.0, 0.0, 5.0, 5.0)),
        ]).frame(SCALE, 22);

        Box::new(TextInput {
            id: Id::new_v4(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            focus: focus_state,
            is_error,
        })
    }
}

impl CommonWidget for TextInput {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
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

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for TextInput {}