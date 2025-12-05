use std::fmt::Debug;
use carbide::environment::{EnvironmentKey};
use carbide::state::{ReadState, StateContract, ValueState};
use carbide::widget::{EnvUpdatingNew, EnvUpdatingNew2, Widget};
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::IntoReadState;
use carbide_core::widget::{AnyWidget, EdgeInsets, HStack, Rectangle, Text, WidgetExt};

use crate::{EnabledKey, Help};
use crate::button::{ButtonStyle, ButtonStyleKey};
use crate::color_picker::{ColorPickerStyle, ColorPickerStyleKey};
use crate::labelled::Labelled;
use crate::picker::{PickerStyle, PickerStyleKey, Tagged};
use crate::slider::{SliderStyle, SliderStyleKey};
use crate::toggle::{ToggleStyle, ToggleStyleKey};

type Enabled<C, K, V> = EnvUpdatingNew2<C, K, V>;

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

    fn enabled<E: IntoReadState<bool>>(self, enabled: E) -> Enabled<Self, impl EnvironmentKey<Value=bool>, impl ReadState<T=bool>> {
        EnvUpdatingNew2::<Self, EnabledKey, E::Output>::new(enabled.into_read_state(), self)
    }

    fn toggle_style(self, value: impl ToggleStyle) -> impl Widget {
        EnvUpdatingNew::<Self, ToggleStyleKey>::new(Box::new(value) as Box<dyn ToggleStyle>, self)
    }

    fn picker_style(self, value: impl PickerStyle + 'static) -> impl Widget {
        EnvUpdatingNew::<Self, PickerStyleKey>::new(Box::new(value) as Box<dyn PickerStyle>, self)
    }

    fn button_style(self, value: impl ButtonStyle + 'static) -> impl Widget {
        EnvUpdatingNew::<Self, ButtonStyleKey>::new(Box::new(value) as Box<dyn ButtonStyle>, self)
    }

    fn color_picker_style(self, value: impl ColorPickerStyle + 'static) -> impl Widget {
        EnvUpdatingNew::<Self, ColorPickerStyleKey>::new(Box::new(value) as Box<dyn ColorPickerStyle>, self)
    }

    fn slider_style(self, value: impl SliderStyle + 'static) -> impl Widget {
        EnvUpdatingNew::<Self, SliderStyleKey>::new(Box::new(value) as Box<dyn SliderStyle>, self)
    }

    fn tag<T: StateContract + PartialEq>(self, tag: T) -> Tagged<T, impl ReadState<T=T>, Self> {
        Tagged::new(self, ValueState::new(tag))
    }

    fn tag_state<T: StateContract + PartialEq, S: ReadState<T=T>>(self, tag: S) -> Tagged<T, S, Self> {
        Tagged::new(self, tag)
    }
}

impl<T> ControlsExt for T where T: WidgetExt {}