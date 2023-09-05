use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position, Color};
use carbide_core::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide_core::focus::Focus;
use carbide_core::flags::Flags;
use carbide_core::layout::{BasicLayouter, Layout, Layouter};
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{IntoReadState, IntoState, LocalState, Map1, Map2, Map5, ReadState, State, TState};
use carbide_core::widget::{CommonWidget, CornerRadii, EdgeInsets, Rectangle, RoundedRectangle, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut, ZStack};

use crate::{PASSWORD_CHAR, PlainTextInput, TextInputState};

const VERTICAL_PADDING: f64 = 0.0;
const HORIZONTAL_PADDING: f64 = 5.0;

/// A plain text input widget. The widget contains no specific styling, other than text color,
/// cursor color/width and selection color. Most common logic has been implemented, such as
/// key shortcuts, mouse click and drag select along with copy and paste. For an example of
/// how to use this widget look at examples/plain_text_input
#[derive(Debug, Clone, Widget)]
pub struct TextInput<F, O, T> where
    F: State<T=Focus>,
    O: ReadState<T=Option<char>>,
    T: State<T=Result<String, String>>,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Box<dyn Widget>,
    obscure: O,

    #[state] text: T,
    #[state] focus: F,
}

impl TextInput<Focus, Option<char>, Result<String, String>> {
    pub fn new<T: IntoState<Result<String, String>>>(text: T) -> TextInput<TState<Focus>, Option<char>, T::Output> {
        let focus = LocalState::new(Focus::Unfocused);
        let obscure = None;
        let text = text.into_state();

        Self::new_internal(text, focus, obscure)
    }
}

impl<F: State<T=Focus>, O: ReadState<T=Option<char>>, T: State<T=Result<String, String>>> TextInput<F, O, T> {

    pub fn obscure(self) -> TextInput<F, Option<char>, T> {
        Self::new_internal(
            self.text,
            self.focus,
            Some(PASSWORD_CHAR),
        )
    }

    pub fn obscure_with<O2: IntoReadState<Option<char>>>(self, obscure: O2) -> TextInput<F, O2::Output, T> {
        Self::new_internal(
            self.text,
            self.focus,
            obscure.into_read_state(),
        )
    }

    fn new_internal<F2: State<T=Focus>, O2: ReadState<T=Option<char>>, T2: State<T=Result<String, String>>>(
        text: T2,
        focus: F2,
        obscure: O2,
    ) -> TextInput<F2, O2, T2> {

        let selection_color = EnvironmentColor::Accent.color();
        let darkened_selection_color = Map1::read_map(selection_color, |col| col.darkened(0.2));

        let stroke_color = Map2::read_map(focus.clone(), text.clone(), |focus: &Focus, text: &Result<String, String>| {
            if text.is_err() {
                return EnvironmentColor::Red;
            }

            if *focus == Focus::Focused {
                return EnvironmentColor::Accent;
            }

            EnvironmentColor::OpaqueSeparator
        });

        // TODO: Change to cached map when available
        let text_state = Map1::map(
            text.clone(),
            |res| match res.as_ref() {
                Ok(s) | Err(s) => s.clone(),
            },
            |new, _| Some(Ok(new)),
        );

        let text_widget = PlainTextInput::new(text_state)
            .font_size(EnvironmentFontSize::Body)
            .text_color(EnvironmentColor::Label)
            .cursor_widget(Rectangle::new().fill(EnvironmentColor::Label))
            .selection_widget(Rectangle::new().fill(darkened_selection_color))
            .focused(focus.clone())
            .obscure(obscure.clone())
            .padding(EdgeInsets::vertical_horizontal(VERTICAL_PADDING, HORIZONTAL_PADDING));

        let child = ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(EnvironmentColor::SecondarySystemBackground)
                .stroke(stroke_color)
                .stroke_style(1.0),
            text_widget.boxed()
        ]).frame_fixed_height(22.0);


        TextInput {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child,
            obscure,
            text,
            focus,
        }
    }
}

impl<F: State<T=Focus>, O: ReadState<T=Option<char>>, T: State<T=Result<String, String>>> CommonWidget for TextInput<F, O, T> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 1);
}

impl<F: State<T=Focus>, O: ReadState<T=Option<char>>, T: State<T=Result<String, String>>> WidgetExt for TextInput<F, O, T> {}
