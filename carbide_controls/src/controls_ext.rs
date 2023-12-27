use carbide_core::environment::{EnvironmentColor, EnvironmentStateContainer};
use carbide_core::state::IntoReadState;
use carbide_core::state::ReadStateExtNew;
use carbide_core::widget::{EdgeInsets, EnvUpdating, HStack, Rectangle, Text, AnyWidget, WidgetExt};

use crate::{Help, Labelled};

type Enabled<T> = EnvUpdating<T>;

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

    fn enabled<E: IntoReadState<bool>>(self, enabled: E) -> Enabled<Self> {
        let mut e = EnvUpdating::new(self);
        e.add(EnvironmentStateContainer::Bool {
            key: "enabled",
            value: enabled.into_read_state().as_dyn_read(),
        });

        e
    }
}

impl<T> ControlsExt for T where T: WidgetExt {}