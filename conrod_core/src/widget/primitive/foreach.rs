use std::collections::HashMap;
use std::fmt::Debug;

use uuid::Uuid;

use crate::Point;
use crate::event::event::NoEvents;
use crate::flags::Flags;
use crate::layout::Layout;
use crate::position::Dimensions;
use crate::state::environment::Environment;
use crate::state::state::{GetState, State};
use crate::state::state_sync::StateSync;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::ChildRender;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};

#[derive(Debug, Clone)]
pub struct ForEach<S: Clone + Debug> {
    id: Uuid,
    children_map: HashMap<Uuid, Box<dyn Widget<S>>>,
    delegate: Box<dyn Widget<S>>,
    ids: State<Vec<Uuid>, S>,
    position: Point,
    dimension: Dimensions,
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

/*impl<S: Clone + Debug> Event<S> for ForEach<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut S) {
        unimplemented!()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        unimplemented!()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: LocalStateList, global_state: &mut S) -> LocalStateList {
        println!("Foreach mouseevent");

        state
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: LocalStateList, global_state: &mut S) -> LocalStateList {
        // Apply state from its parent
        let new_state = self.update_widget_state(state, global_state);

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
        self.update_widget_state(state_for_children, global_state)
    }

    fn get_state(&self, mut current_state: LocalStateList) -> LocalStateList {
        unimplemented!()
    }

    fn update_widget_state(&mut self, states: LocalStateList, global_state: &S) -> LocalStateList {
        states.update_local_state(&mut self.ids, global_state);
        states
    }

    fn sync_state(&mut self, states: LocalStateList, global_state: &S) {
        unimplemented!()
    }
}*/

impl<S: Clone + Debug> NoEvents for ForEach<S> {}

impl<S: Clone + Debug> StateSync<S> for ForEach<S> {
    fn insert_local_state(&self, _env: &mut Environment) {}

    fn update_all_widget_state(&mut self, env: &Environment, _global_state: &S) {
        self.update_local_widget_state(env)
    }

    fn update_local_widget_state(&mut self, env: &Environment) {
        env.update_local_state(&mut self.ids)
    }

    fn sync_state(&mut self, env: &mut Environment, global_state: &S) {
        self.update_all_widget_state(env, global_state);

        self.insert_local_state(env);

        let mut ids = self.ids.clone();

        for (i, child) in self.get_proxied_children().enumerate() {
            env.insert_local_state(&State::<Uuid, S>::new_local("id", &ids.get_value(global_state)[i]));
            env.insert_local_state(&State::<u32, S>::new_local("index", &(i as u32)));
            child.sync_state(env, global_state)
        }

        self.update_local_widget_state(env);
    }
}

impl<S: Clone + Debug> ChildRender for ForEach<S> {}

impl<S: Clone + Debug> Layout<S> for ForEach<S> {
    fn flexibility(&self) -> u32 {
        unimplemented!()
    }

    fn calculate_size(&mut self, _requested_size: Dimensions, _env: &Environment) -> Dimensions {
        unimplemented!()
    }

    fn position_children(&mut self) {
        unimplemented!()
    }
}

impl<S: 'static + Clone + Debug> WidgetExt<S> for ForEach<S> {}