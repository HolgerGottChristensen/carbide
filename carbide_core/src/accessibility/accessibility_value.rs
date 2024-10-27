use crate::accessibility::{AccessibilityContext, Accessibility};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Empty, IntoWidget, Widget, CommonWidget, WidgetSync};
use crate::ModifierWidgetImpl;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Accessibility)]
pub struct AccessibilityValue<C, S> where C: Widget, S: ReadState<T=String> {
    #[state] value: S,
    child: C
}

impl AccessibilityValue<Empty, String> {
    pub fn new<C: IntoWidget, S: IntoReadState<String>>(child: C, label: S) -> AccessibilityValue<C::Output, S::Output> {
        AccessibilityValue {
            value: label.into_read_state(),
            child: child.into_widget(),
        }
    }
}

impl<C: Widget, S: ReadState<T=String>> Accessibility for AccessibilityValue<C, S> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env);

        let value = &*self.value.value();

        let mut child_ctx = AccessibilityContext {
            env: ctx.env,
            nodes: ctx.nodes,
            parent_id: ctx.parent_id,
            children: ctx.children,
            hidden: ctx.hidden,
            inherited_label: ctx.inherited_label,
            inherited_hint: ctx.inherited_hint,
            inherited_value: Some(value),
            inherited_enabled: ctx.inherited_enabled
        };

        // Process the accessibility of the children
        self.child.process_accessibility(&mut child_ctx);
    }
}

impl<C: Widget, S: ReadState<T=String>> CommonWidget for AccessibilityValue<C, S> {
    ModifierWidgetImpl!(self, child: self.child);
}