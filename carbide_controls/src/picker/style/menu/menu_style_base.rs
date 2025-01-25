use crate::picker::style::menu::key_command::PopupButtonKeyCommand;
use carbide::event::{EventId, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseButton, MouseEvent, MouseEventContext, MouseEventHandler};
use carbide::focus::{Focus, FocusManager, Refocus};
use carbide::state::{IntoReadState, IntoState, ReadState, State, StateSync};
use carbide::widget::{CommonWidget, Empty, IntoWidget, OverlayManager, Widget, WidgetId};
use carbide::{CommonWidgetImpl, ModifierWidgetImpl};
use std::fmt::{Debug, Formatter};
use carbide::color::RED;
use carbide::draw::{Color, Dimension, Position};
use carbide::environment::{EnvironmentColor, EnvironmentStack, IntoColorReadState};
use carbide_core::misc::flags::WidgetFlag;
use crate::ControlsOverlayKey;

#[derive(Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent)]
pub struct MenuStyleBase<C, F, E, O, W>
where
    C: Widget,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
    O: Fn(EventId, Color) -> W + Clone + 'static,
    W: Widget
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    child: C,
    focus: F,
    enabled: E,
    open: O
}

impl MenuStyleBase<Empty, Focus, bool, fn(EventId, Color) ->Empty, Empty> {
    pub fn new<C: IntoWidget, F: IntoState<Focus>, E: IntoReadState<bool>, O: Fn(EventId, Color) -> W + Clone + 'static, W: Widget>(
        child: C,
        focus: F,
        enabled: E,
        open: O
    ) -> MenuStyleBase<C::Output, F::Output, E::Output, O, W> {
        MenuStyleBase {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: child.into_widget(),
            focus: focus.into_state(),
            enabled: enabled.into_read_state(),
            open,
        }
    }
}


/*impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> PlainPopUpButton<T, F, S, M, E> {
    fn open_popup(&self, event_id: EventId, env: &mut EnvironmentStack) {
        let selected = self.selected.value();
        let hover_model = LocalState::new(self.model.value().iter().position(|x| x == &*selected));

        let del = PopupItemDelegate {
            hover_model: hover_model.clone(),
            selected_item: self.selected.clone(),
            popup_item_delegate: self.popup_item_delegate,
            text_delegate: self.text_delegate,
            enabled: self.enabled.as_dyn_read(),
            overlay_id: Some("controls_popup_layer".to_string()),
            event_id,
        };

        let popup_delegate_widget = (self.popup_delegate)(self.model.clone(), del, self.enabled.as_dyn_read());

        let popup = crate::plain::plain_pop_up_button_popup::PlainPopUpButtonPopUp::new(
            popup_delegate_widget,
            hover_model,
            self.model.clone(),
            self.selected.clone(),
            self.enabled.as_dyn_read(),
            Some("controls_popup_layer".to_string()),
            self.position.clone(),
            self.dimension.clone(),
            event_id
        ).boxed();

        OverlayManager::get::<ControlsOverlayKey>(env, |manager| {
            manager.insert(popup)
        })
    }
}*/

impl<
    C: Widget,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
    O: Fn(EventId, Color) -> W + Clone + 'static,
    W: Widget
> CommonWidget for MenuStyleBase<C, F, E, O, W> {
    CommonWidgetImpl!(self, position: self.position, dimension: self.dimension, child: self.child, flag: WidgetFlag::FOCUSABLE, focus: self.focus);
}

impl<
    C: Widget,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
    O: Fn(EventId, Color) -> W + Clone + 'static,
    W: Widget
> KeyboardEventHandler for MenuStyleBase<C, F, E, O, W> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if self.get_focus() != Focus::Focused || !*self.enabled.value() { return; }

        if event == PopupButtonKeyCommand::Open {
            let mut accent = EnvironmentColor::Accent.color();
            accent.sync(ctx.env_stack);

            OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                let popup = (self.open)(EventId::default(), *accent.value());
                manager.insert(popup);
            });
        }
    }
}

impl<
    C: Widget,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
    O: Fn(EventId, Color) -> W + Clone + 'static,
    W: Widget
> MouseEventHandler for MenuStyleBase<C, F, E, O, W> {
    // Implementing this instead of handle_mouse_event makes all the children not receive events.
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !*ctx.is_current { return }
        match event {
            MouseEvent::Press { position, id, button: MouseButton::Left, .. } => {
                if self.is_inside(*position) {
                    if !*self.enabled.value() {
                        return;
                    }

                    if self.get_focus() != Focus::Focused {
                        self.set_focus(Focus::FocusRequested);
                        FocusManager::get(ctx.env_stack, |manager| {
                            manager.request_focus(Refocus::FocusRequest)
                        });
                    }

                    let mut accent = EnvironmentColor::Accent.color();
                    accent.sync(ctx.env_stack);

                    OverlayManager::get::<ControlsOverlayKey>(ctx.env_stack, |manager| {
                        let popup = (self.open)(*id, *accent.value());
                        manager.insert(popup);
                    });
                } else {
                    if self.get_focus() == Focus::Focused {
                        self.set_focus(Focus::FocusReleased);
                        FocusManager::get(ctx.env_stack, |manager| {
                            manager.request_focus(Refocus::FocusRequest)
                        });
                    }
                }
            }
            _ => (),
        }
    }
}

impl<
    C: Widget,
    F: State<T=Focus>,
    E: ReadState<T=bool>,
    O: Fn(EventId, Color) -> W + Clone + 'static,
    W: Widget
> Debug for MenuStyleBase<C, F, E, O, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuStyleBaseComponent")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("child", &self.child)
            .field("focus", &self.focus)
            .field("enabled", &self.enabled)
            .finish()
    }
}