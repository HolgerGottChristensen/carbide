use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::environment::Key;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, IntoReadState, ReadState};
use carbide::widget::AnyWidget;
use crate::CheckBoxValue;
use crate::toggle::toggle_value::ToggleValue;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ToggleStyleKey;

impl Key for ToggleStyleKey {
    type Value = Box<dyn ToggleStyle>;
}

pub trait ToggleStyle: Debug + DynClone + 'static {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, value: Box<dyn AnyState<T=ToggleValue>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>) -> Box<dyn AnyWidget>;
}

clone_trait_object!(ToggleStyle);