use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::draw::{AutomaticStyle, Color};
use carbide::environment::{EnvironmentStack, Key};
use carbide::state::{AnyReadState, AnyState};
use crate::color_dialog::style::macos::MacOSNativeColorDialogStyle;
use crate::NativeStyle;
use crate::open_dialog::style::OpenDialogStyle;

mod macos;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ColorDialogStyleKey;

impl Key for ColorDialogStyleKey {
    type Value = Box<dyn ColorDialogStyle>;
}

pub trait ColorDialogStyle: Debug + DynClone + 'static {
    fn open(&self, color: Box<dyn AnyState<T=Color>>, show_alpha: Box<dyn AnyReadState<T=bool>>, env_stack: &mut EnvironmentStack);
}

impl ColorDialogStyle for AutomaticStyle {
    fn open(&self, color: Box<dyn AnyState<T=Color>>, show_alpha: Box<dyn AnyReadState<T=bool>>, env_stack: &mut EnvironmentStack) {
        ColorDialogStyle::open(&NativeStyle, color, show_alpha, env_stack)
    }
}

impl ColorDialogStyle for NativeStyle {
    fn open(&self, color: Box<dyn AnyState<T=Color>>, show_alpha: Box<dyn AnyReadState<T=bool>>, env_stack: &mut EnvironmentStack) {
        MacOSNativeColorDialogStyle.open(color, show_alpha, env_stack)
    }
}


clone_trait_object!(ColorDialogStyle);