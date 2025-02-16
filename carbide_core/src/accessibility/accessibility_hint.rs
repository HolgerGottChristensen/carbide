use carbide::widget::Identifiable;
use crate::accessibility::{AccessibilityContext, Accessibility};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Empty, IntoWidget, Widget, CommonWidget, WidgetSync, WidgetId};
use crate::ModifierWidgetImpl;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Accessibility)]
pub struct AccessibilityHint<C, S> where C: Widget, S: ReadState<T=String> {
    #[state] hint: S,
    child: C
}

impl AccessibilityHint<Empty, String> {
    pub fn new<C: IntoWidget, S: IntoReadState<String>>(child: C, label: S) -> AccessibilityHint<C::Output, S::Output> {
        AccessibilityHint {
            hint: label.into_read_state(),
            child: child.into_widget(),
        }
    }
}

impl<C: Widget, S: ReadState<T=String>> Accessibility for AccessibilityHint<C, S> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env);

        let hint = &*self.hint.value();

        let mut child_ctx = AccessibilityContext {
            env: ctx.env,
            nodes: ctx.nodes,
            parent_id: ctx.parent_id,
            children: ctx.children,
            hidden: ctx.hidden,
            inherited_label: ctx.inherited_label,
            inherited_hint: Some(hint),
            inherited_value: ctx.inherited_value,
            inherited_enabled: ctx.inherited_enabled,
        };

        // Process the accessibility of the children
        self.child.process_accessibility(&mut child_ctx);
    }
}

impl<C: Widget, S: ReadState<T=String>> Identifiable for AccessibilityHint<C, S> {
    fn id(&self) -> WidgetId {
        self.child.id()
    }
}

impl<C: Widget, S: ReadState<T=String>> CommonWidget for AccessibilityHint<C, S> {
    ModifierWidgetImpl!(self, child: self.child);
}