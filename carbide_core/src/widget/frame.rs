use std::fmt::Debug;
use crate::draw::{Dimension, Position};
use crate::prelude::*;

pub static SCALE: f64 = -1.0;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Frame {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    #[state] x: F64State,
    #[state] y: F64State,
    fixed_x: bool,
    fixed_y: bool,
    #[state] width: FrameState,
    #[state] height: FrameState,
}

impl Frame {
    pub fn init(
        width: impl Into<TState<f64>>,
        height: impl Into<TState<f64>>,
        child: Box<dyn Widget>,
    ) -> Box<Frame> {
        let width = width.into();
        let height = height.into();

        Box::new(Frame {
            id: Default::default(),
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

    pub fn init_width(width: impl Into<TState<f64>>, child: Box<dyn Widget>) -> Box<Frame> {
        Box::new(Frame {
            id: Default::default(),
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
            id: Default::default(),
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

    pub fn with_fixed_x(mut self, x: F64State) -> Box<Frame> {
        self.x = x;
        self.fixed_x = true;

        Box::new(self)
    }

    pub fn with_fixed_y(mut self, y: F64State) -> Box<Frame> {
        self.y = y;
        self.fixed_y = true;

        Box::new(self)
    }

    pub fn with_fixed_position(mut self, x: F64State, y: F64State) -> Box<Frame> {
        self.x = x;
        self.fixed_x = true;
        self.y = y;
        self.fixed_y = true;

        Box::new(self)
    }
}

impl CommonWidget for Frame {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
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
        if let FrameState::Expand(e) = &mut self.width {
            e.set_value(requested_size.width);
        }

        if let FrameState::Expand(e) = &mut self.height {
            e.set_value(requested_size.height);
        }

        let dimensions = self.dimension();

        self.child.calculate_size(dimensions, env);

        self.dimension()
    }

    fn position_children(&mut self) {
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
        self.child.position_children();
    }
}

impl WidgetExt for Frame {}

#[derive(Clone, Debug)]
enum FrameState {
    Expand(TState<f64>),
    Fixed(TState<f64>)
}

impl NewStateSync for FrameState {
    fn sync(&mut self, env: &mut Environment) -> bool {
        match self {
            FrameState::Expand(e) => {
                e.sync(env)
            }
            FrameState::Fixed(f) => {
                f.sync(env)
            }
        }
    }
}


impl ReadState<f64> for FrameState {
    fn value(&self) -> ValueRef<f64> {
        match self {
            FrameState::Expand(e) => {
                e.value()
            }
            FrameState::Fixed(f) => {
                f.value()
            }
        }
    }
}

impl State<f64> for FrameState {
    fn value_mut(&mut self) -> ValueRefMut<f64> {
        unimplemented!("Should not be called")
    }

    fn set_value(&mut self, value: f64) {
        match self {
            FrameState::Expand(e) => {
                e.set_value(value)
            }
            FrameState::Fixed(f) => {
                f.set_value(value)
            }
        }
    }

}