use crate::draw::{Dimension, Position};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use carbide_macro::carbide_default_builder;
use crate::environment::Environment;
use crate::state::{NewStateSync, ReadState, StateContract, StateSync, TState};
use crate::widget::{Widget, WidgetExt, WidgetId};

/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Widget)]
#[carbide_exclude(StateSync)]
pub struct Match<T>
where
    T: StateContract,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state]
    local_state: TState<T>,
    widgets: Vec<(fn(&T) -> bool, Box<dyn Widget>)>,
    current_index: Option<usize>,
}

impl<T: StateContract> Match<T> {
    #[carbide_default_builder]
    pub fn new(state: impl Into<TState<T>>) -> Box<Self> {
        Box::new(Match {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            local_state: state.into(),
            widgets: vec![],
            current_index: None,
        })
    }

    pub fn case(mut self, f: (fn(&T) -> bool, Box<dyn Widget>)) -> Box<Self> {
        self.widgets.push(f);
        Box::new(self)
    }

    pub fn default(mut self, widget: Box<dyn Widget>) -> Box<Self> {
        self.widgets.push((|_| true, widget));
        Box::new(self)
    }

    fn find_new_matching_child(&self) -> Option<usize> {
        let val = self.local_state.value();

        self.widgets.iter().position(|a| a.0(&val))
    }
}

impl<T: StateContract> StateSync for Match<T> {
    fn capture_state(&mut self, env: &mut Environment) {
        self.local_state.sync(env);

        // Always find the first match in the vec
        self.current_index = self.find_new_matching_child();

        // With the below code we match in the order of the vec and stay at an item as long as it
        // matches. I dont know if this is desirable. I think it is more efficient, because we dont
        // have to iterate the vec all the times.

        /*if let Some(index) = self.current_index {
            if self.widgets[index].0(&self.local_state.value()) {
                // If we match the current case still we are happy
            } else {
                self.current_index = self.find_new_matching_child();
            }
        } else {
            self.current_index = self.find_new_matching_child();
        }*/
    }

    fn release_state(&mut self, env: &mut Environment) {
        self.local_state.sync(env);
    }
}

impl<T: StateContract> carbide_core::widget::CommonWidget for Match<T> {
    fn id(&self) -> carbide_core::widget::WidgetId {
        self.id
    }

    fn children(&self) -> carbide_core::widget::WidgetIter {
        if let Some(index) = self.current_index {
            if (self.widgets[index].1).flag() == carbide_core::flags::Flags::PROXY {
                (self.widgets[index].1).children()
            } else if (self.widgets[index].1).flag() == carbide_core::flags::Flags::IGNORE {
                carbide_core::widget::WidgetIter::Empty
            } else {
                carbide_core::widget::WidgetIter::single(&(self.widgets[index].1))
            }
        } else {
            carbide_core::widget::WidgetIter::Empty
        }
    }

    fn children_mut(&mut self) -> carbide_core::widget::WidgetIterMut {
        if let Some(index) = self.current_index {
            if (self.widgets[index].1).flag() == carbide_core::flags::Flags::PROXY {
                (self.widgets[index].1).children_mut()
            } else if (self.widgets[index].1).flag() == carbide_core::flags::Flags::IGNORE {
                carbide_core::widget::WidgetIterMut::Empty
            } else {
                carbide_core::widget::WidgetIterMut::single(&mut (self.widgets[index].1))
            }
        } else {
            carbide_core::widget::WidgetIterMut::Empty
        }
    }

    fn children_direct(&mut self) -> carbide_core::widget::WidgetIterMut {
        if let Some(index) = self.current_index {
            carbide_core::widget::WidgetIterMut::single(&mut (self.widgets[index].1))
        } else {
            carbide_core::widget::WidgetIterMut::Empty
        }
    }

    fn children_direct_rev(&mut self) -> carbide_core::widget::WidgetIterMut {
        if let Some(index) = self.current_index {
            carbide_core::widget::WidgetIterMut::single(&mut (self.widgets[index].1))
        } else {
            carbide_core::widget::WidgetIterMut::Empty
        }
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

impl<T: StateContract> WidgetExt for Match<T> {}

impl<T: StateContract> Debug for Match<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Match")
            .field("current_index", &self.current_index)
            .finish()
    }
}

#[macro_export]
macro_rules! matches_case {
    (@inner $i2:ident, $( $pattern:pat_param )|+ $( if $guard: expr )?, $next:ident) => {
        let $next = carbide_core::state::FieldState::new2($i2.clone(), |a| {
            #[allow(unused_variables)]
            match a {
                $( $pattern )|+ $( if $guard )? => {
                    $next
                }
                _ => panic!("Not matching: &{}", stringify!{$next})
            }
        }, |b| {
            #[allow(unused_variables)]
            match b {
                $( $pattern )|+ $( if $guard )? => {
                    $next
                }
                _ => panic!("Not matching: &mut {}", stringify!{$next})
            }
        });
    };
    (@inner $i2:ident, $( $pattern:pat_param )|+ $( if $guard: expr )?, $next:ident, $($rest:ident),+) => {

        carbide_core::matches_case!(@inner $i2, $( $pattern )|+ $( if $guard )?, $next);
        carbide_core::matches_case!(@inner $i2, $( $pattern )|+ $( if $guard )?, $($rest),+);
    };
    ($i2:ident, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?, $($i1:ident),+ => $widget:expr) => {
        (|a| {
            #[allow(unused_variables)]
            match a {
                $( $pattern )|+ $( if $guard )? => true,
                _ => false
            }
        },{
            carbide_core::matches_case!(@inner $i2, $( $pattern )|+ $( if $guard )?, $($i1),+);

            $widget
        })
    };
    ($i2:ident, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?, $widget:expr) => {
        (|a| {
            #[allow(unused_variables)]
            match a {
                $( $pattern )|+ $( if $guard )? => true,
                _ => false
            }
        },{
            $widget
        })
    }
}
