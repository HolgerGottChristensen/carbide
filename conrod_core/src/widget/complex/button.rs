use uuid::Uuid;
use ::{Point, Rect};
use position::Dimensions;
use widget::render::{Render, ChildRender};
use render::primitive::Primitive;
use graph::Container;
use widget::{Id, Rectangle, Oval, HStack, Text};
use text::font::Map;
use widget::common_widget::CommonWidget;
use ::{text, Scalar};
use widget::primitive::Widget;
use widget::primitive::widget::WidgetExt;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use state::state::{StateList, GetState, State};
use daggy::petgraph::graph::node_index;
use render::primitive_kind::PrimitiveKind;
use widget::layout::Layout;
use layout::basic_layouter::BasicLayouter;
use widget::primitive::spacer::{Spacer, SpacerDirection};
use input::Key;
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};
use widget::primitive::foreach::ForEach;
use widget::primitive::v_stack::VStack;
use widget::complex::foreachtest::ForeachTest;

#[derive(Debug, Clone)]
pub struct SyncTest<S> {
    id: Uuid,
    child: Box<dyn Widget<S>>,
    position: Point,
    dimension: Dimensions,
    value: State<String>,
    fore: State<Vec<Uuid>>
}

impl<S: 'static + Clone> SyncTest<S> {
    pub fn new(value: State<String>) -> Box<SyncTest<S>> {
        let fore = State::<Vec<Uuid>>::new("a", &(0..5).map(|_| Uuid::new_v4()).collect::<Vec<Uuid>>());

        Box::new(Self {
            id: Uuid::new_v4(),
            child: HStack::initialize(vec![
                    Spacer::new(SpacerDirection::Horizontal),
                    VStack::initialize(vec![
                        ForEach::new(fore.clone(), ForeachTest::new())
                    ]),
                    ForEach::new((0..5).map(|_| Uuid::new_v4()).collect::<Vec<Uuid>>().into(), Rectangle::initialize(vec![]).frame(10.0,10.0)),
                    Text::initialize(value.clone(), vec![]),
                    Spacer::new(SpacerDirection::Horizontal),
                    Text::initialize(value.clone(), vec![]),
                    Spacer::new(SpacerDirection::Horizontal),
            ]),
            position: [100.0,100.0],
            dimension: [100.0,100.0],
            value,
            fore
        })
    }
}

impl<S> CommonWidget<S> for SyncTest<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<S> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
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

impl<S> Event<S> for SyncTest<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        match event {
            KeyboardEvent::Text(s, _) => {
                self.value.push_str(s);
            }
            KeyboardEvent::Press(key, modifier) => {
                match key {
                    Key::Backspace => {
                        self.value.pop();
                    },
                    Key::NumPadPlus => {
                        self.fore.push(Uuid::new_v4())
                    },
                    Key::NumPadMinus => {
                        if self.fore.len() > 1 {
                            let last = self.fore.len()-1;
                            self.fore.remove(last);
                        }

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

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList) -> StateList {
        self.process_mouse_event_default(event, consumed, state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut S) -> StateList {
        self.process_keyboard_event_default(event, state, global_state)
    }

    fn get_state(&self, mut current_state: StateList) -> StateList {
        current_state.replace_state(self.value.clone().into());
        current_state.replace_state(self.fore.clone().into());
        current_state
    }

    fn apply_state(&mut self, states: StateList) -> StateList {
        states.update_local_state(&mut self.value);
        states
    }

    fn sync_state(&mut self, states: StateList) {
        self.sync_state_default(states);
    }
}

impl<S> ChildRender for SyncTest<S> {}

impl<S> Layout for SyncTest<S> {
    fn flexibility(&self) -> u32 {
        2
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

impl<S: 'static + Clone> WidgetExt<S> for SyncTest<S> {}