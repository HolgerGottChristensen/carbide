use carbide_core::prelude::*;
use crate::calculator::calculator_state::CalculatorState;
use carbide_core::color::rgb_bytes;
use carbide_core::event_handler::MouseEvent;
use carbide_core::widget::{ChildRender, SingleChildLayout};

#[derive(Clone, Widget)]
#[global_state(CalculatorState)]
#[event(handle_mouse_event)]
pub struct CalculatorButton {
    id: Uuid,
    child: Box<dyn Widget<CalculatorState>>,
    position: Point,
    dimension: Dimensions,
    function: Option<fn(myself: &mut Self, global_state: &mut CalculatorState)>
}

impl CalculatorButton {
    pub fn new(display: Box<dyn Widget<CalculatorState>>) -> Box<CalculatorButton> {
        Box::new(CalculatorButton {
            id: Uuid::new_v4(),
            child: Rectangle::initialize(vec![
                display
            ]).fill(rgb_bytes(76,0,19)),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            function: None
        })
    }

    pub fn on_released(mut self, func: fn(&mut Self, &mut CalculatorState)) -> Box<Self>{
        self.function = Some(func);
        Box::new(self)
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, _: &mut Environment<CalculatorState>, global_state: &mut CalculatorState) {
        if !self.is_inside(event.get_current_mouse_position()) {return}
        match event {
            MouseEvent::Release(_, _, _) => {
                match self.function {
                    None => {}
                    Some(f) => {
                        f(self, global_state)
                    }
                }
            }
            _ => ()
        }
    }
}

impl CommonWidget<CalculatorState> for CalculatorButton {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<CalculatorState> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<CalculatorState> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<CalculatorState> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<CalculatorState> {
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


impl ChildRender for CalculatorButton {}

impl SingleChildLayout for CalculatorButton {
    fn flexibility(&self) -> u32 {
        10
    }
}
