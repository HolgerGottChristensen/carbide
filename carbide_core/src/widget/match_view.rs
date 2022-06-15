use std::collections::HashMap;
use std::hash::Hash;
use crate::draw::{Dimension, Position};
use crate::prelude::*;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(StateSync)]
pub struct Match<T> where T: StateContract + Hash + PartialEq + Eq + 'static {
    id: Uuid,
    position: Position,
    dimension: Dimension,
    #[state]
    local_state: TState<T>,
    widgets: HashMap<T, Box<dyn Widget>>,
    current_key: Option<T>,
    current_child: Box<dyn Widget>,
}

impl<T: StateContract + Hash + PartialEq + Eq  + 'static> Match<T> {
    pub fn new<V: Into<TState<T>>>(state: V, widgets: Vec<(T, Box<dyn Widget>)>) -> Box<Self> {
        let mut w = HashMap::new();
        for (key, value) in widgets {
            w.insert(key, value);
        }

        Box::new(Match {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            local_state: state.into(),
            widgets: w,
            current_key: None,
            current_child: Rectangle::new().fill(EnvironmentColor::Green)
        })
    }
}

impl<T: Hash + StateContract + PartialEq + Eq  + 'static> StateSync for Match<T> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.local_state.sync(env);
        if let Some(current_key) = &self.current_key {
            if current_key == self.local_state.value().deref() { return; }
            if let Some(mut w) = self.widgets.remove(&self.local_state.value()) {
                std::mem::swap(&mut w, &mut self.current_child);
                self.widgets.insert(current_key.clone(), w);
                self.current_key = Some(self.local_state.value().clone());
            } else {
                self.current_child = Rectangle::new().fill(EnvironmentColor::Green);
                self.current_key = None;
            }
        } else {
            if let Some(w) = self.widgets.remove(&self.local_state.value()) {
                self.current_child = w;
                self.current_key = Some(self.local_state.value().clone());
            }
        }
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.local_state.sync(env);
    }
}

impl<T: Hash + StateContract + PartialEq + Eq  + 'static> carbide_core::widget::CommonWidget for Match<T> {
    fn id(&self) -> carbide_core::widget::WidgetId {
        self.id
    }

    fn set_id(&mut self, id: carbide_core::widget::WidgetId) {
        self.id = id;
    }

    fn children(&self) -> carbide_core::widget::WidgetIter {
        if (self.current_child).flag() == carbide_core::flags::Flags::PROXY {
            (self.current_child).children()
        } else if (self.current_child).flag() == carbide_core::flags::Flags::IGNORE {
            carbide_core::widget::WidgetIter::Empty
        } else {
            carbide_core::widget::WidgetIter::single(&(self.current_child))
        }
    }

    fn children_mut(&mut self) -> carbide_core::widget::WidgetIterMut {
        if (self.current_child).flag() == carbide_core::flags::Flags::PROXY {
            (self.current_child).children_mut()
        } else if (self.current_child).flag() == carbide_core::flags::Flags::IGNORE {
            carbide_core::widget::WidgetIterMut::Empty
        } else {
            carbide_core::widget::WidgetIterMut::single(&mut (self.current_child))
        }
    }

    fn children_direct(&mut self) -> carbide_core::widget::WidgetIterMut {
        carbide_core::widget::WidgetIterMut::single(&mut (self.current_child))
    }

    fn children_direct_rev(&mut self) -> carbide_core::widget::WidgetIterMut {
        carbide_core::widget::WidgetIterMut::single(&mut (self.current_child))
    }

    fn position(&self) -> carbide_core::draw::Position {
        self.position
    }

    fn set_position(&mut self, position: carbide_core::draw::Position) {
        (self.position) = position;
    }

    fn dimension(&self) -> carbide_core::draw::Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: carbide_core::draw::Dimension) {
        (self.dimension) = dimension
    }
}

//CommonWidgetImpl!(Match, self, id: self.id, child: self.current_child, position: self.position, dimension: self.dimension);

impl<T: Hash + StateContract + PartialEq + Eq + 'static> WidgetExt for Match<T> {}
