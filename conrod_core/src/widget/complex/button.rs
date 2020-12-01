use uuid::Uuid;
use ::{Point, Rect};
use position::Dimensions;
use widget::render::{Render, ChildRender};
use render::primitive::Primitive;
use graph::Container;
use widget::{Id, Rectangle, HStack, Text};
use text::font::Map;
use widget::common_widget::CommonWidget;
use ::{text, Scalar};
use widget::primitive::Widget;
use widget::primitive::widget::WidgetExt;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use state::state::{StateList, DefaultState, GetState, State};
use daggy::petgraph::graph::node_index;
use render::primitive_kind::PrimitiveKind;
use widget::layout::Layout;
use layout::basic_layouter::BasicLayouter;
use widget::primitive::spacer::{Spacer, SpacerDirection};
use input::Key;

#[derive(Debug)]
pub struct SyncTest {
    id: Uuid,
    children: Vec<Box<dyn Widget>>,
    position: Point,
    dimension: Dimensions,
    value: State<String>
}

impl SyncTest {
    pub fn new(value: State<String>) -> Box<SyncTest> {
        Box::new(Self {
            id: Uuid::new_v4(),
            children: vec![
                HStack::initialize(vec![
                    Spacer::new(SpacerDirection::Horizontal),
                    Text::initialize(value.clone(), vec![]),
                    Spacer::new(SpacerDirection::Horizontal),
                    Text::initialize(value.clone(), vec![]),
                    Spacer::new(SpacerDirection::Horizontal),
                ])
            ],
            position: [100.0,100.0],
            dimension: [100.0,100.0],
            value
        })
    }
}

impl CommonWidget for SyncTest {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_children(&self) -> &Vec<Box<dyn Widget>> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        &mut self.children
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn get_x(&self) -> Scalar {
        self.position[0]
    }

    fn set_x(&mut self, x: Scalar) {
        self.position = [x, self.position[1]];
    }

    fn get_y(&self) -> Scalar {
        self.position[1]
    }

    fn set_y(&mut self, y: Scalar) {
        self.position = [self.position[0], y];
    }

    fn get_size(&self) -> Dimensions {
        self.dimension
    }

    fn get_width(&self) -> Scalar {
        self.dimension[0]
    }

    fn get_height(&self) -> Scalar {
        self.dimension[1]
    }
}

impl Event for SyncTest {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        match event {
            KeyboardEvent::Text(s, _) => {
                self.value.push_str(s);
            }
            KeyboardEvent::Press(key, modifier) => {
                match key {
                    Key::Backspace => {
                        self.value.pop();
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        ()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList<DefaultState>) -> StateList<DefaultState> {
        self.process_mouse_event_default(event, consumed, state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList<DefaultState>) -> StateList<DefaultState> {
        self.process_keyboard_event_default(event, state)
    }

    fn get_state(&self, mut current_state: StateList<DefaultState>) -> StateList<DefaultState> {
        current_state.replace_state(self.value.clone().into());
        current_state
    }

    fn apply_state(&mut self, states: StateList<DefaultState>) -> StateList<DefaultState> {
        match states.get_state(&self.value.id) {
            None => (),
            Some(v) => {
                self.value = v.clone().into()
            }
        }
        states
    }

    fn sync_state(&mut self, states: StateList<DefaultState>) {
        self.sync_state_default(states);
    }
}

impl ChildRender for SyncTest {}

impl Layout for SyncTest {
    fn flexibility(&self) -> u32 {
        2
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {

        self.dimension = self.children.first_mut().unwrap().calculate_size(requested_size, fonts);
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in &mut self.children {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl WidgetExt for SyncTest {}