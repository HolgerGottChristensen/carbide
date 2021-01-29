use crate::prelude::*;

pub static SCALE: f64 = -1.0;

#[derive(Debug, Clone, Widget)]
pub struct Frame<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    expand_width: bool,
    expand_height: bool,
}

impl<GS: GlobalState> WidgetExt<GS> for Frame<GS> {}

impl<S: GlobalState> Frame<S> {
    pub fn init(width: Scalar, height: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {

        let expand_width = width == SCALE;
        let expand_height = height == SCALE;

        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, height],
            expand_width,
            expand_height,
        })
    }

    pub fn init_width(width: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, 0.0],
            expand_width: false,
            expand_height: true
        })
    }

    pub fn init_height(height: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [0.0, height],
            expand_width: true,
            expand_height: false
        })
    }
}

impl<S: GlobalState> CommonWidget<S> for Frame<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::single(&mut self.child)
    }


    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        [self.dimension[0].abs(), self.dimension[1].abs()]
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<S: GlobalState> Layout<S> for Frame<S> {
    fn flexibility(&self) -> u32 {
        if self.expand_width || self.expand_height {
            8
        } else {
            9
        }
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {

        if self.expand_width {
            self.set_width(requested_size[0]);
        }

        if self.expand_height {
            self.set_height(requested_size[1]);
        }

        let dimensions = self.dimension;

        self.child.calculate_size(dimensions, env);

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = [self.dimension[0].abs(), self.dimension[1].abs()];


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl<S: GlobalState> Render<S> for Frame<S> {

    fn get_primitives(&mut self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, [self.dimension[0].abs(), self.dimension[1].abs()]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(fonts);
        prims.extend(children);

        return prims;
    }
}