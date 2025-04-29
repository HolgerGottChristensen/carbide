use crate::widget::WidgetId;
use crate::accessibility::{AccessibilityContext, Accessibility};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Empty, IntoWidget, Widget, CommonWidget, WidgetSync, Identifiable};
use crate::ModifierWidgetImpl;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Accessibility)]
pub struct AccessibilityLabel<C, S> where C: Widget, S: ReadState<T=String> {
    #[state] label: S,
    child: C
}

impl AccessibilityLabel<Empty, String> {
    pub fn new<C: IntoWidget, S: IntoReadState<String>>(child: C, label: S) -> AccessibilityLabel<C::Output, S::Output> {
        AccessibilityLabel {
            label: label.into_read_state(),
            child: child.into_widget(),
        }
    }
}

impl<C: Widget, S: ReadState<T=String>> Accessibility for AccessibilityLabel<C, S> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env);

        let label = &*self.label.value();

        let mut child_ctx = AccessibilityContext {
            env: ctx.env,
            nodes: ctx.nodes,
            parent_id: ctx.parent_id,
            children: ctx.children,
            hidden: ctx.hidden,
            inherited_label: Some(label),
            inherited_hint: ctx.inherited_hint,
            inherited_value: ctx.inherited_value,
            inherited_enabled: ctx.inherited_enabled,
        };

        // Process the accessibility of the children
        self.child.process_accessibility(&mut child_ctx);
    }
}

impl<C: Widget, S: ReadState<T=String>> Identifiable for AccessibilityLabel<C, S> {
    fn id(&self) -> WidgetId {
        self.child.id()
    }
}

impl<C: Widget, S: ReadState<T=String>> CommonWidget for AccessibilityLabel<C, S> {
    ModifierWidgetImpl!(self, child: self.child);
}