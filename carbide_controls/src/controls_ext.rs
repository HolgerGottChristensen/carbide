use carbide_core::environment::EnvironmentColor;
use carbide_core::state::IntoReadState;
use carbide_core::widget::{EdgeInsets, Rectangle, Text, WidgetExt};
use crate::Help;

pub trait ControlsExt: WidgetExt {
    fn help<H: IntoReadState<String>>(self, help: H) -> Help<Self> {
        Help::new(
            self,
            Text::new(help)
                .padding(EdgeInsets::vertical_horizontal(1.0, 5.0))
                .background(*Rectangle::new().fill(EnvironmentColor::Accent))
                .boxed()
        )
    }
}

impl<T> ControlsExt for T where T: WidgetExt {}