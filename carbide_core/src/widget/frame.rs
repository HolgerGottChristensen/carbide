use std::fmt::Debug;


use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::state::{NewStateSync, ReadState, State, TState, ValueRef, ValueRefMut};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

pub static SCALE: f64 = -1.0;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Frame {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    #[state]
    x: TState<f64>,
    #[state]
    y: TState<f64>,
    fixed_x: bool,
    fixed_y: bool,
    #[state]
    width: FrameState,
    #[state]
    height: FrameState,
}

impl Frame {
    #[carbide_default_builder]
    pub fn new(
        width: impl Into<TState<f64>>,
        height: impl Into<TState<f64>>,
        child: Box<dyn Widget>,
    ) -> Box<Frame> {}

    pub fn new(
        width: impl Into<TState<f64>>,
        height: impl Into<TState<f64>>,
        child: Box<dyn Widget>,
    ) -> Box<Frame> {
        let width = width.into();
        let height = height.into();

        Box::new(Frame {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            x: 0.0.into(),
            y: 0.0.into(),
            fixed_x: false,
            fixed_y: false,
            width: FrameState::Fixed(width),
            height: FrameState::Fixed(height),
        })
    }

    pub fn expand_width(mut self) -> Box<Frame> {
        self.width = FrameState::Expand(10.0.into());
        Box::new(self)
    }

    pub fn expand_height(mut self) -> Box<Frame> {
        self.height = FrameState::Expand(10.0.into());
        Box::new(self)
    }

    pub fn fit_width(mut self) -> Box<Frame> {
        self.width = FrameState::Fit(10.0.into());
        Box::new(self)
    }

    pub fn fit_height(mut self) -> Box<Frame> {
        self.height = FrameState::Fit(10.0.into());
        Box::new(self)
    }

    pub fn init_width(width: impl Into<TState<f64>>, child: Box<dyn Widget>) -> Box<Frame> {
        Box::new(Frame {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            x: 0.0.into(),
            y: 0.0.into(),
            fixed_x: false,
            fixed_y: false,
            width: FrameState::Fixed(width.into()),
            height: FrameState::Expand(0.0.into()),
        })
    }

    pub fn init_height(height: impl Into<TState<f64>>, child: Box<dyn Widget>) -> Box<Frame> {
        Box::new(Frame {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            x: 0.0.into(),
            y: 0.0.into(),
            fixed_x: false,
            fixed_y: false,
            width: FrameState::Expand(0.0.into()),
            height: FrameState::Fixed(height.into()),
        })
    }

    pub fn with_fixed_x(mut self, x: TState<f64>) -> Box<Frame> {
        self.x = x;
        self.fixed_x = true;

        Box::new(self)
    }

    pub fn with_fixed_y(mut self, y: TState<f64>) -> Box<Frame> {
        self.y = y;
        self.fixed_y = true;

        Box::new(self)
    }

    pub fn with_fixed_position(mut self, x: TState<f64>, y: TState<f64>) -> Box<Frame> {
        self.x = x;
        self.fixed_x = true;
        self.y = y;
        self.fixed_y = true;

        Box::new(self)
    }
}

impl CommonWidget for Frame {
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
        if let FrameState::Expand(_) = self.width {
            8
        } else if let FrameState::Expand(_) = self.height {
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

impl Layout for Frame {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let fixed_height = matches!(&self.height, FrameState::Fixed(_));

        let height = *self.height.value();

        if let FrameState::Expand(e) = &mut self.width {
            e.set_value(requested_size.width);
        } else if let FrameState::Fit(f) = &mut self.width {
            let child_dimensions = if fixed_height {
                self.child
                    .calculate_size(Dimension::new(requested_size.width, height), env)
            } else {
                self.child.calculate_size(requested_size, env)
            };
            f.set_value(child_dimensions.width);
        }

        let width = *self.width.value();

        if let FrameState::Expand(e) = &mut self.height {
            e.set_value(requested_size.height);
        } else if let FrameState::Fit(f) = &mut self.height {
            let child_dimensions = self
                .child
                .calculate_size(Dimension::new(width, requested_size.height), env);
            f.set_value(child_dimensions.height);
        }

        let dimensions = self.dimension();

        self.child.calculate_size(dimensions, env);

        self.dimension()
    }

    fn position_children(&mut self, env: &mut Environment) {
        if self.fixed_x {
            let new_x = *self.x.value();
            self.set_x(new_x);
        }

        if self.fixed_y {
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

impl WidgetExt for Frame {}

#[derive(Clone, Debug)]
enum FrameState {
    Expand(TState<f64>),
    Fit(TState<f64>),
    Fixed(TState<f64>),
}

impl NewStateSync for FrameState {
    fn sync(&mut self, env: &mut Environment) -> bool {
        match self {
            FrameState::Expand(e) => e.sync(env),
            FrameState::Fixed(f) => f.sync(env),
            FrameState::Fit(f) => f.sync(env),
        }
    }
}

impl ReadState for FrameState {
    type T = f64;

    fn value(&self) -> ValueRef<f64> {
        match self {
            FrameState::Expand(e) => e.value(),
            FrameState::Fixed(f) => f.value(),
            FrameState::Fit(f) => f.value(),
        }
    }
}

impl State for FrameState {
    fn value_mut(&mut self) -> ValueRefMut<f64> {
        unimplemented!("Should not be called")
    }

    fn set_value(&mut self, value: f64) {
        match self {
            FrameState::Expand(e) => e.set_value(value),
            FrameState::Fixed(f) => f.set_value(value),
            FrameState::Fit(f) => f.set_value(value),
        }
    }
}
