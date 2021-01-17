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
use crate::state::global_state::GlobalState;

#[derive(Debug, Clone)]
pub struct ForEach<S: GlobalState> {
    id: Uuid, // --
    children_map: HashMap<Uuid, Box<dyn Widget<S>>>,
    delegate: Box<dyn Widget<S>>,
    ids: State<Vec<Uuid>, S>,
    position: Point, // --
    dimension: Dimensions, // --
}

impl<S: GlobalState> Widget<S> for ForEach<S> {}

impl<S: GlobalState> ForEach<S> {
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

impl<S: GlobalState> CommonWidget<S> for ForEach<S> {
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

impl<S: GlobalState> NoEvents for ForEach<S> {}

impl<S: GlobalState> StateSync<S> for ForEach<S> {
    fn insert_local_state(&self, _env: &mut Environment<S>) {}

    fn update_all_widget_state(&mut self, env: &Environment<S>, _global_state: &S) {
        self.update_local_widget_state(env)
    }

    fn update_local_widget_state(&mut self, env: &Environment<S>) {
        env.update_local_state(&mut self.ids)
    }

    fn sync_state(&mut self, env: &mut Environment<S>, global_state: &S) {
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

impl<S: GlobalState> ChildRender for ForEach<S> {}

impl<S: GlobalState> Layout<S> for ForEach<S> {
    fn flexibility(&self) -> u32 {
        unimplemented!()
    }

    fn calculate_size(&mut self, _requested_size: Dimensions, _env: &Environment<S>) -> Dimensions {
        unimplemented!()
    }

    fn position_children(&mut self) {
        unimplemented!()
    }
}

impl<S: GlobalState> WidgetExt<S> for ForEach<S> {}