use carbide::environment::EnvironmentFontSize;
use carbide::widget::WidgetExt;
use crate::draw::Color;
use crate::environment::EnvironmentColor;
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Text, Widget};

// ---------------------------------------------------
//  Definitions
// ---------------------------------------------------

pub trait IntoWidget: Clone {
    type Output: Widget + WidgetExt;

    fn into_widget(self) -> Self::Output;
}

// ---------------------------------------------------
//  Implementations
// ---------------------------------------------------

impl<T> IntoWidget for T where T: Widget + WidgetExt {
    type Output = T;

    fn into_widget(self) -> Self::Output {
        self
    }
}

impl IntoWidget for String {
    type Output = Text<String, <EnvironmentFontSize as IntoReadState<u32>>::Output, <EnvironmentColor as IntoReadState<Color>>::Output>;

    fn into_widget(self) -> Self::Output {
        Text::new(self)
            .color(EnvironmentColor::Label)
            .font_size(EnvironmentFontSize::Body)
    }
}

impl IntoWidget for &'static str {
    type Output = Text<String, <EnvironmentFontSize as IntoReadState<u32>>::Output, <EnvironmentColor as IntoReadState<Color>>::Output>;

    fn into_widget(self) -> Self::Output {
        Text::new(self.to_string())
            .color(EnvironmentColor::Label)
            .font_size(EnvironmentFontSize::Body)
    }
}