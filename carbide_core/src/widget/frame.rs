use std::fmt::Debug;
use carbide_macro::carbide_default_builder2;

use crate::draw::{Alignment, Dimension, Position};
use crate::environment::{Environment};
use crate::layout::{Layout, LayoutContext};
use crate::state::{AnyReadState, AnyState, IntoState, StateSync, ReadState, State, ValueRef, ValueRefMut};
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Frame<W, H, C> where
    W: State<T=f64>,
    H: State<T=f64>,
    C: Widget
{
    #[id] id: WidgetId,
    child: C,
    position: Position,
    #[state] width: Fixity<W>,
    #[state] height: Fixity<H>,
}

impl Frame<f64, f64, Empty> {
    #[carbide_default_builder2]
    pub fn new<W: IntoState<f64>, H: IntoState<f64>, C: Widget>(
        width: W,
        height: H,
        child: C,
    ) -> Frame<W::Output, H::Output, C> {
        Frame {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            width: Fixity::Fixed(width.into_state()),
            height: Fixity::Fixed(height.into_state()),
        }
    }
}

impl<W: State<T=f64>, H: State<T=f64>, C: Widget> Frame<W, H, C> {
    /// Note: This disconnects from the existing width value
    pub fn expand_width(self) -> Frame<f64, H, C> {
        Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            width: Fixity::Expand(10.0),
            height: self.height,
        }
    }

    /// Note: This disconnects from the existing height value
    pub fn expand_height(self) -> Frame<W, f64, C> {
        Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            width: self.width,
            height: Fixity::Expand(10.0),
        }
    }

    /// Note: This disconnects from the existing width value
    pub fn fit_width(self) -> Frame<f64, H, C> {
        Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            width: Fixity::Fit(10.0),
            height: self.height,
        }
    }

    /// Note: This disconnects from the existing height value
    pub fn fit_height(self) -> Frame<W, f64, C> {
        Frame {
            id: self.id,
            child: self.child,
            position: self.position,
            width: self.width,
            height: Fixity::Fit(10.0),
        }
    }
}

impl<W: State<T=f64>, H: State<T=f64>, C: Widget> CommonWidget for Frame<W, H, C> {
    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_rev(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
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

impl<W: State<T=f64>, H: State<T=f64>, C: Widget> Layout for Frame<W, H, C> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        let fixed_height = matches!(&self.height, Fixity::Fixed(_));
        let height = *self.height.value();


        if let Fixity::Expand(_) = &self.width {
            self.width.set_value(requested_size.width);
        } else if let Fixity::Fit(_) = &mut self.width {
            let child_dimensions = if fixed_height {
                self.child.calculate_size(Dimension::new(requested_size.width, height), ctx)
            } else {
                self.child.calculate_size(requested_size, ctx)
            };
            self.width.set_value(child_dimensions.width);
        }

        let width = *self.width.value();

        if let Fixity::Expand(_) = &mut self.height {
            self.height.set_value(requested_size.height);
        } else if let Fixity::Fit(_) = &mut self.height {
            let child_dimensions = self.child.calculate_size(Dimension::new(width, requested_size.height), ctx);
            self.height.set_value(child_dimensions.height);
        }

        let dimensions = self.dimension();

        self.child.calculate_size(dimensions, ctx);

        self.dimension()
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let position = self.position;
        let dimension = Dimension::new(self.width(), self.height());

        self.child.set_position(Alignment::Center.position(position, dimension, self.child.dimension()));
        self.child.position_children(ctx);
    }
}

#[derive(Clone, Debug)]
enum Fixity<T: State<T=f64>> {
    Expand(f64),
    Fit(f64),
    Fixed(T),
}

impl<T: State<T=f64> + Clone> StateSync for Fixity<T> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        match self {
            Fixity::Expand(_) => false,
            Fixity::Fit(_) => false,
            Fixity::Fixed(s) => s.sync(env),
        }
    }
}

impl<T: State<T=f64> + Clone> AnyReadState for Fixity<T> {
    type T = f64;

    fn value_dyn(&self) -> ValueRef<Self::T> {
        match self {
            Fixity::Expand(s) => ValueRef::Borrow(s),
            Fixity::Fit(s) => ValueRef::Borrow(s),
            Fixity::Fixed(s) => s.value()
        }
    }
}

impl<T: State<T=f64> + Clone> AnyState for Fixity<T> {
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
            Fixity::Fixed(s) => s.set_value(value)
        }
    }
}

