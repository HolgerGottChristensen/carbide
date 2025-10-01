use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::automatic_style::AutomaticStyle;
use carbide::color::Color;
use carbide::environment::{Environment, EnvironmentKey};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState};
use carbide::widget::{AnySequence, AnyWidget, Sequence};
use crate::button::BorderedStyle;
use crate::context_menu::menu_widget::AnyMenuWidget;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ContextMenuStyleKey;

impl EnvironmentKey for ContextMenuStyleKey {
    type Value = Box<dyn ContextMenuStyle>;
}

pub trait ContextMenuStyle: Debug + DynClone {
    fn open(&self, menu_items: &dyn AnySequence<dyn AnyMenuWidget>);
}

impl ContextMenuStyle for AutomaticStyle {
    fn open(&self, menu_items: &dyn AnySequence<dyn AnyMenuWidget>) {

    }
}

clone_trait_object!(ContextMenuStyle);