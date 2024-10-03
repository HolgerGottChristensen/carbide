use std::fmt::{Debug, Formatter};
use carbide::environment::WidgetTransferAction;
use carbide::event::{EventId, KeyboardEventContext, MouseButton, MouseEventContext};
use carbide::layout::LayoutContext;
use carbide::update::{Update, UpdateContext};

use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::environment::EnvironmentColor;
use carbide_core::event::{
    Key, KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler,
};
use carbide_core::flags::WidgetFlag;
use carbide_core::focus::{Focus, Refocus};
use carbide_core::layout::{Layout};
use carbide_core::state::{AnyReadState, AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, ReadStateExtNew, State, StateContract, StateExtNew};
use carbide_core::widget::*;

use crate::plain::plain_pop_up_button_popup::PlainPopUpButtonPopUp;
use crate::plain::plain_pop_up_button_popup_item::PlainPopUpButtonPopUpItem;

#[derive(Clone, Widget)]
#[carbide_exclude(Layout, MouseEvent, KeyboardEvent, Update)]
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
    position: LocalState<Position>,
    dimension: LocalState<Dimension>,
    #[state] focus: F,
    #[state] enabled: E,

    // Delegates
    text_delegate: TextDelegateGenerator<T>,
    delegate: DelegateGenerator<T>, // Used to generate the control
    popup_delegate: PopupDelegateGenerator<T, S, M>, // Used to generate the popup
    popup_item_delegate: PopupItemDelegateGenerator<T, S>, // Used to generate each item in the popup

    child: Box<dyn AnyWidget>,

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
            |t| Map1::read_map(t, |s| format!("{:?}", s)).as_dyn_read(),
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
            self.text_delegate,
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
            self.text_delegate,
            self.delegate,
            self.popup_delegate,
            self.popup_item_delegate
        )
    }

    pub fn text_delegate(self, text_delegate: TextDelegateGenerator<T>) -> PlainPopUpButton<T, F, S, M, E> {
        Self::new_internal(
            self.selected,
            self.model,
            self.focus,
            self.enabled,
            text_delegate,
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
            self.text_delegate,
            delegate,
            self.popup_delegate,
            self.popup_item_delegate
        )
    }

    pub fn popup_delegate(self, popup_delegate: PopupDelegateGenerator<T, S, M>) -> PlainPopUpButton<T, F, S, M, E> {
        Self::new_internal(
            self.selected,
            self.model,
            self.focus,
            self.enabled,
            self.text_delegate,
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
            self.text_delegate,
            self.delegate,
            self.popup_delegate,
            popup_item_delegate
        )
    }

    fn open_popup(&self, event_id: EventId, env: &mut Environment) {
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

        let popup = PlainPopUpButtonPopUp::new(
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

        env.transfer_widget(Some("controls_popup_layer".to_string()), WidgetTransferAction::Push(popup));
    }

    fn new_internal<T2: StateContract + PartialEq, F2: State<T=Focus>, S2: State<T=T2>, M2: ReadState<T=Vec<T2>>, E2: ReadState<T=bool>>(
        selected: S2,
        model: M2,
        focus: F2,
        enabled: E2,
        text_delegate: TextDelegateGenerator<T2>,
        delegate: DelegateGenerator<T2>,
        popup_delegate: PopupDelegateGenerator<T2, S2, M2>,
        popup_item_delegate: PopupItemDelegateGenerator<T2, S2>,
    ) -> PlainPopUpButton<T2, F2, S2, M2, E2> {

        let child = delegate(selected.as_dyn(), focus.as_dyn(), enabled.as_dyn_read(), text_delegate);

        PlainPopUpButton {
            id: WidgetId::new(),
            position: LocalState::new(Position::new(0.0, 0.0)),
            dimension: LocalState::new(Dimension::new(100.0, 100.0)),
            focus,
            enabled,

            text_delegate,
            delegate,
            popup_delegate,
            popup_item_delegate,

            child,

            selected,
            model,
        }
    }
}


#[cfg(feature = "carbide_fluent")]
impl<
    T: StateContract + PartialEq + carbide_fluent::Localizable,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> PlainPopUpButton<T, F, S, M, E> {
    pub fn localize(self) -> PlainPopUpButton<T, F, S, M, E> {
        self.text_delegate(|item| carbide_fluent::LocalizedString::new(item).as_dyn_read())
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> CommonWidget for PlainPopUpButton<T, F, S, M, E> {
    fn position(&self) -> Position {
        *self.position.value()
    }

    fn set_position(&mut self, position: Position) {
        self.position.set_value(position);
    }

    fn dimension(&self) -> Dimension {
        *self.dimension.value()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension.set_value(dimension);
    }

    CommonWidgetImpl!(self, id: self.id, child: self.child, flag: WidgetFlag::FOCUSABLE, flexibility: 1, focus: self.focus);
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
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if self.get_focus() != Focus::Focused || !*self.enabled.value() { return; }

        if event == PopupButtonKeyCommand::Open {
            self.open_popup(EventId::default(), ctx.env);
            //ctx.env.request_animation_frame();
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
                        ctx.env.request_focus(Refocus::FocusRequest);
                    }
                    self.open_popup(*id, ctx.env);
                    //ctx.env.request_animation_frame();
                } else {
                    if self.get_focus() == Focus::Focused {
                        self.set_focus(Focus::FocusReleased);
                        ctx.env.request_focus(Refocus::FocusRequest);
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
> Update for PlainPopUpButton<T, F, S, M, E> {
    fn update(&mut self, _ctx: &mut UpdateContext) {
        //self.popup.ensure_overlay_correct(ctx.env)
    }
}

impl<
    T: StateContract + PartialEq,
    F: State<T=Focus>,
    S: State<T=T>,
    M: ReadState<T=Vec<T>>,
    E: ReadState<T=bool>,
> Layout for PlainPopUpButton<T, F, S, M, E> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let dimension = self.child.calculate_size(requested_size, ctx);
        self.set_dimension(dimension);
        dimension
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let alignment = self.alignment();
        let position = self.position();
        let dimension = self.dimension();

        self.child.set_position(alignment.position(position, dimension, self.child.dimension()));
        self.child.position_children(ctx);

        /*positioning(position, dimension, &mut self.popup);

        // Keep the Y within the screen
        let popup_position = self.popup.position();
        let popup_dimension = self.popup.dimension();
        let mut y = popup_position.y();

        if y + popup_dimension.height > ctx.env.current_window_height() {
            y = ctx.env.current_window_height() - popup_dimension.height;
        }
        if y < 0.0 {
            y = 0.0;
        }

        self.popup.set_position(Position::new(popup_position.x(), y));
        self.popup.position_children(ctx);*/
    }
}



// ---------------------------------------------------
//  Delegates
// ---------------------------------------------------
type TextDelegateGenerator<T> = fn(Box<dyn AnyReadState<T=T>>)->Box<dyn AnyReadState<T=String>>;

type DelegateGenerator<T> = fn(
    selected_item: Box<dyn AnyState<T=T>>,
    focused: Box<dyn AnyState<T=Focus>>,
    enabled: Box<dyn AnyReadState<T=bool>>,
    text_delegate: TextDelegateGenerator<T>,
) -> Box<dyn AnyWidget>;

type PopupDelegateGenerator<T, S, M> = fn(
    model: M,
    delegate: PopupItemDelegate<T, S>,
    enabled: Box<dyn AnyReadState<T=bool>>,
) -> Box<dyn AnyWidget>;

type PopupItemDelegateGenerator<T, S> = fn(
    item: Box<dyn AnyState<T=T>>,
    index: Box<dyn AnyReadState<T=usize>>,
    hover: Box<dyn AnyReadState<T=bool>>,
    selected: S,
    enabled: Box<dyn AnyReadState<T=bool>>,
    text_delegate: TextDelegateGenerator<T>,
) -> Box<dyn AnyWidget>;

#[derive(Clone)]
pub struct PopupItemDelegate<T, S>
    where
        T: StateContract,
        S: State<T=T>,
{
    hover_model: LocalState<Option<usize>>,
    selected_item: S,
    popup_item_delegate: PopupItemDelegateGenerator<T, S>,
    text_delegate: TextDelegateGenerator<T>,
    enabled: Box<dyn AnyReadState<T=bool>>,
    overlay_id: Option<String>,
    event_id: EventId,
}

impl<T: StateContract, S: State<T=T>> Delegate<T, Box<dyn AnyWidget>> for PopupItemDelegate<T, S> {
    fn call(&self, item: Box<dyn AnyState<T=T>>, index: Box<dyn AnyReadState<T=usize>>) -> Box<dyn AnyWidget> {
        // Map the hovered index to a boolean state telling us whether this item is hovered.
        // If we set this state to be hovered, we set the index, and otherwise we set to None.
        let hover_state = Map2::map(
            self.hover_model.clone(),
            index.clone().ignore_writes(),
            |a, b| {
                if let Some(a) = a {
                    *a == *b
                } else {
                    false
                }
            },
            |new, mut s1, s2| {
                if new {
                    *s1 = Some(*s2);
                } else {
                    *s1 = None;
                }
            }
        ).as_dyn();

        let popup_item_delegate = (self.popup_item_delegate)(
            item.clone(),
            index.as_dyn_read(),
            hover_state.as_dyn_read(),
            self.selected_item.clone(),
            self.enabled.as_dyn_read(),
            self.text_delegate,
        );

        PlainPopUpButtonPopUpItem::new(
            popup_item_delegate,
            self.selected_item.clone(),
            item,
            hover_state,
            self.overlay_id.clone(),
            self.event_id,
        ).boxed()
    }
}

fn default_delegate<T: StateContract + PartialEq>(
    selected_item: Box<dyn AnyState<T=T>>,
    focused: Box<dyn AnyState<T=Focus>>,
    _enabled: Box<dyn AnyReadState<T=bool>>,
    text_delegate: TextDelegateGenerator<T>,
) -> Box<dyn AnyWidget> {
    let background_color = Map1::read_map(focused.clone(), |focused| {
        match *focused {
            Focus::Focused => EnvironmentColor::Green,
            _ => EnvironmentColor::Blue,
        }
    });

    ZStack::new((
        Rectangle::new().fill(background_color),
        Text::new(text_delegate(selected_item.as_dyn_read())),
    )).boxed()
}

fn default_popup_item_delegate<T: StateContract + PartialEq, S: State<T=T>>(
    item: Box<dyn AnyState<T=T>>,
    _index: Box<dyn AnyReadState<T=usize>>,
    hover_state: Box<dyn AnyReadState<T=bool>>,
    _selected_state: S,
    _enabled: Box<dyn AnyReadState<T=bool>>,
    text_delegate: TextDelegateGenerator<T>,
) -> Box<dyn AnyWidget> {
    let item_color = Map1::read_map(hover_state.clone(), |hovered| {
        if *hovered {
            EnvironmentColor::Pink
        } else {
            EnvironmentColor::Gray
        }
    });

    ZStack::new((
        Rectangle::new().fill(item_color),
        Text::new(text_delegate(item.as_dyn_read())),
    ))
        .frame_fixed_height(30.0)
        .boxed()
}

fn default_popup_delegate<T: StateContract + PartialEq, S: State<T=T>, M: ReadState<T=Vec<T>>>(
    model: M,
    delegate: PopupItemDelegate<T, S>,
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
                matches!(self, KeyboardEvent::Press(Key::ArrowDown, _))
            }
            PopupButtonKeyCommand::Prev => {
                matches!(self, KeyboardEvent::Press(Key::ArrowUp, _))
            }
            PopupButtonKeyCommand::Select => {
                matches!(self, KeyboardEvent::Press(Key::Enter, _))
            }
            PopupButtonKeyCommand::Close => {
                matches!(self, KeyboardEvent::Press(Key::Escape, _))
            }
            PopupButtonKeyCommand::Open => {
                matches!(self, KeyboardEvent::Press(Key::Space, _) | KeyboardEvent::Press(Key::Enter, _))
            }
        }
    }
}