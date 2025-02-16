use carbide::accessibility::{Accessibility, AccessibilityContext};
use carbide::color::RED;
use carbide_core::cursor::MouseCursor;
use carbide::draw::{AutomaticStyle, Dimension, Position};
use carbide_core::flags::WidgetFlag;
use carbide::focus::{FocusManager, Refocus};
use carbide::lifecycle::{InitializationContext, Initialize};
use carbide::state::{IntoState, ReadStateExtNew, State};
use carbide::CommonWidgetImpl;
use carbide_core::environment::IntoColorReadState;
use carbide_core::focus::Focus;
use carbide_core::state::{LocalState, ReadState};
use carbide_core::widget::*;
use std::fmt::{Debug, Formatter};

use crate::button::style::ButtonStyleKey;
use crate::{EnabledState};

#[derive(Clone, Widget)]
#[carbide_exclude(Accessibility, Initialize)]
pub struct Button<F, A, E, H, P, L> where
    F: State<T=Focus>,
    A: Action + Clone + 'static,
    E: ReadState<T=bool>,
    H: State<T=bool>,
    P: State<T=bool>,
    L: Widget,
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Box<dyn AnyWidget>,
    action: A,
    label: L,

    #[state] focus: F,
    #[state] enabled: E,
    #[state] hovered: H,
    #[state] pressed: P,

    cursor: MouseCursor,
}

impl Button<LocalState<Focus>, fn(MouseAreaActionContext), bool, LocalState<bool>, LocalState<bool>, Empty> {
    pub fn new<L: IntoWidget, A: Action + Clone + 'static>(label: L, action: A) -> Button<LocalState<Focus>, A, impl ReadState<T=bool>, LocalState<bool>, LocalState<bool>, L::Output> {
        Button {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: Box::new(Rectangle::new().fill(RED)),
            action,
            label: label.into_widget(),
            focus: LocalState::new(Focus::Unfocused),
            enabled: EnabledState::new(true),
            hovered: LocalState::new(false),
            pressed: LocalState::new(false),
            cursor: MouseCursor::Pointer,
        }
    }
}

impl<F: State<T=Focus>, A: Action + Clone + 'static, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> Button<F, A, E, H, P, L> {
    pub fn hovered<H2: IntoState<bool>>(self, hovered: H2) -> Button<F, A, E, H2::Output, P, L> {
        Button {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            child: self.child,
            action: self.action,
            label: self.label,
            focus: self.focus,
            enabled: self.enabled,
            hovered: hovered.into_state(),
            pressed: self.pressed,
            cursor: self.cursor,
        }
    }

    pub fn pressed<P2: IntoState<bool>>(self, pressed: P2) -> Button<F, A, E, H, P2::Output, L> {
        Button {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            child: self.child,
            action: self.action,
            label: self.label,
            focus: self.focus,
            enabled: self.enabled,
            hovered: self.hovered,
            pressed: pressed.into_state(),
            cursor: self.cursor,
        }
    }

    pub fn cursor(mut self, cursor: MouseCursor) -> Button<F, A, E, H, P, L> {
        self.cursor = cursor;
        self
    }
}

impl<F: State<T=Focus>, A: Action + Clone + 'static, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> CommonWidget for Button<F, A, E, H, P, L> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus>, A: Action + Clone + 'static, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> Accessibility for Button<F, A, E, H, P, L> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.enabled.sync(ctx.env);
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

impl<F: State<T=Focus>, A: Action + Clone + 'static, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> Initialize for Button<F, A, E, H, P, L> {
    fn initialize(&mut self, ctx: &mut InitializationContext) {
        let style = ctx.env.get::<ButtonStyleKey>().map(|a | &**a).unwrap_or(&AutomaticStyle);

        let inner = style.create(self.label.clone().boxed(), self.focus.as_dyn_read(), self.enabled.as_dyn_read(), self.hovered.as_dyn_read(), self.pressed.as_dyn_read());

        self.child = MouseArea::new(inner)
            .custom_on_click(ButtonAction {
                action: self.action.clone(),
                focus: self.focus.clone(),
                enabled: self.enabled.clone(),
            })
            .custom_on_click_outside(ButtonOutsideAction {
                action: self.action.clone(),
                focus: self.focus.clone(),
                enabled: self.enabled.clone(),
            })
            .focused(self.focus.clone())
            .pressed(self.pressed.clone())
            .hovered(self.hovered.clone())
            .hover_cursor(self.cursor)
            .boxed();
    }
}

impl<F: State<T=Focus>, A: Action + Clone + 'static, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>, L: Widget> Debug for Button<F, A, E, H, P, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainButton")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focus)
            .field("child", &self.child)
            .finish()
    }
}

#[derive(Debug, Clone)]
struct ButtonAction<A, F, E> where
    A: Action,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
{
    action: A,
    focus: F,
    enabled: E,
}

impl<A: Action + Clone + 'static, F: State<T=Focus>, E: ReadState<T=bool>> MouseAreaAction for ButtonAction<A, F, E> {
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
struct ButtonOutsideAction<A, F, E> where
    A: Action,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
{
    action: A,
    focus: F,
    enabled: E,
}

impl<A: Action + Clone + 'static, F: State<T=Focus>, E: ReadState<T=bool>> MouseAreaAction for ButtonOutsideAction<A, F, E> {
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