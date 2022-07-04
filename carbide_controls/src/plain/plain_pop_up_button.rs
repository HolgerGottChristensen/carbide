use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{
    Key, KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, WidgetEvent,
};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::layout::{Layout, Layouter};
use carbide_core::prelude::{EnvironmentColor, Primitive};
use carbide_core::render::Render;
use carbide_core::state::{
    BoolState, FocusState, LocalState, Map3, ReadState, State, StateContract, StateExt, StateKey,
    StateSync, TState, UsizeState,
};
use carbide_core::widget::*;
use carbide_core::{Color, Scalar};

use crate::plain::plain_pop_up_button_popup::PlainPopUpButtonPopUp;
use crate::plain::plain_pop_up_button_popup_item::PlainPopUpButtonPopUpItem;
use crate::{List, PlainButton};

#[derive(Clone)]
pub struct PopupDelegate<T>
where
    T: StateContract,
{
    hover_model: TState<Vec<bool>>,
    selected_item: TState<T>,
    popup_item_delegate: PopupItemDelegateGenerator<T>,
}

impl<T: StateContract> Delegate<T> for PopupDelegate<T> {
    fn call(&self, item: TState<T>, index: UsizeState) -> Box<dyn Widget> {
        let hover_state = self.hover_model.index(index.clone());
        let selected_item_del = self.selected_item.clone();

        let popup_item_delegate = (self.popup_item_delegate)(
            item.clone(),
            index,
            hover_state.clone(),
            selected_item_del.clone(),
        );

        let hovered = PlainButton::new(popup_item_delegate).hovered(hover_state.clone());

        PlainPopUpButtonPopUpItem::new(hovered, hover_state, item, selected_item_del)
    }
}

type DelegateGenerator<T: StateContract + PartialEq + 'static> =
    fn(selected_item: TState<T>, focused: FocusState) -> Box<dyn Widget>;
type PopupDelegateGenerator<T: StateContract + PartialEq + 'static> =
    fn(model: TState<Vec<T>>, delegate: PopupDelegate<T>) -> Box<dyn Widget>;
type PopupItemDelegateGenerator<T: StateContract + PartialEq + 'static> = fn(
    item: TState<T>,
    index: UsizeState,
    hover: BoolState,
    selected: TState<T>,
) -> Box<dyn Widget>;

#[derive(Clone, Widget)]
#[carbide_exclude(Render, Layout, MouseEvent, KeyboardEvent)]
pub struct PlainPopUpButton<T>
where
    T: StateContract + PartialEq + 'static,
{
    id: WidgetId,
    #[state]
    focus: FocusState,
    child: Box<dyn Widget>,

    popup_item_delegate: PopupItemDelegateGenerator<T>,
    popup_delegate: PopupDelegateGenerator<T>,
    delegate: DelegateGenerator<T>,

    position: Position,
    dimension: Dimension,
    popup_list_spacing: f64,
    popup: Overlay,
    #[state]
    selected_item: TState<T>,
    #[state]
    model: TState<Vec<T>>,
}

impl<T: StateContract + PartialEq + 'static> PlainPopUpButton<T> {
    pub fn new<M: Into<TState<Vec<T>>>, S: Into<TState<T>>>(
        model: M,
        selected_state: S,
    ) -> Box<Self> {
        let focus: FocusState = LocalState::new(Focus::Unfocused).into();

        Self::new_internal(
            focus,
            model,
            selected_state,
            Self::default_popup_item_delegate,
            Self::default_popup_delegate,
            Self::default_delegate,
        )
    }

    pub fn delegate(mut self, delegate: DelegateGenerator<T>) -> Box<Self> {
        self.delegate = delegate;
        Self::new_internal(
            self.focus,
            self.model,
            self.selected_item,
            self.popup_item_delegate,
            self.popup_delegate,
            self.delegate,
        )
    }

    pub fn popup_item_delegate(
        mut self,
        popup_item_delegate: PopupItemDelegateGenerator<T>,
    ) -> Box<Self> {
        self.popup_item_delegate = popup_item_delegate;
        Self::new_internal(
            self.focus,
            self.model,
            self.selected_item,
            self.popup_item_delegate,
            self.popup_delegate,
            self.delegate,
        )
    }

    pub fn popup_delegate(mut self, popup_delegate: PopupDelegateGenerator<T>) -> Box<Self> {
        self.popup_delegate = popup_delegate;
        Self::new_internal(
            self.focus,
            self.model,
            self.selected_item,
            self.popup_item_delegate,
            self.popup_delegate,
            self.delegate,
        )
    }

    fn new_internal<M: Into<TState<Vec<T>>>, S: Into<TState<T>>>(
        focus: FocusState,
        model: M,
        selected_state: S,
        popup_item_delegate: PopupItemDelegateGenerator<T>,
        popup_delegate: PopupDelegateGenerator<T>,
        delegate: DelegateGenerator<T>,
    ) -> Box<PlainPopUpButton<T>> {
        let model = model.into();
        let selected_item = selected_state.into();
        let hover_model: TState<Vec<bool>> =
            LocalState::new(vec![false; model.value().len()]).into();

        let del = PopupDelegate {
            hover_model: hover_model.clone(),
            selected_item: selected_item.clone(),
            popup_item_delegate,
        };

        let popup_delegate_widget = popup_delegate(model.clone(), del);

        let popup = Overlay::new(PlainPopUpButtonPopUp::new(
            popup_delegate_widget,
            hover_model,
        ));

        let child = delegate(selected_item.clone(), focus.clone());

        Box::new(PlainPopUpButton {
            id: WidgetId::new(),
            focus,
            child,
            popup_item_delegate,
            popup_delegate,
            delegate,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            popup_list_spacing: 0.0,
            popup,
            selected_item,
            model,
        })
    }

    fn default_delegate(selected_item: TState<T>, focused: FocusState) -> Box<dyn Widget> {
        let blue = EnvironmentColor::Blue.state();
        let green = EnvironmentColor::Green.state();

        let background_color = Map3::read_map(
            focused,
            blue,
            green,
            |focus: &Focus, blue: &Color, green: &Color| match focus {
                Focus::Focused => *green,
                _ => *blue,
            },
        )
        .ignore_writes();

        let text = selected_item.map(|a: &T| format!("{:?}", a));
        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(text.ignore_writes()),
        ])
    }

    fn default_popup_item_delegate(
        item: TState<T>,
        index: UsizeState,
        hover_state: BoolState,
        selected_state: TState<T>,
    ) -> Box<dyn Widget> {
        let item_color: TState<Color> = hover_state
            .choice(
                EnvironmentColor::Pink.state(),
                EnvironmentColor::Gray.state(),
            )
            .ignore_writes();

        ZStack::new(vec![
            Rectangle::new().fill(item_color),
            Text::new(item.map(|a: &T| format!("{:?}", *a)).ignore_writes()),
        ])
        .frame_fixed_height(30.0)
    }

    fn default_popup_delegate(
        model: TState<Vec<T>>,
        delegate: PopupDelegate<T>,
    ) -> Box<dyn Widget> {
        List::new(model, delegate).spacing(0.0).clip()
    }
}

impl<T: StateContract + PartialEq + 'static> KeyboardEventHandler for PlainPopUpButton<T> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        if self.get_focus() != Focus::Focused {
            return;
        }

        match event {
            KeyboardEvent::Press(key, modifier) => match (key, modifier) {
                (Key::Space, _) | (Key::Return, _) => {
                    self.popup.set_showing(true);
                    env.add_overlay("controls_popup_layer", Some(self.popup.clone()));
                    env.request_animation_frame();
                }
                (_, _) => {}
            },
            _ => {}
        }
    }
}

impl<T: StateContract + PartialEq + 'static> MouseEventHandler for PlainPopUpButton<T> {
    // Implementing this instead of handle_mouse_event makes all the children not receive events.
    fn process_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment) {
        match event {
            MouseEvent::Click(_, position, _) => {
                if self.is_inside(*position) {
                    self.popup.set_showing(true);
                    //println!("{:#?}", self.popup);
                    env.add_overlay("controls_popup_layer", Some(self.popup.clone()));
                    env.request_animation_frame();
                }
            }
            _ => (),
        }
    }
}

impl<T: StateContract + PartialEq + 'static> CommonWidget for PlainPopUpButton<T> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_focus(&self) -> Focus {
        self.focus.value().clone()
    }

    fn set_focus(&mut self, focus: Focus) {
        *self.focus.value_mut() = focus;
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn flexibility(&self) -> u32 {
        10
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<T: StateContract + PartialEq + 'static> Layout for PlainPopUpButton<T> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let dimensions = self.child.calculate_size(requested_size, env);
        self.set_dimension(dimensions);

        // We calculate the size for the popup if it is open
        if self.popup.is_showing() {
            let max_height = 400.0;
            let max_height = env.get_corrected_height().min(max_height);
            let popup_request = Dimension::new(dimensions.width, max_height);
            self.popup.calculate_size(popup_request, env);
            self.popup.set_y(env.get_corrected_height());
        }
        dimensions
    }

    fn position_children(&mut self) {
        let positioning = self.alignment().positioner();
        let position = self.position();
        let dimension = self.dimension();

        if let Some(mut child) = self.children_mut().next() {
            positioning(position, dimension, child.deref_mut());
            child.position_children();
        }

        if self.popup.is_showing() {
            // This is kinda a hack to get the window height, by setting it in the size calculation
            // where we have that information. In the future we should probably allow the position
            // children to have access to the environment.
            let window_height = self.popup.y();
            let child_height = self.popup.height();

            let y = self.position.y() + self.dimension.height / 2.0 - child_height / 2.0;

            // We need the y to inside the window
            let y = y.min(window_height - child_height).max(0.0);

            self.popup.set_x(self.x());
            self.popup.set_y(y);
            self.popup.position_children();
        }
    }
}

impl<T: StateContract + PartialEq + 'static> Render for PlainPopUpButton<T> {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.child.process_get_primitives(primitives, env);
    }
}

impl<T: StateContract + PartialEq + 'static> Debug for PlainPopUpButton<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainPopUpButton")
            .field("child", &self.child)
            .finish()
    }
}

impl<T: StateContract + PartialEq + 'static> WidgetExt for PlainPopUpButton<T> {}
