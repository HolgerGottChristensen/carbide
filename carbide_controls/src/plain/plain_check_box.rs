use std::fmt::{Debug, Formatter};
use smallvec::SmallVec;
use carbide::accessibility::{Accessibility, AccessibilityAction, AccessibilityContext, AccessibilityNode, NodeBuilder, Point, Rect, Role, Size, Toggled};
use carbide::{accessibility, closure};
use carbide::event::{AccessibilityEvent, AccessibilityEventContext, AccessibilityEventHandler};
use carbide::focus::Focusable;
use carbide::widget::{Action, MouseAreaAction, MouseAreaActionContext, WidgetSync};
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::ModifierKey;
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::{Focus, Refocus};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, ReadStateExtNew, State};
use carbide_core::widget::{AnyWidget, CommonWidget, MouseArea, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack};

use crate::{enabled_state, EnabledState};
use crate::types::*;

pub trait PlainCheckBoxDelegate: Clone + 'static {
    fn call(&self, focus: impl ReadState<T=Focus>, checked: impl ReadState<T=CheckBoxValue>, enabled: impl ReadState<T=bool>) -> Box<dyn AnyWidget>;
}

impl<K> PlainCheckBoxDelegate for K where K: Fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=CheckBoxValue>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> + Clone + 'static {
    fn call(&self, item: impl ReadState<T=Focus>, index: impl ReadState<T=CheckBoxValue>, enabled: impl ReadState<T=bool>) -> Box<dyn AnyWidget> {
        self(item.as_dyn_read(), index.as_dyn_read(), enabled.as_dyn_read())
    }
}

#[derive(Clone, Widget)]
#[carbide_exclude(Accessibility, AccessibilityEvent)]
pub struct PlainCheckBox<F, C, D, E> where
    F: State<T=Focus>,
    C: State<T=CheckBoxValue>,
    D: PlainCheckBoxDelegate,
    E: ReadState<T=bool>,
{
    id: WidgetId,
    #[state] focus: F,
    #[state] enabled: E,
    child: Box<dyn AnyWidget>,
    position: Position,
    dimension: Dimension,
    delegate: D,
    #[state] checked: C,
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

        ZStack::new(vec![
            Rectangle::new().fill(background_color).boxed(),
            Text::new(val).boxed(),
        ]).boxed()
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
        let delegate_widget = delegate.call(focus.as_dyn_read(), checked.as_dyn_read(), enabled.as_dyn_read());

        let button = MouseArea::new(delegate_widget)
            .custom_on_click(CheckboxAction {
                checked: checked.clone(),
                focus: focus.clone(),
                enabled: enabled.clone(),
            }).custom_on_click_outside(CheckBoxOutsideAction {
                focus: focus.clone()
            })
            .focused(focus.clone());

        let button = Box::new(button);

        let child = button;

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

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> AccessibilityEventHandler for PlainCheckBox<F, C, D, E> {
    fn handle_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        match event.action {
            AccessibilityAction::Click => {
                self.enabled.sync(ctx.env);
                self.focus.sync(ctx.env);

                if !*self.enabled.value() {
                    return;
                }

                if *self.checked.value() == CheckBoxValue::True {
                    *self.checked.value_mut() = CheckBoxValue::False;
                } else {
                    *self.checked.value_mut() = CheckBoxValue::True;
                }

                if *self.focus.value() != Focus::Focused {
                    *self.focus.value_mut() = Focus::FocusRequested;
                    ctx.env.request_focus(Refocus::FocusRequest);
                }
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

        let mut builder = NodeBuilder::new(Role::CheckBox);

        builder.set_bounds(Rect::from_origin_size(
            Point::new(self.x() * ctx.env.scale_factor(), self.y() * ctx.env.scale_factor()),
            Size::new(self.width() * ctx.env.scale_factor(), self.height() * ctx.env.scale_factor()),
        ));

        if ctx.hidden {
            builder.set_hidden();
        }

        if self.is_focusable() {
            builder.add_action(accessibility::Action::Focus);
        }

        if self.get_focus() == Focus::Focused {
            builder.add_action(accessibility::Action::Blur);
        }

        if let Some(label) = ctx.inherited_label {
            builder.set_name(label);
        } else {
            let labels = nodes.iter().filter_map(|x| x.name()).collect::<Vec<_>>();

            builder.set_name(labels.join(", "));
        }

        if let Some(hint) = ctx.inherited_hint {
            builder.set_description(hint);
        }

        if let Some(value) = ctx.inherited_value {
            builder.set_value(value);
        } else {
            match &*self.checked.value() {
                CheckBoxValue::True => {
                    builder.set_toggled(Toggled::True);
                }
                CheckBoxValue::Mixed => {
                    // The toggled mixed reads the checkbox as ticked on MacOS which is not the
                    // same as how swiftui reads the checkbox state.
                    builder.set_value("mixed");
                }
                CheckBoxValue::False => {
                    builder.set_toggled(Toggled::False);
                }
            }
        }

        if !*self.enabled.value() {
            builder.set_disabled();
        }

        builder.add_action(AccessibilityAction::Click);

        builder.set_author_id(format!("{:?}", self.id()));

        ctx.nodes.push(self.id(), builder.build());

        ctx.children.push(self.id());
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> CommonWidget for PlainCheckBox<F, C, D, E> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone, D: PlainCheckBoxDelegate, E: ReadState<T=bool>> Debug for PlainCheckBox<F, C, D, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainCheckBox")
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

impl<C: State<T=CheckBoxValue>, F: State<T=Focus>, E: ReadState<T=bool>> MouseAreaAction for CheckboxAction<C, F, E> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self.enabled.sync(ctx.env);
        self.focus.sync(ctx.env);

        if !*self.enabled.value() {
            return;
        }

        if *self.checked.value() == CheckBoxValue::True {
            *self.checked.value_mut() = CheckBoxValue::False;
        } else {
            *self.checked.value_mut() = CheckBoxValue::True;
        }

        if *self.focus.value() != Focus::Focused {
            *self.focus.value_mut() = Focus::FocusRequested;
            ctx.env.request_focus(Refocus::FocusRequest);
        }
    }
}

#[derive(Debug, Clone)]
struct CheckBoxOutsideAction<F> where
    F: State<T=Focus>,
{
    focus: F,
}

impl<F: State<T=Focus>> MouseAreaAction for CheckBoxOutsideAction<F> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self.focus.sync(ctx.env);
        if *self.focus.value() == Focus::Focused {
            self.focus.set_value(Focus::FocusReleased);
            ctx.env.request_focus(Refocus::FocusRequest);
        }
    }
}