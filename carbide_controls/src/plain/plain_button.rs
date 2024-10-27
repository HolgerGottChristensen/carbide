use crate::{enabled_state, EnabledState};
use carbide::accessibility::AccessibilityContext;
use carbide::color::ColorExt;
use carbide::environment::IntoColorReadState;
use carbide::focus::Focusable;
use carbide::widget::{Action, MouseAreaActionContext};
use carbide_core::accessibility::Accessibility;
use carbide_core::color::ORANGE;
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::EnvironmentColor;
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::{Focus, Refocus};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map1, Map4, ReadState, ReadStateExtNew, State};
use carbide_core::widget::{AnyWidget, CommonWidget, MouseArea, MouseAreaAction, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack};
use carbide_core::CommonWidgetImpl;
use std::fmt::{Debug, Formatter};

pub trait PlainButtonDelegate: Clone + 'static {
    type Output: Widget;
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Self::Output;
}

impl<K, C: Widget> PlainButtonDelegate for K where K: Fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> C + Clone + 'static {
    type Output = C;
    fn call(&self, focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> C {
        self(focus, hovered, pressed, enabled)
    }
}

type DefaultPlainButtonDelegate = fn(Box<dyn AnyReadState<T=Focus>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>, Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget>;
type DefaultPlainButtonAction = fn(MouseAreaActionContext);

#[derive(Clone, Widget)]
#[carbide_exclude(Accessibility)]
pub struct PlainButton<F, A, D, E, H, P> where
    F: State<T=Focus>,
    A: Action + Clone + 'static,
    D: PlainButtonDelegate,
    E: ReadState<T=bool>,
    H: State<T=bool>,
    P: State<T=bool>,
{
    id: WidgetId,
    #[state] focus: F,
    #[state] enabled: E,
    child: MouseArea<
        ButtonAction<A, F, E>,
        ButtonOutsideAction<A, F, E>,
        <F as IntoState<Focus>>::Output,
        D::Output,
        <H as IntoState<bool>>::Output,
        <P as IntoState<bool>>::Output
    >,
    position: Position,
    dimension: Dimension,
    delegate: D,
    action: A,
    #[state] hovered: H,
    #[state] pressed: P,
    cursor: MouseCursor,
}

impl PlainButton<Focus, DefaultPlainButtonAction, DefaultPlainButtonDelegate, bool, bool, bool> {
    pub fn new<A: Action + Clone + 'static>(action: A) -> PlainButton<LocalState<Focus>, A, DefaultPlainButtonDelegate, EnabledState, LocalState<bool>, LocalState<bool>> {

        let focus_state = LocalState::new(Focus::Unfocused);
        let hovered = LocalState::new(false);
        let pressed = LocalState::new(false);

        Self::new_internal(
            action,
            focus_state,
            PlainButton::default_delegate,
            enabled_state(),
            MouseCursor::Pointer,
            hovered,
            pressed
        )
    }

    fn default_delegate(focus: Box<dyn AnyReadState<T=Focus>>, hovered: Box<dyn AnyReadState<T=bool>>, pressed: Box<dyn AnyReadState<T=bool>>, enabled: Box<dyn AnyReadState<T=bool>>) -> Box<dyn AnyWidget> {
        let val = Map1::read_map(focus, |focused| {
            format!("{:?}", focused)
        });

        let background_color = Map4::read_map(EnvironmentColor::Accent.color(), pressed, hovered, enabled, |col, pressed, hovered, enabled| {
            if !*enabled {
                return ORANGE;
            }

            if *pressed {
                col.darkened(0.1)
            } else if *hovered {
                col.lightened(0.1)
            } else {
                *col
            }
        });

        ZStack::new(vec![
            Rectangle::new().fill(background_color).boxed(),
            Text::new(val).boxed(),
        ]).boxed()
    }
}

impl<F: State<T=Focus>, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>> PlainButton<F, A, D, E, H, P> {

    pub fn delegate<D2: PlainButtonDelegate>(
        self,
        delegate: D2,
    ) -> PlainButton<F, A, D2, E, H, P> {
        let action = self.action;
        let focus_state = self.focus;

        Self::new_internal(action, focus_state, delegate, self.enabled, self.cursor, self.hovered, self.pressed)
    }

    pub fn focused<F2: IntoState<Focus>>(self, focused: F2) -> PlainButton<F2::Output, A, D, E, H, P> {
        Self::new_internal(self.action, focused.into_state(), self.delegate, self.enabled, self.cursor, self.hovered, self.pressed)
    }

    pub fn pressed<P2: IntoState<bool>>(self, pressed: P2) -> PlainButton<F, A, D, E, H, P2::Output> {
        Self::new_internal(self.action, self.focus, self.delegate, self.enabled, self.cursor, self.hovered, pressed.into_state())
    }

    pub fn hovered<H2: IntoState<bool>>(self, hovered: H2) -> PlainButton<F, A, D, E, H2::Output, P> {
        Self::new_internal(self.action, self.focus, self.delegate, self.enabled, self.cursor, hovered.into_state(), self.pressed)
    }

    pub fn enabled<E2: IntoReadState<bool>>(self, enabled: E2) -> PlainButton<F, A, D, E2::Output, H, P> {
        Self::new_internal(self.action, self.focus, self.delegate, enabled.into_read_state(), self.cursor, self.hovered, self.pressed)
    }

    pub fn cursor(self, cursor: MouseCursor) -> PlainButton<F, A, D, E, H, P> {
        Self::new_internal(self.action, self.focus, self.delegate, self.enabled, cursor, self.hovered, self.pressed)
    }

    fn new_internal<
        F2: State<T=Focus>,
        A2: Action,
        D2: PlainButtonDelegate,
        E2: ReadState<T=bool>,
        H2: State<T=bool>,
        P2: State<T=bool>,
    >(
        action: A2,
        focus: F2,
        delegate: D2,
        enabled: E2,
        cursor: MouseCursor,
        hovered: H2,
        pressed: P2,
    ) -> PlainButton<F2, A2, D2, E2, H2, P2> {

        let delegate_widget = delegate.call(focus.as_dyn_read(), hovered.as_dyn_read(), pressed.as_dyn_read(), enabled.as_dyn_read());

        let area = MouseArea::new(delegate_widget)
            .custom_on_click(ButtonAction {
                action: action.clone(),
                focus: focus.clone(),
                enabled: enabled.clone(),
            })
            .custom_on_click_outside(ButtonOutsideAction {
                action: action.clone(),
                focus: focus.clone(),
                enabled: enabled.clone(),
            })
            .focused(focus.clone())
            .pressed(pressed.clone())
            .hovered(hovered.clone())
            .hover_cursor(cursor);


        PlainButton {
            id: WidgetId::new(),
            focus,
            child: area,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            delegate,
            action,
            hovered,
            enabled,
            cursor,
            pressed,
        }
    }
}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>> CommonWidget for PlainButton<F, A, D, E, H, P> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: WidgetFlag::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>> Accessibility for PlainButton<F, A, D, E, H, P> {
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

impl<F: State<T=Focus> + Clone, A: Action + Clone + 'static, D: PlainButtonDelegate, E: ReadState<T=bool>, H: State<T=bool>, P: State<T=bool>> Debug for PlainButton<F, A, D, E, H, P> {
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

        {
            if *self.enabled.value() {
                if *self.focus.value() != Focus::Focused {
                    self.focus.set_value(Focus::FocusRequested);
                    ctx.env.request_focus(Refocus::FocusRequest);
                }
                (self.action)(ctx);
            }
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
            ctx.env.request_focus(Refocus::FocusRequest);
        }
    }
}