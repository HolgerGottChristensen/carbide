use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::automatic_style::AutomaticStyle;
use carbide::environment::{EnvironmentKey, Shared};
use carbide::widget::AnySequence;
use crate::context_menu::menu_widget::AnyMenuWidget;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ContextMenuStyleKey;

impl EnvironmentKey for ContextMenuStyleKey {
    type Value = Box<dyn ContextMenuStyle>;
    type Access = Shared;
}

pub trait ContextMenuStyle: Debug + DynClone {
    fn open(&self, menu_items: &dyn AnySequence<dyn AnyMenuWidget>);
}

impl ContextMenuStyle for AutomaticStyle {
    fn open(&self, _menu_items: &dyn AnySequence<dyn AnyMenuWidget>) {

    }
}

clone_trait_object!(ContextMenuStyle);