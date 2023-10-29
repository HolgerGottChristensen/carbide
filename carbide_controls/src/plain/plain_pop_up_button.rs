use std::fmt::{Debug, Formatter};

use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::environment::EnvironmentColor;
use carbide_core::event::{
    Key, KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler,
};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Refocus};
use carbide_core::layout::{Layout};
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, ReadStateExtNew, State, StateContract, StateExtNew};
use carbide_core::widget::*;

use crate::plain::plain_pop_up_button_popup::PlainPopUpButtonPopUp;

#[derive(Clone, Widget)]
#[carbide_exclude(Layout, MouseEvent, KeyboardEvent)]
pub struct PlainPopUpButton<T, F, S, M, E>
where
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
{
    // Default fields
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] focus: F,
    #[state] enabled: E,

    // Delegates
    delegate: DelegateGenerator<T>, // Used to generate the control
    popup_delegate: PopupDelegateGenerator<T, S, M, LocalState<bool>>, // Used to generate the popup
    popup_item_delegate: PopupItemDelegateGenerator<T, S>, // Used to generate each item in the popup

    child: Box<dyn AnyWidget>,
    popup: Overlay<Box<dyn AnyWidget>, LocalState<bool>>,
    popup_open: LocalState<bool>,

    #[state] selected: S,
    #[state] model: M,
}

impl PlainPopUpButton<bool, Focus, bool, Vec<bool>, bool> {
    pub fn new<T: StateContract + PartialEq, S: IntoState<T>, M: IntoReadState<Vec<T>>>(
        selected: S,
        model: M,
    ) -> PlainPopUpButton<T, LocalState<Focus>, S::Output, M::Output, bool> {
        let focus = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            selected.into_state(),
            model.into_read_state(),
            focus,
            true,
            default_delegate,
            default_popup_delegate,
            default_popup_item_delegate,
        )
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> PlainPopUpButton<T, F, S, M, E> {
    pub fn enabled<E2: IntoReadState<bool>>(self, enabled: E2) -> PlainPopUpButton<T, F, S, M, E2::Output> {
        Self::new_internal(
            self.selected,
            self.model,
            self.focus,
            enabled.into_read_state(),
            self.delegate,
            self.popup_delegate,
            self.popup_item_delegate
        )
    }

    pub fn focused<F2: IntoState<Focus>>(self, focus: F2) -> PlainPopUpButton<T, F2::Output, S, M, E> {
        Self::new_internal(
            self.selected,
            self.model,
            focus.into_state(),
            self.enabled,
            self.delegate,
            self.popup_delegate,
            self.popup_item_delegate
        )
    }

    pub fn delegate(self, delegate: DelegateGenerator<T>) -> PlainPopUpButton<T, F, S, M, E> {
        Self::new_internal(
            self.selected,
            self.model,
            self.focus,
            self.enabled,
            delegate,
            self.popup_delegate,
            self.popup_item_delegate
        )
    }

    pub fn popup_delegate(self, popup_delegate: PopupDelegateGenerator<T, S, M, LocalState<bool>>) -> PlainPopUpButton<T, F, S, M, E> {
        Self::new_internal(
            self.selected,
            self.model,
            self.focus,
            self.enabled,
            self.delegate,
            popup_delegate,
            self.popup_item_delegate
        )
    }

    pub fn popup_item_delegate(self, popup_item_delegate: PopupItemDelegateGenerator<T, S>) -> PlainPopUpButton<T, F, S, M, E> {
        Self::new_internal(
            self.selected,
            self.model,
            self.focus,
            self.enabled,
            self.delegate,
            self.popup_delegate,
            popup_item_delegate
        )
    }

    fn new_internal<T2: StateContract + PartialEq, F2: State<T=Focus>, S2: State<T=T2>, M2: ReadState<T=Vec<T2>>, E2: ReadState<T=bool>>(
        selected: S2,
        model: M2,
        focus: F2,
        enabled: E2,
        delegate: DelegateGenerator<T2>,
        popup_delegate: PopupDelegateGenerator<T2, S2, M2, LocalState<bool>>,
        popup_item_delegate: PopupItemDelegateGenerator<T2, S2>,
    ) -> PlainPopUpButton<T2, F2, S2, M2, E2> {
        // Stores whether the popup is currently open or closed
        let popup_open = LocalState::new(false);

        let hover_model = LocalState::new(None);

        let del = PopupDelegate {
            hover_model: hover_model.clone(),
            selected_item: selected.clone(),
            popup_item_delegate,
            popup_open: popup_open.clone(),
            enabled: enabled.as_dyn_read(),
        };

        let popup_delegate_widget = popup_delegate(model.clone(), del, enabled.as_dyn_read());

        let popup = PlainPopUpButtonPopUp::new(
            popup_delegate_widget,
            hover_model,
            popup_open.as_dyn(),
            model.clone(),
            selected.clone(),
            enabled.as_dyn_read(),
        ).boxed().overlay("controls_popup_layer", popup_open.clone());

        let child = delegate(selected.as_dyn(), focus.as_dyn(), popup_open.as_dyn_read(), enabled.as_dyn_read());

        PlainPopUpButton {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            focus,
            enabled,

            delegate,
            popup_delegate,
            popup_item_delegate,

            child,
            popup,
            popup_open,

            selected,
            model,
        }
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> CommonWidget for PlainPopUpButton<T, F, S, M, E> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 1, focus: self.focus);
}


impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> Debug for PlainPopUpButton<T, F, S, M, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainPopUpButton")
            .field("child", &self.child)
            .finish()
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> WidgetExt for PlainPopUpButton<T, F, S, M, E> {}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> KeyboardEventHandler for PlainPopUpButton<T, F, S, M, E> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        if self.get_focus() != Focus::Focused || !*self.enabled.value() { return; }

        if event == PopupButtonKeyCommand::Open {
            self.popup_open.set_value(true);
            env.request_animation_frame();
        }
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> MouseEventHandler for PlainPopUpButton<T, F, S, M, E> {
    // Implementing this instead of handle_mouse_event makes all the children not receive events.
    fn process_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment) {
        if !env.is_event_current() { return }
        match event {
            MouseEvent::Click(_, position, _) => {
                if self.is_inside(*position) {
                    if !*self.enabled.value() {
                        return;
                    }

                    if self.get_focus() != Focus::Focused {
                        self.set_focus(Focus::FocusRequested);
                        env.request_focus(Refocus::FocusRequest);
                    }
                    self.popup_open.set_value(true);
                    env.request_animation_frame();
                } else {
                    if self.get_focus() == Focus::Focused {
                        self.set_focus(Focus::FocusReleased);
                        env.request_focus(Refocus::FocusRequest);
                    }
                }
            }
            _ => (),
        }
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> Layout for PlainPopUpButton<T, F, S, M, E> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let dimensions = self.child.calculate_size(requested_size, env);
        self.set_dimension(dimensions);

        let max_height = 400.0;
        let max_height = env.current_window_height().min(max_height);
        let popup_request = Dimension::new(dimensions.width, max_height);

        self.popup.calculate_size(popup_request, env);

        dimensions
    }

    fn position_children(&mut self, env: &mut Environment) {
        let positioning = self.alignment().positioner();
        let position = self.position();
        let dimension = self.dimension();

        positioning(position, dimension, &mut self.child);
        self.child.position_children(env);

        positioning(position, dimension, &mut self.popup);

        // Keep the Y within the screen
        let popup_position = self.popup.position();
        let popup_dimension = self.popup.dimension();
        let mut y = popup_position.y();

        if y + popup_dimension.height > env.current_window_height() {
            y = env.current_window_height() - popup_dimension.height;
        }
        if y < 0.0 {
            y = 0.0;
        }

        self.popup.set_position(Position::new(popup_position.x(), y));
        self.popup.position_children(env);
    }
}



// ---------------------------------------------------
//  Delegates
// ---------------------------------------------------
type DelegateGenerator<T> = fn(
    selected_item: Box<dyn AnyState<T=T>>,
    focused: Box<dyn AnyState<T=Focus>>,
    popup_open: Box<dyn AnyReadState<T=bool>>,
    enabled: Box<dyn AnyReadState<T=bool>>,
) -> Box<dyn AnyWidget>;

type PopupDelegateGenerator<T, S, M, B> = fn(
    model: M,
    delegate: PopupDelegate<T, S, B>,
    enabled: Box<dyn AnyReadState<T=bool>>,
) -> Box<dyn AnyWidget>;

type PopupItemDelegateGenerator<T, S> = fn(
    item: Box<dyn AnyState<T=T>>,
    index: Box<dyn AnyReadState<T=usize>>,
    hover: Box<dyn AnyReadState<T=bool>>,
    selected: S,
    enabled: Box<dyn AnyReadState<T=bool>>,
) -> Box<dyn AnyWidget>;

#[derive(Clone)]
pub struct PopupDelegate<T, S, B>
    where
        T: StateContract,
        S: State<T=T>,
        B: State<T=bool>,
{
    hover_model: LocalState<Option<usize>>,
    selected_item: S,
    popup_item_delegate: PopupItemDelegateGenerator<T, S>,
    popup_open: B,
    enabled: Box<dyn AnyReadState<T=bool>>,
}

impl<T: StateContract, S: State<T=T>, B: State<T=bool>> Delegate<T, Box<dyn AnyWidget>> for PopupDelegate<T, S, B> {
    fn call(&self, item: Box<dyn AnyState<T=T>>, index: Box<dyn AnyState<T=usize>>) -> Box<dyn AnyWidget> {
        let selected_item_del = self.selected_item.clone();
        let popup_open = self.popup_open.clone();
        let enabled = self.enabled.clone();

        // Map the hovered index to a boolean state telling us whether this item is hovered.
        // If we set this state to be hovered, we set the index, and otherwise we set to None.
        let hover_state = Map2::map(
            self.hover_model.clone(),
            index.clone(),
            |a, b| {
                if let Some(a) = a {
                    *a == *b
                } else {
                    false
                }
            },
            |new, _s1, s2| {
                if new {
                    (Some(Some(*s2)), None)
                } else {
                    (Some(None), None)
                }
            }
        ).as_dyn();

        let popup_item_delegate = (self.popup_item_delegate)(
            item.clone(),
            index.as_dyn_read(),
            hover_state.as_dyn_read(),
            selected_item_del.clone(),
            enabled.as_dyn_read(),
        );

        popup_item_delegate
            .on_click(move |_env: &mut Environment, _| {
                selected_item_del.clone().set_value(item.value().clone());
                popup_open.clone().set_value(false);
            })
            .hovered(hover_state)
            .boxed()
    }
}

fn default_delegate<T: StateContract + PartialEq>(
    selected_item: Box<dyn AnyState<T=T>>,
    focused: Box<dyn AnyState<T=Focus>>,
    _popup_open: Box<dyn AnyReadState<T=bool>>,
    _enabled: Box<dyn AnyReadState<T=bool>>,
) -> Box<dyn AnyWidget> {
    let background_color = Map1::read_map(focused.clone(), |focused| {
        match *focused {
            Focus::Focused => EnvironmentColor::Green,
            _ => EnvironmentColor::Blue,
        }
    });

    ZStack::new(vec![
        Rectangle::new().fill(background_color).boxed(),
        Text::new(selected_item.map(|a| format!("{:?}", a))).boxed(),
    ]).boxed()
}

fn default_popup_item_delegate<T: StateContract + PartialEq, S: State<T=T>>(
    item: Box<dyn AnyState<T=T>>,
    _index: Box<dyn AnyReadState<T=usize>>,
    hover_state: Box<dyn AnyReadState<T=bool>>,
    _selected_state: S,
    _enabled: Box<dyn AnyReadState<T=bool>>,
) -> Box<dyn AnyWidget> {
    let item_color = Map1::read_map(hover_state.clone(), |hovered| {
        if *hovered {
            EnvironmentColor::Pink
        } else {
            EnvironmentColor::Gray
        }
    });

    ZStack::new(vec![
        Rectangle::new().fill(item_color).boxed(),
        Text::new(item.map(|a: &T| format!("{:?}", *a)).ignore_writes()).boxed(),
    ])
        .frame_fixed_height(30.0)
        .boxed()
}

fn default_popup_delegate<T: StateContract + PartialEq, S: State<T=T>, M: ReadState<T=Vec<T>>, B: State<T=bool>>(
    model: M,
    delegate: PopupDelegate<T, S, B>,
    _enabled: Box<dyn AnyReadState<T=bool>>,
) -> Box<dyn AnyWidget> {
    VStack::new(ForEach::new(model.ignore_writes(), delegate))
        .spacing(1.0)
        .padding(1.0)
        .background(Rectangle::new().fill(EnvironmentColor::Yellow))
        .boxed()
}

// ---------------------------------------------------
//  Key commands
// ---------------------------------------------------
pub(super) enum PopupButtonKeyCommand {
    Next,
    Prev,
    Select,
    Close,
    Open,
}

impl PartialEq<PopupButtonKeyCommand> for &KeyboardEvent {
    fn eq(&self, other: &PopupButtonKeyCommand) -> bool {
        match other {
            PopupButtonKeyCommand::Next => {
                matches!(self, KeyboardEvent::Press(Key::Down, _))
            }
            PopupButtonKeyCommand::Prev => {
                matches!(self, KeyboardEvent::Press(Key::Up, _))
            }
            PopupButtonKeyCommand::Select => {
                matches!(self, KeyboardEvent::Press(Key::Return, _) | KeyboardEvent::Press(Key::Return2, _))
            }
            PopupButtonKeyCommand::Close => {
                matches!(self, KeyboardEvent::Press(Key::Escape, _))
            }
            PopupButtonKeyCommand::Open => {
                matches!(self, KeyboardEvent::Press(Key::Space, _) | KeyboardEvent::Press(Key::Return, _) | KeyboardEvent::Press(Key::Return2, _))
            }
        }
    }
}