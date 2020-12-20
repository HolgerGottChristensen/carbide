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
use state::state::{StateList, GetState, State};
use daggy::petgraph::graph::node_index;
use render::primitive_kind::PrimitiveKind;

use layout::basic_layouter::BasicLayouter;
use widget::primitive::spacer::{Spacer, SpacerDirection};
use input::Key;
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};
use std::collections::HashMap;
use layout::Layout;
use std::fmt::Debug;


#[derive(Debug, Clone)]
pub struct ForEach<S: Clone + Debug> {
    id: Uuid,
    children_map: HashMap<Uuid, Box<dyn Widget<S>>>,
    delegate: Box<dyn Widget<S>>,
    ids: State<Vec<Uuid>, S>,
    position: Point,
    dimension: Dimensions
}

impl<S: Clone + Debug> ForEach<S> {
    pub fn new(ids: State<Vec<Uuid>, S>, delegate: Box<dyn Widget<S>>) -> Box<ForEach<S>> {

        let mut map = HashMap::new();

        for i in ids.get_latest_value() {
            map.insert(i.clone(), Clone::clone(&delegate));
        }

        Box::new(Self {
            id: Uuid::new_v4(),
            children_map: map,
            delegate,
            ids,
            position: [100.0,100.0],
            dimension: [100.0,100.0]
        })
    }
}

impl<S: Clone + Debug> CommonWidget<S> for ForEach<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Proxy
    }

    fn get_children(&self) -> WidgetIter<S> {
        let mut w = WidgetIter::Empty;

        for id in self.ids.get_latest_value().iter().rev() {
            let item = self.children_map.get(id).unwrap();

            if item.get_flag() == Flags::Proxy {
                w = WidgetIter::Multi(Box::new(item.get_children()), Box::new(w));
            } else {
                w = WidgetIter::Single(item, Box::new(w))
            }
        }

        w
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        let mut w = WidgetIterMut::Empty;

        for id in self.ids.get_latest_value().iter().rev() {
            let contains = self.children_map.contains_key(id).clone();
            if !contains{
                self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
            }
        }

        for id in self.ids.get_latest_value().iter().rev() {

            let item: &mut Box<dyn Widget<S>> = unsafe {
                let p: *mut Box<dyn Widget<S>> = self.children_map.get_mut(id).unwrap();
                p.as_mut().unwrap()
            };

            if item.get_flag() == Flags::Proxy {
                w = WidgetIterMut::Multi(Box::new(item.get_children_mut()), Box::new(w));
            } else {
                w = WidgetIterMut::Single(item, Box::new(w))
            }
        }

        w
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        let mut w = WidgetIterMut::Empty;

        for id in self.ids.get_latest_value().iter().rev() {
            let contains = self.children_map.contains_key(id).clone();
            if !contains{
                self.children_map.insert(id.clone(), Clone::clone(&self.delegate));
            }
        }

        for id in self.ids.get_latest_value().iter().rev() {
            let item: &mut Box<dyn Widget<S>> = unsafe {
                let p: *mut Box<dyn Widget<S>> = self.children_map.get_mut(id).unwrap();
                p.as_mut().unwrap()
            };

            if item.get_flag() == Flags::Proxy {
                w = WidgetIterMut::Multi(Box::new(item.get_proxied_children()), Box::new(w));
            } else {
                w = WidgetIterMut::Single(item, Box::new(w))
            }
        }

        w
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

impl<S: Clone + Debug> Event<S> for ForEach<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut S) {
        unimplemented!()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        unimplemented!()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList, global_state: &mut S) -> StateList {
        println!("Foreach mouseevent");

        state
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut S) -> StateList {
        // Apply state from its parent
        let new_state = self.apply_state(state, global_state);

        // Add the state from itself, to the state list
        let mut state_for_children = new_state; //self.get_state(new_state);

        let mut ids = self.ids.clone();

        for (i, child) in self.get_proxied_children().enumerate() {

            state_for_children.replace_state(State::<Uuid, S>::new("id", &ids.get_value(global_state)[i]).into());
            state_for_children.replace_state(State::<u32, S>::new("index", &(i as u32)).into());
            // Then we delegate the event to its children, we also makes sure to update
            // current state for the next child
            state_for_children = child.process_keyboard_event(event, state_for_children, global_state);

        }
        // We then apply the changed state from its children, to save it for itself.
        self.apply_state(state_for_children, global_state)
    }

    fn get_state(&self, mut current_state: StateList) -> StateList {
        unimplemented!()
    }

    fn apply_state(&mut self, states: StateList, global_state: &S) -> StateList {
        states.update_local_state(&mut self.ids, global_state);
        states
    }

    fn sync_state(&mut self, states: StateList, global_state: &S) {
        unimplemented!()
    }
}

impl<S: Clone + Debug> ChildRender for ForEach<S> {}

impl<S: Clone + Debug> Layout for ForEach<S> {
    fn flexibility(&self) -> u32 {
        unimplemented!()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        unimplemented!()
    }

    fn position_children(&mut self) {
        unimplemented!()
    }
}

impl<S: 'static + Clone + Debug> WidgetExt<S> for ForEach<S> {}