use std::any::TypeId;
use carbide::accessibility::{Accessibility, AccessibilityContext};
use carbide::color::RED;
use carbide_core::cursor::MouseCursor;
use carbide::draw::{AutomaticStyle, Dimension, Position};
use carbide_core::flags::WidgetFlag;
use carbide::focus::{FocusManager, Refocus};
use carbide::state::{AnyReadState, AnyState, IntoState, ReadStateExt, State, StateExtNew};
use carbide::CommonWidgetImpl;
use carbide_core::focus::Focus;
use carbide_core::state::{LocalState, ReadState};
use carbide_core::widget::*;
use std::fmt::{Debug, Formatter};
use carbide::environment::Environment;
use carbide::widget::properties::WidgetKindSimple;
use crate::button::style::ButtonStyleKey;
use crate::{EnabledState};

#[derive(Clone, Widget)]
#[carbide_exclude(Accessibility, Sync)]
pub struct Button<A, L> where
    A: Action + Clone + 'static,
    L: Widget + WidgetProperties<Kind=WidgetKindSimple>,
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Box<dyn AnyWidget>,
    style_id: TypeId,

    action: A,
    label: L,

    #[state] focused: Box<dyn AnyState<T=Focus>>,
    #[state] enabled: Box<dyn AnyReadState<T=bool>>,
    #[state] hovered: Box<dyn AnyState<T=bool>>,
    #[state] pressed: Box<dyn AnyState<T=bool>>,

    cursor: MouseCursor,
}

impl Button<fn(MouseAreaActionContext), Empty> {
    pub fn new<L: IntoWidget, A: Action + Clone + 'static>(label: L, action: A) -> Button<A, L::Output> where L::Output: WidgetProperties<Kind=WidgetKindSimple> {
        Button {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: Box::new(Rectangle::new().fill(RED)),
            style_id: TypeId::of::<()>(),
            action,
            label: label.into_widget(),
            focused: LocalState::new(Focus::Unfocused).as_dyn(),
            enabled: EnabledState::new(true).as_dyn_read(),
            hovered: LocalState::new(false).as_dyn(),
            pressed: LocalState::new(false).as_dyn(),
            cursor: MouseCursor::Pointer,
        }
    }
}

impl<A: Action + Clone + 'static, L: Widget + WidgetProperties<Kind=WidgetKindSimple>> Button<A, L> {
    pub fn hovered<H2: IntoState<bool>>(mut self, hovered: H2) -> Button<A, L> {
        self.hovered = hovered.into_dyn_state();
        self
    }

    pub fn pressed<P2: IntoState<bool>>(mut self, pressed: P2) -> Button<A, L> {
        self.pressed = pressed.into_dyn_state();
        self
    }

    pub fn cursor(mut self, cursor: MouseCursor) -> Button<A, L> {
        self.cursor = cursor;
        self
    }
}

impl<A: Action + Clone + 'static, L: Widget + WidgetProperties<Kind=WidgetKindSimple>> WidgetSync for Button<A, L> {
    fn sync(&mut self, env: &mut Environment) {
        self.focused.sync(env);
        self.enabled.sync(env);
        self.hovered.sync(env);
        self.pressed.sync(env);

        let style = env.get::<ButtonStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle);

        if style.key() != self.style_id {
            self.style_id = style.key();

            let inner = style.create(
                self.label.clone().boxed(),
                self.focused.clone().as_dyn_read(),
                self.enabled.clone().as_dyn_read(),
                self.hovered.clone().as_dyn_read(),
                self.pressed.clone().as_dyn_read()
            );

            self.child = MouseArea::new(inner)
                .custom_on_click(ButtonAction {
                    action: self.action.clone(),
                    focus: self.focused.clone(),
                    enabled: self.enabled.clone(),
                })
                .custom_on_click_outside(ButtonOutsideAction {
                    action: self.action.clone(),
                    focus: self.focused.clone(),
                    enabled: self.enabled.clone(),
                })
                .focused(self.focused.clone())
                .pressed(self.pressed.clone())
                .hovered(self.hovered.clone())
                .hover_cursor(self.cursor)
                .boxed();
        }
    }
}
impl<A: Action + Clone + 'static, L: Widget + WidgetProperties<Kind=WidgetKindSimple>> CommonWidget for Button<A, L> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focused);
}

impl<A: Action + Clone + 'static, L: Widget + WidgetProperties<Kind=WidgetKindSimple>> Accessibility for Button<A, L> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env);

        let enabled = *self.enabled.value();

        self.child.process_accessibility(&mut AccessibilityContext {
            env: ctx.env,
            nodes: ctx.nodes,
            parent_id: ctx.parent_id,
            children: ctx.children,
            hidden: ctx.hidden,
            inherited_label: ctx.inherited_label,
            inherited_hint: ctx.inherited_hint,
            inherited_value: ctx.inherited_value,
            inherited_enabled: Some(enabled),
        })
    }
}

impl<A: Action + Clone + 'static, L: Widget + WidgetProperties<Kind=WidgetKindSimple>> Debug for Button<A, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focused)
            .field("child", &self.child)
            .field("enabled", &self.enabled)
            .finish()
    }
}

#[derive(Debug, Clone)]
struct ButtonAction<A> where
    A: Action
{
    action: A,
    focus: Box<dyn AnyState<T=Focus>>,
    enabled: Box<dyn AnyReadState<T=bool>>,
}

impl<A: Action + Clone + 'static> MouseAreaAction for ButtonAction<A> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self.enabled.sync(ctx.env);
        self.focus.sync(ctx.env);

        if *self.enabled.value() {
            if *self.focus.value() != Focus::Focused {
                self.focus.set_value(Focus::FocusRequested);

                FocusManager::get(ctx.env, |manager| {
                    manager.request_focus(Refocus::FocusRequest)
                });
            }
            (self.action)(ctx);
        }
    }
}

#[derive(Debug, Clone)]
struct ButtonOutsideAction<A> where
    A: Action
{
    action: A,
    focus: Box<dyn AnyState<T=Focus>>,
    enabled: Box<dyn AnyReadState<T=bool>>,
}

impl<A: Action + Clone + 'static> MouseAreaAction for ButtonOutsideAction<A> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self.focus.sync(ctx.env);
        if *self.focus.value() == Focus::Focused {
            self.focus.set_value(Focus::FocusReleased);

            FocusManager::get(ctx.env, |manager| {
                manager.request_focus(Refocus::FocusRequest)
            });
        }
    }
}