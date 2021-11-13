use carbide_core::color::rgb_bytes;
use carbide_core::draw::Dimension;
use carbide_core::event::event_handler::MouseEvent;
use carbide_core::prelude::*;
use carbide_core::widget::{ChildRender, SingleChildLayout};

use crate::calculator::calculator_state::CalculatorState;

#[derive(Clone, Widget)]
#[global_state(CalculatorState)]
#[event(handle_mouse_event)]
pub struct CalculatorButton {
    id: Uuid,
    child: Box<dyn Widget<CalculatorState>>,
    position: Point,
    dimension: Dimensions,
    function: Option<fn(myself: &mut Self, global_state: &mut CalculatorState)>,
}

impl CalculatorButton {
    pub fn new(display: Box<dyn Widget<CalculatorState>>) -> Box<CalculatorButton> {
        Box::new(CalculatorButton {
            id: Uuid::new_v4(),
            child: Rectangle::new_old(vec![display]).fill(rgb_bytes(76, 0, 19)),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            function: None,
        })
    }

    pub fn on_released(mut self, func: fn(&mut Self, &mut CalculatorState)) -> Box<Self> {
        self.function = Some(func);
        Box::new(self)
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        _consumed: &bool,
        _: &mut Environment<CalculatorState>,
        global_state: &mut CalculatorState,
    ) {
        if !self.is_inside(event.get_current_mouse_position()) {
            return;
        }
        match event {
            MouseEvent::Release(_, _, _) => match self.function {
                None => {}
                Some(f) => f(self, global_state),
            },
            _ => (),
        }
    }
}

impl CommonWidget<CalculatorState> for CalculatorButton {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::EMPTY
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

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl ChildRender for CalculatorButton {}

impl SingleChildLayout for CalculatorButton {
    fn flexibility(&self) -> u32 {
        10
    }
}
