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
use carbide_core::layout::{Layout, Layouter};
use carbide_core::render::Render;
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, ReadStateExtNew, State, StateContract, StateExtNew, TState};
use carbide_core::widget::*;

use crate::plain::plain_pop_up_button_popup::PlainPopUpButtonPopUp;

#[derive(Clone, Widget)]
#[carbide_exclude(Layout, MouseEvent, KeyboardEvent)]
pub struct PlainPopUpButton<T, F, S, M>
where
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
{
    // Default fields
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] focus: F,

    // Delegates
    delegate: DelegateGenerator<T>, // Used to generate the control
    popup_delegate: PopupDelegateGenerator<T, S, M, TState<bool>>, // Used to generate the popup
    popup_item_delegate: PopupItemDelegateGenerator<T, S>, // Used to generate each item in the popup

    child: Box<dyn Widget>,
    popup: Overlay<Box<dyn Widget>, TState<bool>>,
    popup_open: TState<bool>,

    #[state] selected: S,
    #[state] model: M,
}

impl PlainPopUpButton<bool, Focus, bool, Vec<bool>> {
    pub fn new<T: StateContract + PartialEq, S: IntoState<T>, M: IntoReadState<Vec<T>>>(
        selected: S,
        model: M,
    ) -> PlainPopUpButton<T, TState<Focus>, S::Output, M::Output> {
        let focus = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            selected.into_state(),
            model.into_read_state(),
            focus,
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
    M: ReadState<T=Vec<T>>
> PlainPopUpButton<T, F, S, M> {
    pub fn delegate(self, delegate: DelegateGenerator<T>) -> PlainPopUpButton<T, F, S, M> {
        Self::new_internal(
            self.selected,
            self.model,
            self.focus,
            delegate,
            self.popup_delegate,
            self.popup_item_delegate
        )
    }

    pub fn popup_delegate(self, popup_delegate: PopupDelegateGenerator<T, S, M, TState<bool>>) -> PlainPopUpButton<T, F, S, M> {
        Self::new_internal(
            self.selected,
            self.model,
            self.focus,
            self.delegate,
            popup_delegate,
            self.popup_item_delegate
        )
    }

    pub fn popup_item_delegate(self, popup_item_delegate: PopupItemDelegateGenerator<T, S>) -> PlainPopUpButton<T, F, S, M> {
        Self::new_internal(
            self.selected,
            self.model,
            self.focus,
            self.delegate,
            self.popup_delegate,
            popup_item_delegate
        )
    }

    fn new_internal<T2: StateContract + PartialEq, F2: State<T=Focus>, S2: State<T=T2>, M2: ReadState<T=Vec<T2>>>(
        selected: S2,
        model: M2,
        focus: F2,
        delegate: DelegateGenerator<T2>,
        popup_delegate: PopupDelegateGenerator<T2, S2, M2, TState<bool>>,
        popup_item_delegate: PopupItemDelegateGenerator<T2, S2>,
    ) -> PlainPopUpButton<T2, F2, S2, M2> {
        // Stores whether the popup is currently open or closed
        let popup_open = LocalState::new(false);

        let hover_model = LocalState::new(None);

        let del = PopupDelegate {
            hover_model: hover_model.clone(),
            selected_item: selected.clone(),
            popup_item_delegate,
            popup_open: popup_open.clone(),
        };

        let popup_delegate_widget = popup_delegate(model.clone(), del);

        let popup = PlainPopUpButtonPopUp::new(
            popup_delegate_widget,
            hover_model,
            popup_open.as_dyn(),
            model.clone(),
            selected.clone(),
        ).boxed().overlay("controls_popup_layer", popup_open.clone());

        let child = delegate(selected.as_dyn(), focus.as_dyn(), popup_open.as_dyn_read());

        PlainPopUpButton {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            focus,

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
    M: ReadState<T=Vec<T>>
> CommonWidget for PlainPopUpButton<T, F, S, M> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 10, focus: self.focus);
}


impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>
> Debug for PlainPopUpButton<T, F, S, M> {
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
    M: ReadState<T=Vec<T>>
> WidgetExt for PlainPopUpButton<T, F, S, M> {}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>
> KeyboardEventHandler for PlainPopUpButton<T, F, S, M> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        if self.get_focus() != Focus::Focused { return; }

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
    M: ReadState<T=Vec<T>>
> MouseEventHandler for PlainPopUpButton<T, F, S, M> {
    // Implementing this instead of handle_mouse_event makes all the children not receive events.
    fn process_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment) {
        if !env.is_event_current() { return }
        match event {
            MouseEvent::Click(_, position, _) => {
                if self.is_inside(*position) {
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
    M: ReadState<T=Vec<T>>
> Layout for PlainPopUpButton<T, F, S, M> {
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
        self.popup.position_children(env);
    }
}



// ---------------------------------------------------
//  Delegates
// ---------------------------------------------------
type DelegateGenerator<T: StateContract + PartialEq> =
    fn(selected_item: Box<dyn AnyState<T=T>>, focused: Box<dyn AnyState<T=Focus>>, popup_open: Box<dyn AnyReadState<T=bool>>) -> Box<dyn Widget>;

type PopupDelegateGenerator<T: StateContract + PartialEq, S: State<T=T>, M: ReadState<T=Vec<T>>, B: State<T=bool>,> =
    fn(model: M, delegate: PopupDelegate<T, S, B>) -> Box<dyn Widget>;

type PopupItemDelegateGenerator<T: StateContract + PartialEq, S: State<T=T>> =
    fn(item: Box<dyn AnyState<T=T>>, index: Box<dyn AnyReadState<T=usize>>, hover: Box<dyn AnyReadState<T=bool>>, selected: S) -> Box<dyn Widget>;

#[derive(Clone)]
pub struct PopupDelegate<T, S, B>
    where
        T: StateContract,
        S: State<T=T>,
        B: State<T=bool>,
{
    hover_model: TState<Option<usize>>,
    selected_item: S,
    popup_item_delegate: PopupItemDelegateGenerator<T, S>,
    popup_open: B,
}

impl<T: StateContract, S: State<T=T>, B: State<T=bool>> Delegate<T, Box<dyn Widget>> for PopupDelegate<T, S, B> {
    fn call(&self, item: Box<dyn AnyState<T=T>>, index: Box<dyn AnyState<T=usize>>) -> Box<dyn Widget> {
        let selected_item_del = self.selected_item.clone();
        let popup_open = self.popup_open.clone();

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
            |new, s1, s2| {
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
        );

        popup_item_delegate
            .on_click(move |env: &mut Environment, _| {
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
    popup_open: Box<dyn AnyReadState<T=bool>>,
) -> Box<dyn Widget> {
    let background_color = Map1::read_map(focused.clone(), |focused| {
        match *focused {
            Focus::Focused => EnvironmentColor::Green,
            _ => EnvironmentColor::Blue,
        }
    });

    ZStack::new(vec![
        Rectangle::new().fill(background_color),
        Text::new(selected_item.map(|a| format!("{:?}", a))),
    ])
}

fn default_popup_item_delegate<T: StateContract + PartialEq, S: State<T=T>>(
    item: Box<dyn AnyState<T=T>>,
    _index: Box<dyn AnyReadState<T=usize>>,
    hover_state: Box<dyn AnyReadState<T=bool>>,
    _selected_state: S,
) -> Box<dyn Widget> {
    let item_color = Map1::read_map(hover_state.clone(), |hovered| {
        if *hovered {
            EnvironmentColor::Pink
        } else {
            EnvironmentColor::Gray
        }
    });

    ZStack::new(vec![
        Rectangle::new().fill(item_color),
        Text::new(item.map(|a: &T| format!("{:?}", *a)).ignore_writes()),
    ])
        .frame_fixed_height(30.0)
}

fn default_popup_delegate<T: StateContract + PartialEq, S: State<T=T>, M: ReadState<T=Vec<T>>, B: State<T=bool>>(
    model: M,
    delegate: PopupDelegate<T, S, B>,
) -> Box<dyn Widget> {
    VStack::new(vec![
        ForEach::new(model.ignore_writes(), delegate)
    ]).spacing(1.0)
        .padding(1.0)
        .background(*Rectangle::new().fill(EnvironmentColor::Yellow))
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