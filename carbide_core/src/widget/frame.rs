use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::render::ChildRender;

pub static SCALE: f64 = -1.0;

#[derive(Debug, Clone, Widget)]
pub struct Frame {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    #[state] x: F64State,
    #[state] y: F64State,
    fixed_x: bool,
    fixed_y: bool,
    #[state] width: F64State,
    #[state] height: F64State,
    expand_width: bool,
    expand_height: bool,
}

impl Frame {
    pub fn init<W: Into<F64State>, H: Into<F64State>>(width: W, height: H, child: Box<dyn Widget>) -> Box<Frame> {
        let width = width.into();
        let height = height.into();
        let expand_width = *width.value() == SCALE;

        let expand_height = *height.value() == SCALE;

        Box::new(Frame {
            id: Default::default(),
            child: Box::new(child),
            position: Position::new(0.0, 0.0),
            x: 0.0.into(),
            y: 0.0.into(),
            fixed_x: false,
            fixed_y: false,
            width: width.into(),
            height: height.into(),
            expand_width,
            expand_height,
        })
    }

    pub fn init_width(width: F64State, child: Box<dyn Widget>) -> Box<Frame> {
        Box::new(Frame {
            id: Default::default(),
            child: Box::new(child),
            position: Position::new(0.0, 0.0),
            x: 0.0.into(),
            y: 0.0.into(),
            fixed_x: false,
            fixed_y: false,
            width,
            height: 0.0.into(),
            expand_width: false,
            expand_height: true,
        })
    }

    pub fn init_height(height: F64State, child: Box<dyn Widget>) -> Box<Frame> {
        Box::new(Frame {
            id: Default::default(),
            child: Box::new(child),
            position: Position::new(0.0, 0.0),
            x: 0.0.into(),
            y: 0.0.into(),
            fixed_x: false,
            fixed_y: false,
            width: 0.0.into(),
            height,
            expand_width: true,
            expand_height: false,
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
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(self.child.deref())
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(self.child.deref_mut())
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn get_position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimension {
        Dimension::new(*self.width.value(), *self.height.value())
    }

    fn set_dimension(&mut self, dimensions: Dimension) {
        *self.width.value_mut() = dimensions.width;
        *self.height.value_mut() = dimensions.height;
    }
}

impl Layout for Frame {
    fn flexibility(&self) -> u32 {
        if self.expand_width || self.expand_height {
            8
        } else {
            9
        }
    }

    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        if self.expand_width {
            self.set_width(requested_size.width);
        }

        if self.expand_height {
            self.set_height(requested_size.height);
        }

        let dimensions = self.get_dimension();

        self.child.calculate_size(dimensions, env);

        self.get_dimension()
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

        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = Dimension::new(self.get_width(), self.get_height());


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl ChildRender for Frame {}

impl WidgetExt for Frame {}