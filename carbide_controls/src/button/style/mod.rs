mod plain;
mod plain_prominent;
mod bordered;
mod bordered_prominent;

use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::environment::EnvironmentKey;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState};
use carbide::widget::{AnyWidget, Widget};
pub use plain::PlainStyle;
pub use plain_prominent::PlainProminentStyle;
pub use bordered::BorderedStyle;
pub use bordered_prominent::BorderedProminentStyle;
use carbide::draw::AutomaticStyle;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ButtonStyleKey;

impl EnvironmentKey for ButtonStyleKey {
    type Value = Box<dyn ButtonStyle>;
}

pub trait ButtonStyle: Debug + DynClone {
    fn create(
        &self,
        label: Box<dyn AnyWidget>,
        focus: Box<dyn AnyReadState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        hovered: Box<dyn AnyReadState<T=bool>>,
        pressed: Box<dyn AnyReadState<T=bool>>,
    ) -> Box<dyn AnyWidget>;
}

impl ButtonStyle for AutomaticStyle {
    fn create(&self, label: Box<dyn AnyWidget>, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        BorderedStyle.create(label, focus, enabled, hovered, pressed)
    }
}

clone_trait_object!(ButtonStyle);