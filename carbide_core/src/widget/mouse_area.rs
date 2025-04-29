use std::fmt::{Debug, Formatter};
use accesskit::{Node, Point, Rect, Role, Size};
use smallvec::SmallVec;
use crate::accessibility;
use crate::accessibility::{AccessibilityContext, AccessibilityNode};
use crate::environment::Environment;
use crate::event::{AccessibilityEvent, AccessibilityEventContext, OtherEvent, OtherEventContext};
use crate::scene::SceneManager;
use crate::widget::WidgetSync;
use carbide_macro::carbide_default_builder2;
use crate::accessibility::{Accessibility, AccessibilityAction};
use crate::CommonWidgetImpl;
use crate::misc::cursor::MouseCursor;
use crate::draw::{Dimension, Position};
use crate::event::{Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseButton, MouseEvent, MouseEventHandler, KeyboardEventContext, MouseEventContext, AccessibilityEventHandler, OtherEventHandler};
use crate::misc::flags::WidgetFlag;
use crate::focus::{Focus, Focusable};
use crate::state::{IntoState, State};
use crate::widget::{CommonWidget, Widget, WidgetId, Empty, Identifiable};
use crate::widget::managers::{ShortcutPressed, ShortcutReleased};

#[derive(Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent, OtherEvent, Accessibility, AccessibilityEvent)]
pub struct MouseArea<I, O, F, C, H, P> where
    I: MouseAreaAction,
    O: MouseAreaAction,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
{
    #[id] id: WidgetId,
    #[state] focus: F,
    child: C,
    position: Position,
    dimension: Dimension,
    click: I,
    click_outside: O,
    #[state] is_hovered: H,
    #[state] is_pressed: P,
    hover_cursor: MouseCursor,
    pressed_cursor: Option<MouseCursor>,
}

impl MouseArea<fn(MouseAreaActionContext), fn(MouseAreaActionContext), Focus, Empty, bool, bool> {

    #[carbide_default_builder2]
    pub fn new<C: Widget>(child: C) -> MouseArea<fn(MouseAreaActionContext), fn(MouseAreaActionContext), Focus, C, bool, bool> {
        MouseArea {
            id: WidgetId::new(),
            focus: Focus::Unfocused,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            click: |_| {},
            click_outside: |_| {},
            is_hovered: false,
            is_pressed: false,
            hover_cursor: MouseCursor::Pointer,
            pressed_cursor: None,
        }
    }
}

impl<
    I: MouseAreaAction + Clone + 'static,
    O: MouseAreaAction + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> MouseArea<I, O, F, C, H, P> {
    /// Example: .on_click(move |ctx: MouseAreaActionContext| {})
    pub fn on_click<A: Action>(self, action: A) -> MouseArea<A, O, F, C, H, P> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: action,
            click_outside: self.click_outside,
            is_hovered: self.is_hovered,
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn on_click_outside<A: Action>(self, action: A) -> MouseArea<I, A, F, C, H, P> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: self.click,
            click_outside: action,
            is_hovered: self.is_hovered,
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn custom_on_click<A: MouseAreaAction>(self, action: A) -> MouseArea<A, O, F, C, H, P> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: action,
            click_outside: self.click_outside,
            is_hovered: self.is_hovered,
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn custom_on_click_outside<A: MouseAreaAction>(self, action: A) -> MouseArea<I, A, F, C, H, P> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: self.click,
            click_outside: action,
            is_hovered: self.is_hovered,
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn hovered<T: IntoState<bool>>(self, is_hovered: T) -> MouseArea<I, O, F, C, T::Output, P> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: self.click,
            click_outside: self.click_outside,
            is_hovered: is_hovered.into_state(),
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn pressed<T: IntoState<bool>>(self, pressed: T) -> MouseArea<I, O, F, C, H, T::Output> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: self.click,
            click_outside: self.click_outside,
            is_hovered: self.is_hovered,
            is_pressed: pressed.into_state(),
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn focused<T: IntoState<Focus>>(self, focused: T) -> MouseArea<I, O, T::Output, C, H, P> {
        MouseArea {
            id: self.id,
            focus: focused.into_state(),
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: self.click,
            click_outside: self.click_outside,
            is_hovered: self.is_hovered,
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn hover_cursor(mut self, cursor: MouseCursor) -> Self {
        self.hover_cursor = cursor;
        self
    }

    pub fn pressed_cursor(mut self, cursor: MouseCursor) -> Self {
        self.pressed_cursor = Some(cursor);
        self
    }
}

impl<
    I: MouseAreaAction + Clone + 'static,
    O: MouseAreaAction + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> KeyboardEventHandler for MouseArea<I, O, F, C, H, P> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if self.get_focus() != Focus::Focused {
            return;
        }

        match event {
            KeyboardEvent::Press { key: Key::Enter, .. } => {
                self.is_pressed.set_value(true);
            }
            KeyboardEvent::Release { key: Key::Enter, .. } => {
                if *self.is_pressed.value() {
                    self.is_pressed.set_value(false);
                    self.click.call(MouseAreaActionContext {
                        env: ctx.env,
                        modifier_key: ModifierKey::empty()
                    });
                } else {
                    self.is_pressed.set_value(false);
                }
            }
            _ => (),
        }
    }
}

impl<
    I: MouseAreaAction + Clone + 'static,
    O: MouseAreaAction + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> OtherEventHandler for MouseArea<I, O, F, C, H, P> {
    fn handle_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {
        if *ctx.is_current && !*ctx.is_consumed {
            if event.is::<ShortcutPressed>() {
                self.is_pressed.set_value(true);
                *ctx.is_consumed = true;
            }
            if event.is::<ShortcutReleased>() {
                self.is_pressed.set_value(false);
                self.click.call(MouseAreaActionContext {
                    env: ctx.env,
                    modifier_key: ModifierKey::empty()
                });
                *ctx.is_consumed = true;
            }
        }
    }
}

impl<
    I: MouseAreaAction + Clone + 'static,
    O: MouseAreaAction + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> MouseEventHandler for MouseArea<I, O, F, C, H, P> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Press { button: MouseButton::Left, position: mouse_position, .. } => {
                if self.is_inside(*mouse_position) {
                    self.is_pressed.set_value(true);
                }
            }
            MouseEvent::Release { button: MouseButton::Left, position: mouse_position, .. } => {
                if self.is_inside(*mouse_position) {
                    self.is_pressed.set_value(false);
                }
            }
            MouseEvent::Move { to, .. } => {
                if *self.is_hovered.value() {
                    if !self.is_inside(*to) {
                        self.is_pressed.set_value(false);
                        self.is_hovered.set_value(false);
                    }
                } else {
                    if self.is_inside(*to) {
                        self.is_hovered.set_value(true);
                    }
                }
            }
            MouseEvent::Click(MouseButton::Left, mouse_position, modifier)
            | MouseEvent::NClick(MouseButton::Left, mouse_position, modifier, _) => {
                if self.is_inside(*mouse_position) {
                    //self.request_focus(ctx.env);
                    self.click.call(MouseAreaActionContext {
                        env: ctx.env,
                        modifier_key: *modifier
                    });
                } else {
                    //self.set_focus(Focus::Unfocused);
                    self.click_outside.call(MouseAreaActionContext {
                        env: ctx.env,
                        modifier_key: *modifier
                    });
                }
            }
            _ => (),
        }
    }
}

impl<
    I: MouseAreaAction + Clone + 'static,
    O: MouseAreaAction + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> CommonWidget for MouseArea<I, O, F, C, H, P> {
    CommonWidgetImpl!(self, child: self.child, flag: WidgetFlag::FOCUSABLE, focus: self.focus, position: self.position, dimension: self.dimension);

    fn cursor(&self) -> Option<MouseCursor> {
        if *self.is_hovered.value() {
            return Some(self.hover_cursor)
        }

        if *self.is_pressed.value() {
            return self.pressed_cursor;
        }

        None
    }
}

impl<
    I: MouseAreaAction + Clone + 'static,
    O: MouseAreaAction + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> AccessibilityEventHandler for MouseArea<I, O, F, C, H, P> {
    fn handle_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        match event.action {
            AccessibilityAction::Click => {
                self.click.call(MouseAreaActionContext {
                    env: ctx.env,
                    modifier_key: ModifierKey::empty()
                });
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

impl<
    I: MouseAreaAction + Clone + 'static,
    O: MouseAreaAction + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> Accessibility for MouseArea<I, O, F, C, H, P> {
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

        let mut builder = Node::new(Role::Button);

        let scale_factor = ctx.env.get_mut::<SceneManager>()
            .map(|a| a.scale_factor())
            .unwrap_or(1.0);

        builder.set_bounds(Rect::from_origin_size(
            Point::new(self.x() * scale_factor, self.y() * scale_factor),
            Size::new(self.width() * scale_factor, self.height() * scale_factor),
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
            builder.set_label(label);
        } else {
            let labels = nodes.iter().filter_map(|x| x.label()).collect::<Vec<_>>();

            builder.set_label(labels.join(", "));
        }

        if let Some(hint) = ctx.inherited_hint {
            builder.set_description(hint);
        }

        if let Some(value) = ctx.inherited_value {
            builder.set_value(value);
        }

        if let Some(enabled) = ctx.inherited_enabled {
            if !enabled {
                builder.set_disabled();
            }
        }

        builder.add_action(AccessibilityAction::Click);

        builder.set_author_id(format!("{:?}", self.id()));

        ctx.nodes.push(self.id(), builder);

        ctx.children.push(self.id());
    }
}

impl<
    I: MouseAreaAction + Clone + 'static,
    O: MouseAreaAction + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> Debug for MouseArea<I, O, F, C, H, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MouseArea")
            .field("child", &self.child)
            .finish()
    }
}

// ==============================================
// Utility structs
// ==============================================

// When using this struct as the input for functions, the closures can not be inferred.
pub trait MouseAreaAction: Clone + 'static {
    fn call(&mut self, ctx: MouseAreaActionContext);
}

impl<I> MouseAreaAction for I where I: Fn(MouseAreaActionContext<'_, '_>) + Clone + 'static {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self(ctx);
    }
}

// Since this is a super trait of Fn, closures taking this type can be have
// their parameters inferred by rust.
pub trait Action: Fn(MouseAreaActionContext) + Clone + 'static {}

impl<I> Action for I where I: Fn(MouseAreaActionContext<'_, '_>) + Clone + 'static {}

/// The context given when handling on click actions.
pub struct MouseAreaActionContext<'a, 'b: 'a> {
    pub env: &'a mut Environment<'b>,
    pub modifier_key: ModifierKey
}