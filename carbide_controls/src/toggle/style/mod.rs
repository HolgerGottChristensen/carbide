mod switch_style;
mod checkbox_style;
mod button_style;

pub use button_style::*;
use carbide_core::environment::Key;
use carbide_core::focus::Focus;
use carbide_core::state::{AnyReadState, AnyState};
use carbide_core::widget::AnyWidget;
pub use checkbox_style::*;
use dyn_clone::{clone_trait_object, DynClone};
use std::fmt::Debug;
use carbide::accessibility::Role;
use carbide::draw::AutomaticStyle;
pub use switch_style::*;
use crate::toggle::ToggleValue;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ToggleStyleKey;

impl Key for ToggleStyleKey {
    type Value = Box<dyn ToggleStyle>;
}

pub trait ToggleStyle: Debug + DynClone + 'static {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, value: Box<dyn AnyState<T=ToggleValue>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>) -> Box<dyn AnyWidget>;

    fn toggle_role(&self) -> Role;
}

impl ToggleStyle for AutomaticStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, value: Box<dyn AnyState<T=ToggleValue>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>) -> Box<dyn AnyWidget> {
        CheckboxStyle.create(focus, value, enabled, label)
    }

    fn toggle_role(&self) -> Role {
        CheckboxStyle.toggle_role()
    }
}

clone_trait_object!(ToggleStyle);