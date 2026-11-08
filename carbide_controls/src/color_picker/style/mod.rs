mod plain;

use std::fmt::Debug;
use dyn_clone::clone_trait_object;
use carbide::automatic_style::AutomaticStyle;
use carbide::draw::Color;
use carbide::environment::{EnvironmentKey, Shared};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState};
use carbide::widget::{AnyWidget, WidgetStyle};

pub use plain::PlainStyle;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ColorPickerStyleKey;

impl EnvironmentKey for ColorPickerStyleKey {
    type Value = Box<dyn ColorPickerStyle>;
    type Access = Shared;
}

pub trait ColorPickerStyle: WidgetStyle {
    fn create(
        &self,
        label: Box<dyn AnyWidget>,
        focus: Box<dyn AnyState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        hovered: Box<dyn AnyReadState<T=bool>>,
        pressed: Box<dyn AnyReadState<T=bool>>,
        value: Box<dyn AnyState<T=Color>>
    ) -> Box<dyn AnyWidget>;
}

impl ColorPickerStyle for AutomaticStyle {
    fn create(&self, label: Box<dyn AnyWidget>, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, value: Box<dyn AnyState<T=Color>>) -> Box<dyn AnyWidget> {
        PlainStyle.create(label, focus, enabled, hovered, pressed, value)
    }
}


clone_trait_object!(ColorPickerStyle);