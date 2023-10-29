use std::fmt::{Debug, Formatter};

use carbide_macro::{carbide_default_builder2};

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::state::{IntoReadState, ReadState, StateContract, StateSync};
use crate::widget::{AnyWidget, WidgetExt, WidgetId, Widget, CommonWidget, WidgetSequence};

/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Widget)]
#[carbide_exclude(StateSync)]
pub struct Match<T, S>
where
    T: StateContract,
    S: ReadState<T=T>,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state]
    local_state: S,
    widgets: Vec<(fn(&T) -> bool, Box<dyn AnyWidget>)>,
    current_index: Option<usize>,
}

impl Match<(), ()> {
    #[carbide_default_builder2]
    pub fn new<T: StateContract, S: IntoReadState<T>>(state: S) -> Match<T, S::Output> {
        Match {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            local_state: state.into_read_state(),
            widgets: vec![],
            current_index: None,
        }
    }
}

impl<T: StateContract, S: ReadState<T=T>> Match<T, S> {
    pub fn case(mut self, f: (fn(&T) -> bool, Box<dyn AnyWidget>)) -> Self {
        self.widgets.push(f);
        self
    }

    pub fn default(mut self, widget: Box<dyn AnyWidget>) -> Self {
        self.widgets.push((|_| true, widget));
        self
    }

    fn find_new_matching_child(&self) -> Option<usize> {
        let val = self.local_state.value();

        self.widgets.iter().position(|a| a.0(&val))
    }
}

impl<T: StateContract, S: ReadState<T=T>> StateSync for Match<T, S> {
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

impl<T: StateContract, S: ReadState<T=T>> CommonWidget for Match<T, S> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        if let Some(index) = &self.current_index {
            self.widgets[*index].1.foreach(f);
        }
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if let Some(index) = &self.current_index {
            self.widgets[*index].1.foreach_mut(f);
        }
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if let Some(index) = &self.current_index {
            self.widgets[*index].1.foreach_rev(f);
        }
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if let Some(index) = &self.current_index {
            self.widgets[*index].1.foreach_direct(f);
        }
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if let Some(index) = &self.current_index {
            self.widgets[*index].1.foreach_direct_rev(f);
        }
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        (self.position) = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        (self.dimension) = dimension
    }
}

impl<T: StateContract, S: ReadState<T=T>> WidgetExt for Match<T, S> {}

impl<T: StateContract, S: ReadState<T=T>> Debug for Match<T, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Match")
            .field("current_index", &self.current_index)
            .finish()
    }
}

#[macro_export]
macro_rules! matches_case {
    (@inner $i2:ident, $( $pattern:pat_param )|+ $( if $guard: expr )?, $next:ident) => {
        let $next = carbide::state::FieldState::new($i2.clone(), |a| {
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

        carbide::matches_case!(@inner $i2, $( $pattern )|+ $( if $guard )?, $next);
        carbide::matches_case!(@inner $i2, $( $pattern )|+ $( if $guard )?, $($rest),+);
    };
    ($i2:ident, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?, $($i1:ident),+ => $widget:expr) => {
        (|a| {
            #[allow(unused_variables)]
            match a {
                $( $pattern )|+ $( if $guard )? => true,
                _ => false
            }
        },{
            carbide::matches_case!(@inner $i2, $( $pattern )|+ $( if $guard )?, $($i1),+);

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
