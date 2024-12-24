use carbide::accessibility::Role;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState};
use carbide::widget::{AnySequence, AnyWidget};
use crate::button::{BorderedStyle, ButtonStyle};
use crate::identifiable::AnySelectableWidget;
use crate::picker::{MenuStyle, PickerSelectionType, PickerStyle};
use crate::toggle::{CheckboxStyle, ToggleStyle, ToggleValue};

#[derive(Debug, Clone)]
pub struct AutomaticStyle;

impl ToggleStyle for AutomaticStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, value: Box<dyn AnyState<T=ToggleValue>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>) -> Box<dyn AnyWidget> {
        CheckboxStyle.create(focus, value, enabled, label)
    }

    fn toggle_role(&self) -> Role {
        CheckboxStyle.toggle_role()
    }
}

impl ButtonStyle for AutomaticStyle {
    fn create(&self, label: Box<dyn AnyWidget>, focus: Box<dyn AnyReadState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        BorderedStyle.create(label, focus, enabled, hovered, pressed)
    }
}

impl PickerStyle for AutomaticStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn AnySequence<dyn AnySelectableWidget>>, selection_type: PickerSelectionType) -> Box<dyn AnyWidget> {
        MenuStyle.create(focus, enabled, label, model, selection_type)
    }
}