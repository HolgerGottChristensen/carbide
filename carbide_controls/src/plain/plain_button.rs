use carbide_core::widget::*;
use carbide_core::event_handler::{MouseEvent, KeyboardEvent};
use carbide_core::input::MouseButton;
use carbide_core::input::Key;
use carbide_core::state::state::State;

#[event(handle_keyboard_event, handle_mouse_event)]
#[derive(Clone, Widget)]
pub struct PlainButton<GS> where GS: GlobalState {
    id: Id,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    on_click: Option<fn(myself: &mut Self, env: &mut Environment<GS>, global_state: &mut GS)>,
    #[state] is_hovered: Box<dyn State<bool, GS>>,
    #[state] is_pressed: Box<dyn State<bool, GS>>,
}

impl<GS: GlobalState> PlainButton<GS> {
    pub fn on_click(mut self, fire: fn(myself: &mut Self, env: &mut Environment<GS>, global_state: &mut GS)) -> Box<Self> {
        self.on_click = Some(fire);
        Box::new(self)
    }

    pub fn new(child: Box<dyn Widget<GS>>) -> Box<Self> {
        Box::new(PlainButton {
            id: Id::new_v4(),
            child,
            position: [0.0,0.0],
            dimension: [0.0,0.0],
            on_click: None,
            is_hovered: false.into(),
            is_pressed: false.into()
        })
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment<GS>, global_state: &mut GS) {
        match event {
            MouseEvent::Press(MouseButton::Left, mouse_position, _) => {
                if self.is_inside(*mouse_position) {
                    *self.is_pressed.get_value_mut(global_state) = true;
                }
            }
            MouseEvent::Move { to, .. } => {
                if *self.is_hovered.get_value(global_state) {
                   if !self.is_inside(*to) {
                       *self.is_hovered.get_value_mut(global_state) = false;
                       *self.is_pressed.get_value_mut(global_state) = false;
                   }
                } else {
                    if self.is_inside(*to) {
                        *self.is_hovered.get_value_mut(global_state) = true;
                    }
                }
            }
            MouseEvent::Click(MouseButton::Left, mouse_position, _) |
            MouseEvent::NClick(MouseButton::Left, mouse_position, _, _) => {
                if self.is_inside(*mouse_position) {
                    if let Some(action) = self.on_click {
                        action(self, env, global_state);
                    }
                }
            }
            _ => ()
        }
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        match event {
            KeyboardEvent::Click(Key::Return, _) => {
                if let Some(action) = self.on_click {
                    action(self, env, global_state);
                }
            }
            _ => ()
        }
    }
}

impl<GS: GlobalState> CommonWidget<GS> for PlainButton<GS> {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
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

impl<GS: GlobalState> ChildRender for PlainButton<GS> {}

impl<GS: GlobalState> Layout<GS> for PlainButton<GS> {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: [f64; 2], env: &Environment<GS>) -> [f64; 2] {
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


impl<GS: GlobalState> WidgetExt<GS> for PlainButton<GS> {}
