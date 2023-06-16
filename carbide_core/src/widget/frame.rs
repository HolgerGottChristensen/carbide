use std::fmt::Debug;
use carbide_core::state::AnyState;


use carbide_macro::{carbide_default_builder, carbide_default_builder2};

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::state::{AnyReadState, IntoReadState, NewStateSync, ReadState, State, TState, ValueRef, ValueRefMut};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

pub static SCALE: f64 = -1.0;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Frame<X, Y, W, H, C> where
    X: ReadState<T=f64> + Clone,
    Y: ReadState<T=f64> + Clone,
    W: ReadState<T=f64> + Clone,
    H: ReadState<T=f64> + Clone,
    C: Widget + Clone
{
    id: WidgetId,
    child: C,
    position: Position,
    #[state] x: Fixity<X>,
    #[state] y: Fixity<Y>,
    #[state] width: Fixity<W>,
    #[state] height: Fixity<H>,
}

impl Frame<f64, f64, f64, f64, Empty> {
    #[carbide_default_builder2]
    pub fn new<W: IntoReadState<f64>, H: IntoReadState<f64>, C: Widget + Clone>(
        width: W,
        height: H,
        child: C,
    ) -> Box<Frame<f64, f64, W::Output, H::Output, C>> {
        Box::new(Frame {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            x: Fixity::Expand(0.0),
            y: Fixity::Expand(0.0),
            width: Fixity::Fixed(width.into_read_state()),
            height: Fixity::Fixed(height.into_read_state()),
        })
    }
}

impl<
    X: ReadState<T=f64> + Clone,
    Y: ReadState<T=f64> + Clone,
    W: ReadState<T=f64> + Clone,
    H: ReadState<T=f64> + Clone,
    C: Widget + Clone
> Frame<X, Y, W, H, C> {
    /// Note: This disconnects from the existing width value
    pub fn expand_width(mut self) -> Box<Frame<X, Y, f64, H, C>> {
        Box::new(Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            x: self.x,
            y: self.y,
            width: Fixity::Expand(10.0),
            height: self.height,
        })
    }

    /// Note: This disconnects from the existing height value
    pub fn expand_height(mut self) -> Box<Frame<X, Y, W, f64, C>> {
        Box::new(Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            x: self.x,
            y: self.y,
            width: self.width,
            height: Fixity::Expand(10.0),
        })
    }

    /// Note: This disconnects from the existing width value
    pub fn fit_width(mut self) -> Box<Frame<X, Y, f64, H, C>> {
        Box::new(Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            x: self.x,
            y: self.y,
            width: Fixity::Fit(10.0),
            height: self.height,
        })
    }

    /// Note: This disconnects from the existing height value
    pub fn fit_height(mut self) -> Box<Frame<X, Y, W, f64, C>> {
        Box::new(Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            x: self.x,
            y: self.y,
            width: self.width,
            height: Fixity::Fit(10.0),
        })
    }

    pub fn with_fixed_x<N: IntoReadState<f64>>(mut self, x: N) -> Box<Frame<N::Output, Y, W, H, C>> {
        Box::new(Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            x: Fixity::Fixed(x.into_read_state()),
            y: self.y,
            width: self.width,
            height: self.height,
        })
    }

    pub fn with_fixed_y<N: IntoReadState<f64>>(mut self, y: N) -> Box<Frame<X, N::Output, W, H, C>> {
        Box::new(Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            x: self.x,
            y: Fixity::Fixed(y.into_read_state()),
            width: self.width,
            height: self.height,
        })
    }

    pub fn with_fixed_position<N: IntoReadState<f64>, M: IntoReadState<f64>>(mut self, x: N, y: M) -> Box<Frame<N::Output, M::Output, W, H, C>> {
        Box::new(Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            x: Fixity::Fixed(x.into_read_state()),
            y: Fixity::Fixed(y.into_read_state()),
            width: self.width,
            height: self.height,
        })
    }
}

impl<
    X: ReadState<T=f64> + Clone,
    Y: ReadState<T=f64> + Clone,
    W: ReadState<T=f64> + Clone,
    H: ReadState<T=f64> + Clone,
    C: Widget + Clone
> CommonWidget for Frame<X, Y, W, H, C> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_rev(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        f(&mut self.child);
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn flexibility(&self) -> u32 {
        if let Fixity::Expand(_) = self.width {
            8
        } else if let Fixity::Expand(_) = self.height {
            8
        } else {
            9
        }
    }

    fn dimension(&self) -> Dimension {
        Dimension::new(*self.width.value(), *self.height.value())
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.width.set_value(dimension.width);
        self.height.set_value(dimension.height);
    }
}

impl<
    X: ReadState<T=f64> + Clone,
    Y: ReadState<T=f64> + Clone,
    W: ReadState<T=f64> + Clone,
    H: ReadState<T=f64> + Clone,
    C: Widget + Clone
> Layout for Frame<X, Y, W, H, C> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let fixed_height = matches!(&self.height, Fixity::Fixed(_));
        let height = *self.height.value();


        if let Fixity::Expand(_) = &self.width {
            self.width.set_value(requested_size.width);
        } else if let Fixity::Fit(_) = &mut self.width {
            let child_dimensions = if fixed_height {
                self.child.calculate_size(Dimension::new(requested_size.width, height), env)
            } else {
                self.child.calculate_size(requested_size, env)
            };
            self.width.set_value(child_dimensions.width);
        }

        let width = *self.width.value();

        if let Fixity::Expand(_) = &mut self.height {
            self.height.set_value(requested_size.height);
        } else if let Fixity::Fit(_) = &mut self.height {
            let child_dimensions = self.child.calculate_size(Dimension::new(width, requested_size.height), env);
            self.height.set_value(child_dimensions.height);
        }

        let dimensions = self.dimension();

        self.child.calculate_size(dimensions, env);

        self.dimension()
    }

    fn position_children(&mut self, env: &mut Environment) {
        if let Fixity::Fixed(_) = self.x {
            let new_x = *self.x.value();
            self.set_x(new_x);
        }

        if let Fixity::Fixed(_) = self.y {
            let new_y = *self.y.value();
            self.set_y(new_y);
        }

        let positioning = BasicLayouter::Center.positioner();
        let position = self.position;
        let dimension = Dimension::new(self.width(), self.height());

        positioning(position, dimension, &mut self.child);
        self.child.position_children(env);
    }
}

impl<
    X: ReadState<T=f64> + Clone,
    Y: ReadState<T=f64> + Clone,
    W: ReadState<T=f64> + Clone,
    H: ReadState<T=f64> + Clone,
    C: Widget + Clone
> WidgetExt for Frame<X, Y, W, H, C> {}

#[derive(Clone, Debug)]
enum Fixity<T: ReadState<T=f64> + Clone> {
    Expand(f64),
    Fit(f64),
    Fixed(T),
}

impl<T: ReadState<T=f64> + Clone> NewStateSync for Fixity<T> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        match self {
            Fixity::Expand(_) => false,
            Fixity::Fit(_) => false,
            Fixity::Fixed(s) => s.sync(env),
        }
    }
}

impl<T: ReadState<T=f64> + Clone> AnyReadState for Fixity<T> {
    type T = f64;

    fn value_dyn(&self) -> ValueRef<Self::T> {
        match self {
            Fixity::Expand(s) => ValueRef::Borrow(s),
            Fixity::Fit(s) => ValueRef::Borrow(s),
            Fixity::Fixed(s) => s.value()
        }
    }
}

impl<T: ReadState<T=f64> + Clone> AnyState for Fixity<T> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<Self::T> {
        todo!()
    }

    fn set_value_dyn(&mut self, value: Self::T) {
        match self {
            Fixity::Expand(s) => {
                *s = value;
            }
            Fixity::Fit(s) => {
                *s = value;
            }
            Fixity::Fixed(_) => unreachable!("We should never set fixed states")
        }
    }
}

