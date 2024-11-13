use smallvec::SmallVec;
use carbide::accessibility::{Accessibility, AccessibilityAction, AccessibilityContext, AccessibilityNode, Role, Toggled};
use crate::toggle::toggle_value::ToggleValue;
use crate::{enabled_state, CheckBoxValue, EnabledState, PlainCheckBox, PlainCheckBoxDelegate};
use carbide::draw::{Dimension, Position};
use carbide::flags::WidgetFlag;
use carbide::focus::{Focus, Focusable};
use carbide::lifecycle::{InitializationContext, Initialize};
use carbide::state::{IntoReadState, IntoState, LocalState, ReadState, ReadStateExtNew, State, StateExtNew};
use carbide::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetSync};
use carbide::CommonWidgetImpl;
use carbide::event::{AccessibilityEvent, AccessibilityEventContext, AccessibilityEventHandler};
use crate::toggle::{SwitchStyle, ToggleAction, ToggleStyle, ToggleStyleKey};

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Initialize, Accessibility, AccessibilityEvent)]
pub struct Toggle<F, V, E, L> where
    F: State<T=Focus>,
    V: State<T=ToggleValue>,
    E: ReadState<T=bool>,
    L: ReadState<T=String>,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: Box<dyn AnyWidget>,
    role: Role,
    #[state] focus: F,
    #[state] enabled: E,
    #[state] value: V,
    #[state] label: L,
}

impl Toggle<LocalState<Focus>, ToggleValue, EnabledState, String> {
    pub fn new<L: IntoReadState<String>, C: IntoState<ToggleValue>>(label: L, value: C) -> Toggle<LocalState<Focus>, C::Output, EnabledState, L::Output> {
        let focus_state = LocalState::new(Focus::Unfocused);

        let enabled_state = enabled_state();

        Toggle {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: Empty::new().boxed(),
            role: Default::default(),
            focus: focus_state,
            enabled: enabled_state,
            value: value.into_state(),
            label: label.into_read_state(),
        }
    }
}

impl<F: State<T=Focus>, V: State<T=ToggleValue>, E: ReadState<T=bool>, L: ReadState<T=String>> Initialize for Toggle<F, V, E, L> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        if let Some(style) = ctx.env_stack.get::<ToggleStyleKey>() {
            self.child = style.create(self.focus.as_dyn(), self.value.as_dyn(), self.enabled.as_dyn_read(), self.label.as_dyn_read());
            self.role = style.toggle_role();
        } else {
            self.child = SwitchStyle.create(self.focus.as_dyn(), self.value.as_dyn(), self.enabled.as_dyn_read(), self.label.as_dyn_read());
            self.role = SwitchStyle.toggle_role();
        }
    }
}

impl<F: State<T=Focus>, V: State<T=ToggleValue>, E: ReadState<T=bool>, L: ReadState<T=String>> AccessibilityEventHandler for Toggle<F, V, E, L> {
    fn handle_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        match event.action {
            AccessibilityAction::Click => {
                ToggleAction {
                    value: self.value.clone(),
                    focus: self.focus.clone(),
                    enabled: self.enabled.clone(),
                }.trigger(ctx.env_stack);
            }
            AccessibilityAction::Focus => {
                self.request_focus(ctx.env_stack)
            }
            AccessibilityAction::Blur => {
                self.request_blur(ctx.env_stack)
            }
            _ => ()
        }
    }
}

impl<F: State<T=Focus>, V: State<T=ToggleValue>, E: ReadState<T=bool>, L: ReadState<T=String>> Accessibility for Toggle<F, V, E, L> {
    fn role(&self) -> Option<Role> {
        Some(self.role)
    }

    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env_stack);

        let mut children = SmallVec::<[WidgetId; 8]>::new();

        let mut nodes = SmallVec::<[AccessibilityNode; 1]>::new();

        let mut child_ctx = AccessibilityContext {
            env: ctx.env,
            env_stack: ctx.env_stack,
            nodes: &mut nodes,
            parent_id: Some(self.id()),
            children: &mut children,
            hidden: ctx.hidden,
            inherited_label: None,
            inherited_hint: None,
            inherited_value: None,
            inherited_enabled: None,
        };

        // Process the accessibility of the children
        self.foreach_child_direct(&mut |child | {
            child.process_accessibility(&mut child_ctx);
        });

        let mut node = self.accessibility_create_node(ctx).unwrap();

        if node.label().is_none() {
            let labels = nodes.iter().filter_map(|x| x.label()).collect::<Vec<_>>();
            node.set_label(labels.join(", "));
        }

        if node.value().is_none() {
            match &*self.value.value() {
                ToggleValue::True => node.set_toggled(Toggled::True),
                // The toggled mixed reads the checkbox as ticked on MacOS which is not the
                // same as how swiftui reads the checkbox state.
                ToggleValue::Mixed => node.set_value("mixed"),
                ToggleValue::False => node.set_toggled(Toggled::False),
            }
        }

        if !*self.enabled.value() {
            node.set_disabled();
        }

        node.add_action(AccessibilityAction::Click);

        ctx.nodes.push(self.id(), node);

        ctx.children.push(self.id());
    }
}

impl<F: State<T=Focus>, V: State<T=ToggleValue>, E: ReadState<T=bool>, L: ReadState<T=String>> CommonWidget for Toggle<F, V, E, L> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focus);
}