use std::fmt::Debug;
use carbide::environment::{EnvironmentNew, Key};
use carbide::widget::EnvUpdatingNew;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::IntoReadState;
use carbide_core::widget::{AnyWidget, EdgeInsets, EnvUpdating, HStack, Rectangle, Text, WidgetExt};

use crate::{Help, Labelled};
use crate::toggle_style::{ToggleStyle, ToggleStyleKey};

type Enabled<C, T, S> = EnvUpdating<C, T, S>;

pub trait ControlsExt: WidgetExt {
    fn help<H: IntoReadState<String>>(self, help: H) -> Help<Self> {
        Help::new(
            self,
            Text::new(help)
                .padding(EdgeInsets::vertical_horizontal(1.0, 5.0))
                .background(Rectangle::new().fill(EnvironmentColor::Accent))
                .boxed()
        )
    }

    fn label<L: IntoReadState<String>>(self, label: L) -> Labelled<HStack<Vec<Box<dyn AnyWidget>>>, L::Output> {
        Labelled::new(label, self)
    }

    fn enabled<E: IntoReadState<bool>>(self, enabled: E) -> Enabled<Self, bool, E::Output> {
        EnvUpdating::new("enabled", enabled.into_read_state(), self)
    }

    fn toggle_style(self, value: impl ToggleStyle) -> EnvUpdatingNew<Self, impl Key> {
        EnvUpdatingNew::<Self, ToggleStyleKey>::new(Box::new(value) as Box<dyn ToggleStyle>, self)
    }
}

impl<T> ControlsExt for T where T: WidgetExt {}