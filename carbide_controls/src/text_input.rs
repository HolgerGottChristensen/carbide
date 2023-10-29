use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::focus::Focus;
use carbide_core::render::{Render};
use carbide_core::state::{IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, State, TState};
use carbide_core::widget::{CommonWidget, CornerRadii, EdgeInsets, Rectangle, RoundedRectangle, AnyWidget, WidgetExt, WidgetId, ZStack, Widget};

use crate::{enabled_state, EnabledState, PASSWORD_CHAR, PlainTextInput};

const VERTICAL_PADDING: f64 = 0.0;
const HORIZONTAL_PADDING: f64 = 5.0;

/// A plain text input widget. The widget contains no specific styling, other than text color,
/// cursor color/width and selection color. Most common logic has been implemented, such as
/// key shortcuts, mouse click and drag select along with copy and paste. For an example of
/// how to use this widget look at examples/plain_text_input
#[derive(Debug, Clone, Widget)]
pub struct TextInput<F, O, T, E> where
    F: State<T=Focus>,
    O: ReadState<T=Option<char>>,
    T: State<T=Result<String, String>>,
    E: ReadState<T=bool>,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Box<dyn AnyWidget>,
    obscure: O,

    #[state] text: T,
    #[state] focus: F,
    #[state] enabled: E,
}

impl TextInput<Focus, Option<char>, Result<String, String>, bool> {
    pub fn new<T: IntoState<Result<String, String>>>(text: T) -> TextInput<LocalState<Focus>, Option<char>, T::Output, EnabledState> {
        let focus = LocalState::new(Focus::Unfocused);
        let obscure = None;
        println!("Before into");
        let text = text.into_state();
        println!("After into");

        Self::new_internal(text, focus, obscure, enabled_state())
    }
}

impl<F: State<T=Focus>, O: ReadState<T=Option<char>>, T: State<T=Result<String, String>>, E: ReadState<T=bool>> TextInput<F, O, T, E> {
    pub fn enabled<E2: IntoReadState<bool>>(self, enabled: E2) -> TextInput<F, O, T, E2::Output> {
        Self::new_internal(
            self.text,
            self.focus,
            self.obscure,
            enabled.into_read_state(),
        )
    }

    pub fn obscure(self) -> TextInput<F, Option<char>, T, E> {
        Self::new_internal(
            self.text,
            self.focus,
            Some(PASSWORD_CHAR),
            self.enabled,
        )
    }

    pub fn obscure_with<O2: IntoReadState<Option<char>>>(self, obscure: O2) -> TextInput<F, O2::Output, T, E> {
        Self::new_internal(
            self.text,
            self.focus,
            obscure.into_read_state(),
            self.enabled,
        )
    }

    fn new_internal<F2: State<T=Focus>, O2: ReadState<T=Option<char>>, T2: State<T=Result<String, String>>, E2: ReadState<T=bool>>(
        text: T2,
        focus: F2,
        obscure: O2,
        enabled: E2,
    ) -> TextInput<F2, O2, T2, E2> {

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

        let label_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::Label
            } else {
                EnvironmentColor::SecondaryLabel
            }
        });

        let background_color = Map1::read_map(enabled.clone(), |enabled| {
            if *enabled {
                EnvironmentColor::SecondarySystemBackground
            } else {
                EnvironmentColor::TertiarySystemBackground
            }
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
            .text_color(label_color)
            .cursor_widget(Rectangle::new().fill(EnvironmentColor::Label).boxed())
            .selection_widget(Rectangle::new().fill(darkened_selection_color).boxed())
            .focused(focus.clone())
            .enabled(enabled.clone())
            .obscure(obscure.clone())
            .clip()
            .padding(EdgeInsets::vertical_horizontal(VERTICAL_PADDING, HORIZONTAL_PADDING));

        let child = ZStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(3.0))
                .fill(background_color)
                .stroke(stroke_color)
                .stroke_style(1.0)
                .boxed(),
            text_widget.boxed()
        ]).frame_fixed_height(22.0)
            .boxed();


        TextInput {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child,
            obscure,
            text,
            focus,
            enabled,
        }
    }
}

impl<F: State<T=Focus>, O: ReadState<T=Option<char>>, T: State<T=Result<String, String>>, E: ReadState<T=bool>,> CommonWidget for TextInput<F, O, T, E> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: 1);
}

impl<F: State<T=Focus>, O: ReadState<T=Option<char>>, T: State<T=Result<String, String>>, E: ReadState<T=bool>,> WidgetExt for TextInput<F, O, T, E> {}
