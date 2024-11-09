use std::any::Any;
use crate::types::*;
use crate::{enabled_state, EnabledState, UnfocusAction};
use carbide_core::accessibility::{Accessibility, AccessibilityAction, AccessibilityContext, AccessibilityNode, Role, Toggled};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::{AccessibilityEvent, AccessibilityEventContext, AccessibilityEventHandler};
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, ReadStateExtNew, State};
use carbide_core::widget::{AnyWidget, CommonWidget, MouseArea, MouseAreaAction, MouseAreaActionContext, Rectangle, Text, Widget, WidgetExt, WidgetId, WidgetSync, ZStack};
use carbide_core::CommonWidgetImpl;
use smallvec::SmallVec;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use carbide::lifecycle::{InitializationContext, Initialize};
use carbide::render::{Render, RenderContext};
use carbide::widget::IntoWidget;
use crate::toggle_style::ToggleStyleKey;

pub trait PlainCheckBoxDelegate: Clone + 'static {
    type Output: IntoWidget;
    fn call(&self, focus: impl ReadState<T=Focus>, checked: impl ReadState<T=CheckBoxValue>, enabled: impl ReadState<T=bool>) -> Self::Output;
}

impl<K, W: Widget> PlainCheckBoxDelegate for K where K: Fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=CheckBoxValue>>, Box<dyn AnyReadState<T=bool>>) -> W + Clone + 'static {
    type Output = W;

    fn call(&self, item: impl ReadState<T=Focus>, index: impl ReadState<T=CheckBoxValue>, enabled: impl ReadState<T=bool>) -> W {
        self(item.as_dyn_read(), index.as_dyn_read(), enabled.as_dyn_read())
    }
}

#[derive(Clone, Widget)]
#[carbide_exclude(Accessibility, AccessibilityEvent, Initialize, Render)]
pub struct PlainCheckBox<F, C, D, E> where
    F: State<T=Focus>,
    C: State<T=CheckBoxValue>,
    D: PlainCheckBoxDelegate,
    E: ReadState<T=bool>,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: Box<dyn AnyWidget>,
    #[state] focus: F,
    #[state] enabled: E,
    #[state] checked: C,
    delegate: D,
}

type DefaultPlainCheckBoxDelegate = fn(focus: Box<dyn AnyReadState<T=Focus>>, selected: Box<dyn AnyReadState<T=CheckBoxValue>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget>;

impl PlainCheckBox<Focus, CheckBoxValue, DefaultPlainCheckBoxDelegate, bool> {
    pub fn new<C: IntoState<CheckBoxValue>>(checked: C) -> PlainCheckBox<LocalState<Focus>, C::Output, DefaultPlainCheckBoxDelegate, EnabledState> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            checked.into_state(),
            focus_state,
            PlainCheckBox::default_delegate,
            enabled_state()
        )
    }

    fn default_delegate(focus: Box<dyn AnyReadState<T=Focus>>, checked: Box<dyn AnyReadState<T=CheckBoxValue>>, _enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let background_color = Map1::read_map(checked.clone(), |value| {
            match value {
                CheckBoxValue::True => EnvironmentColor::Green,
                CheckBoxValue::Mixed => EnvironmentColor::Blue,
                CheckBoxValue::False => EnvironmentColor::Red,
            }
        });

        let val = Map2::read_map(checked, focus, |checked, focus| {
            format!("{:?}, {:?}", *checked, focus)
        });

        ZStack::new((
            Rectangle::new().fill(background_color),
            Text::new(val),
        )).boxed()
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>,> PlainCheckBox<F, C, D, E> {

    pub fn delegate<D2: PlainCheckBoxDelegate>(
        self,
        delegate: D2,
    ) -> PlainCheckBox<F, C, D2, E> {
        Self::new_internal(self.checked, self.focus, delegate, self.enabled)
    }

    pub fn focused<F2: IntoState<Focus>>(self, focused: F2) -> PlainCheckBox<F2::Output, C, D, E> {
        Self::new_internal(self.checked, focused.into_state(), self.delegate, self.enabled)
    }

    pub fn enabled<E2: IntoReadState<bool>>(self, enabled: E2) -> PlainCheckBox<F, C, D, E2::Output> {
        Self::new_internal(
            self.checked,
            self.focus,
            self.delegate,
            enabled.into_read_state(),
        )
    }

    fn new_internal<F2: State<T=Focus>, C2: State<T=CheckBoxValue>, D2: PlainCheckBoxDelegate, E2: ReadState<T=bool>>(
        checked: C2,
        focus: F2,
        delegate: D2,
        enabled: E2,
    ) -> PlainCheckBox<F2, C2, D2, E2> {
        let delegate_widget = delegate.call(focus.as_dyn_read(), checked.as_dyn_read(), enabled.as_dyn_read()).into_widget();

        let child = MouseArea::new(delegate_widget)
            .custom_on_click(CheckboxAction {
                checked: checked.clone(),
                focus: focus.clone(),
                enabled: enabled.clone(),
            }).custom_on_click_outside(UnfocusAction(focus.clone()))
            .focused(focus.clone())
            .boxed();

        PlainCheckBox {
            id: WidgetId::new(),
            focus,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            delegate,
            checked,
            enabled
        }
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>,> Initialize for PlainCheckBox<F, C, D, E> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        println!("Initialzed checkbox");
    }
}


impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> AccessibilityEventHandler for PlainCheckBox<F, C, D, E> {
    fn handle_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        match event.action {
            AccessibilityAction::Click => {
                CheckboxAction {
                    checked: self.checked.clone(),
                    focus: self.focus.clone(),
                    enabled: self.enabled.clone(),
                }.trigger(ctx.env);
            }
            AccessibilityAction::Focus => {
                self.request_focus(ctx.env)
            }
            AccessibilityAction::Blur => {
                self.request_blur(ctx.env)
            }
            _ => ()
        }
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> Accessibility for PlainCheckBox<F, C, D, E> {
    fn role(&self) -> Option<Role> {
        Some(Role::CheckBox)
    }

    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env);

        let mut children = SmallVec::<[WidgetId; 8]>::new();

        let mut nodes = SmallVec::<[AccessibilityNode; 1]>::new();

        let mut child_ctx = AccessibilityContext {
            env: ctx.env,
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
            match &*self.checked.value() {
                CheckBoxValue::True => node.set_toggled(Toggled::True),
                // The toggled mixed reads the checkbox as ticked on MacOS which is not the
                // same as how swiftui reads the checkbox state.
                CheckBoxValue::Mixed => node.set_value("mixed"),
                CheckBoxValue::False => node.set_toggled(Toggled::False),
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

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> Render for PlainCheckBox<F, C, D, E> {
    fn render(&mut self, context: &mut RenderContext) {
        self.sync(context.env);

        println!("{:?}", context.env_new.get::<ToggleStyleKey>());

        self.child.render(context);
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> CommonWidget for PlainCheckBox<F, C, D, E> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> Debug for PlainCheckBox<F, C, D, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(PlainCheckBox))
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focus)
            .field("checked", &self.checked)
            .field("enabled", &self.enabled)
            .finish()
    }
}

#[derive(Debug, Clone)]
struct CheckboxAction<C, F, E> where
    C: State<T=CheckBoxValue>,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
{
    checked: C,
    focus: F,
    enabled: E,
}

impl<C: State<T=CheckBoxValue>, F: State<T=Focus>, E: ReadState<T=bool>> CheckboxAction<C, F, E> {
    fn trigger(&mut self, env: &mut Environment) {
        self.enabled.sync(env);

        if !*self.enabled.value() {
            return;
        }

        self.focus.sync(env);
        self.checked.sync(env);

        if *self.checked.value() == CheckBoxValue::True {
            *self.checked.value_mut() = CheckBoxValue::False;
        } else {
            *self.checked.value_mut() = CheckBoxValue::True;
        }

        if *self.focus.value() != Focus::Focused {
            *self.focus.value_mut() = Focus::FocusRequested;
            env.request_focus(Refocus::FocusRequest);
        }
    }
}

impl<C: State<T=CheckBoxValue>, F: State<T=Focus>, E: ReadState<T=bool>> MouseAreaAction for CheckboxAction<C, F, E> {
    fn call(&mut self, ctx: MouseAreaActionContext) { self.trigger(ctx.env) }
}