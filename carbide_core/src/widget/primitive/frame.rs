use crate::prelude::*;

pub static SCALE: f64 = -1.0;

#[derive(Debug, Clone, Widget)]
pub struct Frame<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    #[state] width: Box<dyn State<f64, GS>>,
    #[state] height: Box<dyn State<f64, GS>>,
    expand_width: bool,
    expand_height: bool,
}

impl<GS: GlobalState> WidgetExt<GS> for Frame<GS> {}

impl<GS: GlobalState> Frame<GS> {
    pub fn init(width: Box<dyn State<f64, GS>>, height: Box<dyn State<f64, GS>>, child: Box<dyn Widget<GS>>) -> Box<Frame<GS>> {

        let expand_width = *width.get_latest_value() == SCALE;

        let expand_height = *height.get_latest_value() == SCALE;

        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            width: width.into(),
            height: height.into(),
            expand_width,
            expand_height,
        })
    }

    pub fn init_width(width: Box<dyn State<f64, GS>>, child: Box<dyn Widget<GS>>) -> Box<Frame<GS>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            width,
            height: 0.0.into(),
            expand_width: false,
            expand_height: true
        })
    }

    pub fn init_height(height: Box<dyn State<f64, GS>>, child: Box<dyn Widget<GS>>) -> Box<Frame<GS>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            width: 0.0.into(),
            height,
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
        [*self.width.get_latest_value(), *self.height.get_latest_value()]
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        *self.width.get_latest_value_mut() = dimensions[0];
        *self.height.get_latest_value_mut() = dimensions[1];
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

        let dimensions = self.get_dimension();

        self.child.calculate_size(dimensions, env);

        self.get_dimension()
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = [self.get_width(), self.get_height()];


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl<S: GlobalState> Render<S> for Frame<S> {

    fn get_primitives(&mut self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, [self.get_width(), self.get_height()]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(fonts);
        prims.extend(children);

        return prims;
    }
}