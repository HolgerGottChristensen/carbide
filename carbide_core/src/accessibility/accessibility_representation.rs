use crate::accessibility::{AccessibilityContext, Accessibility};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{Empty, IntoWidget, Widget, CommonWidget, WidgetSync};
use crate::ModifierWidgetImpl;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Accessibility)]
pub struct AccessibilityRepresentation<C, S> where C: Widget, S: Widget {
    representation: S,
    child: C
}

impl AccessibilityRepresentation<Empty, Empty> {
    pub fn new<C: IntoWidget, S: IntoWidget>(child: C, label: S) -> AccessibilityRepresentation<C::Output, S::Output> {
        AccessibilityRepresentation {
            representation: label.into_widget(),
            child: child.into_widget(),
        }
    }
}

impl<C: Widget, S: Widget> Accessibility for AccessibilityRepresentation<C, S> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env);

        // Process the accessibility of the children
        self.representation.process_accessibility(ctx);
    }
}

impl<C: Widget, S: Widget> CommonWidget for AccessibilityRepresentation<C, S> {
    ModifierWidgetImpl!(self, child: self.child);
}