mod inline_style;
mod segmented_style;
mod menu_style;
mod menu;

pub use inline_style::*;
pub use segmented_style::*;
pub use menu_style::*;

use crate::identifiable::AnySelectableWidget;
use carbide::environment::{EnvironmentKey, Shared};
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState};
use carbide::widget::{AnySequence, AnyWidget, WidgetStyle};
use dyn_clone::clone_trait_object;
use std::fmt::Debug;
use carbide::draw::AutomaticStyle;
use crate::picker::picker_selection::PickerSelectionType;

#[derive(Debug, Copy, Clone)]
pub(crate) struct PickerStyleKey;

impl EnvironmentKey for PickerStyleKey {
    type Value = Box<dyn PickerStyle>;
    type Access = Shared;
}

/// Implemented by the built-in picker styles. `create` takes the crate-internal
/// `PickerSelectionType`, so the trait can only be implemented inside this crate.
pub trait PickerStyle: WidgetStyle {
    #[allow(private_interfaces)] // `PickerSelectionType` is crate-internal; see `PickerStyle`.
    fn create(
        &self,
        focus: Box<dyn AnyState<T=Focus>>,
        enabled: Box<dyn AnyReadState<T=bool>>,
        label: Box<dyn AnyReadState<T=String>>,
        model: Box<dyn AnySequence<dyn AnySelectableWidget>>,
        selection_type: PickerSelectionType,
    ) -> Box<dyn AnyWidget>;
}

impl PickerStyle for AutomaticStyle {
    #[allow(private_interfaces)] // `PickerSelectionType` is crate-internal; see `PickerStyle`.
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn AnySequence<dyn AnySelectableWidget>>, selection_type: PickerSelectionType) -> Box<dyn AnyWidget> {
        MenuStyle.create(focus, enabled, label, model, selection_type)
    }
}

clone_trait_object!(PickerStyle);