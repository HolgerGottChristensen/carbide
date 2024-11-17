use carbide::accessibility::Role;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState};
use carbide::widget::AnyWidget;
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