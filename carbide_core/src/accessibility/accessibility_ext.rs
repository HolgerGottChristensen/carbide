use crate::widget::WidgetExt;
use crate::accessibility::accessibility_hint::AccessibilityHint;
use crate::accessibility::accessibility_label::AccessibilityLabel;
use crate::accessibility::accessibility_representation::AccessibilityRepresentation;
use crate::accessibility::accessibility_value::AccessibilityValue;
use crate::state::IntoReadState;
use crate::widget::IntoWidget;

pub trait AccessibilityExt: WidgetExt {
    fn accessibility_label<S: IntoReadState<String>>(self, label: S) -> AccessibilityLabel<Self, S::Output> {
        AccessibilityLabel::new(self, label)
    }

    fn accessibility_hint<S: IntoReadState<String>>(self, hint: S) -> AccessibilityHint<Self, S::Output> {
        AccessibilityHint::new(self, hint)
    }

    fn accessibility_value<S: IntoReadState<String>>(self, value: S) -> AccessibilityValue<Self, S::Output> {
        AccessibilityValue::new(self, value)
    }

    /// Represent the widget using an alternative representation, using the widget tree provided.
    /// The representation is only used to create an accessibility tree, and does not have visuals,
    /// logic or handle events.
    fn accessibility_representation<S: IntoWidget>(self, representation: S) -> AccessibilityRepresentation<Self, S::Output> {
        AccessibilityRepresentation::new(self, representation)
    }
}

impl<T> AccessibilityExt for T where T: WidgetExt {}