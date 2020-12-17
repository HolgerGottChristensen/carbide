use conrod_core::widget::primitive::Widget;
use conrod_core::{Point, Color};
use conrod_core::position::Dimensions;
use conrod_core::widget::{HStack, Rectangle, Image};
use conrod_core::widget::primitive::spacer::{SpacerDirection, Spacer};
use conrod_core::color::{RED, rgb_bytes};
use conrod_core::widget::primitive::edge_insets::EdgeInsets;
use conrod_core::widget::common_widget::CommonWidget;
use crate::calculator::calculator_state::CalculatorState;
use conrod_core::flags::Flags;
use conrod_core::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use conrod_core::event::event::Event;
use conrod_core::event_handler::{MouseEvent, WidgetEvent, KeyboardEvent};
use conrod_core::state::state::StateList;
use conrod_core::widget::render::ChildRender;
use conrod_core::layout::Layout;
use conrod_core::text::font::Map;
use conrod_core::layout::basic_layouter::BasicLayouter;
use conrod_core::widget::primitive::widget::WidgetExt;
use conrod_core::layout::layouter::Layouter;
use uuid::Uuid;
use conrod_core::image::Id;

#[derive(Clone)]
pub struct CalculatorButton {
    id: Uuid,
    child: Box<dyn Widget<CalculatorState>>,
    position: Point,
    dimension: Dimensions,
}

impl CalculatorButton {
    pub fn new(id: Id) -> Box<CalculatorButton> {
        Box::new(CalculatorButton {
            id: Uuid::new_v4(),
            child: Rectangle::initialize(vec![
                Image::new(id, [100.0,100.0], vec![])
            ]).fill(rgb_bytes(76,0,19)),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
        })
    }
}

impl CommonWidget<CalculatorState> for CalculatorButton {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<CalculatorState> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<CalculatorState> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<CalculatorState> {
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

impl Event<CalculatorState> for CalculatorButton {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut CalculatorState) {
        match event {
            KeyboardEvent::Text(st, _) => {
                println!("Hejsa");
            },
            _ => {},
        }
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList) -> StateList {
        self.process_mouse_event_default(event, consumed, state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut CalculatorState) -> StateList {
        self.process_keyboard_event_default(event, state, global_state)
    }

    fn get_state(&self, current_state: StateList) -> StateList {
        current_state
    }

    fn apply_state(&mut self, states: StateList) -> StateList {
        states
    }

    fn sync_state(&mut self, states: StateList) {
        self.sync_state_default(states)
    }
}

impl ChildRender for CalculatorButton {}

impl Layout for CalculatorButton {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        self.dimension = self.child.calculate_size(requested_size, fonts);
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;
        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl WidgetExt<CalculatorState> for CalculatorButton {}