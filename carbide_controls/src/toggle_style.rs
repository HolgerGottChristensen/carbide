use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::environment::Key;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ToggleStyleKey;

impl Key for ToggleStyleKey {
    type Value = Box<dyn ToggleStyle>;
}

pub trait ToggleStyle: Debug + DynClone + 'static {

}

clone_trait_object!(ToggleStyle);

#[derive(Debug, Clone)]
pub struct SwitchStyle;

impl ToggleStyle for SwitchStyle {}

#[derive(Debug, Clone)]
pub struct CheckboxStyle;

impl ToggleStyle for CheckboxStyle {}