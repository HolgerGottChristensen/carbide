use carbide_core::event::event_handler::{KeyboardEvent, MouseEvent};
use carbide_core::input::Key;
use carbide_core::input::MouseButton;
use carbide_core::prelude::Uuid;
use carbide_core::state::state::State;
use carbide_core::widget::*;

#[derive(Clone, Widget)]
#[event(handle_keyboard_event, handle_mouse_event)]
#[focusable]
pub struct PlainButton<T, GS> where GS: GlobalStateContract, T: StateContract + 'static {
    id: Id,
    #[state] focus: FocusState<GS>,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    on_click: Option<fn(myself: &mut Self, env: &mut Environment<GS>, global_state: &mut GS)>,
    on_click_outside: Option<fn(myself: &mut Self, env: &mut Environment<GS>, global_state: &mut GS)>,
    #[state] is_hovered: BoolState<GS>,
    #[state] is_pressed: BoolState<GS>,
    #[state] local_state: TState<T, GS>,
}

impl<T: StateContract + 'static, GS: GlobalStateContract> PlainButton<T, GS> {
    pub fn on_click(mut self, fire: fn(myself: &mut Self, env: &mut Environment<GS>, global_state: &mut GS)) -> Box<Self> {
        self.on_click = Some(fire);
        Box::new(self)
    }

    pub fn on_click_outside(mut self, fire: fn(myself: &mut Self, env: &mut Environment<GS>, global_state: &mut GS)) -> Box<Self> {
        self.on_click_outside = Some(fire);
        Box::new(self)
    }

    pub fn hover<K: Into<BoolState<GS>>>(mut self, is_hovered: K) -> Box<Self> {
        self.is_hovered = is_hovered.into();
        Box::new(self)
    }

    pub fn pressed<K: Into<BoolState<GS>>>(mut self, pressed: K) -> Box<Self> {
        self.is_pressed = pressed.into();
        Box::new(self)
    }

    pub fn focused<K: Into<FocusState<GS>>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Box::new(self)
    }

    pub fn local_state<K: Into<TState<T, GS>>>(mut self, state: K) -> Box<Self> {
        self.local_state = state.into();
        Box::new(self)
    }

    pub fn get_local_state(&mut self) -> &mut TState<T, GS> {
        &mut self.local_state
    }

    pub fn new(child: Box<dyn Widget<GS>>) -> Box<Self> {
        Box::new(PlainButton {
            id: Id::new_v4(),
            focus: CommonState::new_local_with_key(&Focus::Unfocused).into(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            on_click: None,
            on_click_outside: None,
            is_hovered: false.into(),
            is_pressed: false.into(),
            local_state: CommonState::new(&T::default()).into(),
        })
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment<GS>, global_state: &mut GS) {
        match event {
            MouseEvent::Press(MouseButton::Left, mouse_position, _) => {
                if self.is_inside(*mouse_position) {
                    *self.is_pressed.get_value_mut(env, global_state) = true;
                }
            }
            MouseEvent::Release(MouseButton::Left, mouse_position, _) => {
                if self.is_inside(*mouse_position) {
                    *self.is_pressed.get_value_mut(env, global_state) = false;
                }
            }
            MouseEvent::Move { to, .. } => {
                if *self.is_hovered.get_value(env, global_state) {
                    if !self.is_inside(*to) {
                        *self.is_hovered.get_value_mut(env, global_state) = false;
                        *self.is_pressed.get_value_mut(env, global_state) = false;
                    }
                } else {
                    if self.is_inside(*to) {
                        *self.is_hovered.get_value_mut(env, global_state) = true;
                    }
                }
            }
            MouseEvent::Click(MouseButton::Left, mouse_position, _) |
            MouseEvent::NClick(MouseButton::Left, mouse_position, _, _) => {
                if self.is_inside(*mouse_position) {
                    if let Some(action) = self.on_click {
                        action(self, env, global_state);
                    }
                } else {
                    if let Some(action) = self.on_click_outside {
                        action(self, env, global_state);
                    }
                }
            }
            _ => ()
        }
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        if self.get_focus() != Focus::Focused { return }

        match event {
            KeyboardEvent::Click(Key::Return, _) => {
                if let Some(action) = self.on_click {
                    action(self, env, global_state);
                    //self.set_focus_and_request(Focus::FocusReleased, env);
                }
            }
            _ => ()
        }
    }
}

impl<T: StateContract, GS: GlobalStateContract> CommonWidget<GS> for PlainButton<T, GS> {
    fn get_id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn get_children(&self) -> WidgetIter {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<T: StateContract, GS: GlobalStateContract> ChildRender for PlainButton<T, GS> {}

impl<T: StateContract, GS: GlobalStateContract> Layout<GS> for PlainButton<T, GS> {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        if let Some(child) = self.get_children_mut().next() {
            child.calculate_size(requested_size, env);
        }

        self.set_dimension(requested_size);

        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.get_position();
        let dimension = self.get_dimension();

        if let Some(child) = self.get_children_mut().next() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}


impl<T: StateContract + 'static, GS: GlobalStateContract> WidgetExt<GS> for PlainButton<T, GS> {}
