use std::ops::Neg;

use uuid::Uuid;

use ::{Point, Scalar};
use ::{Rect, text};
use event::event::{Event, NoEvents};
use event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use flags::Flags;
use graph::Container;
use layout::basic_layouter::BasicLayouter;
use layout::Layout;
use layout::layouter::Layouter;
use position::Dimensions;
use render::primitive::Primitive;
use state::environment::Environment;
use state::state::LocalStateList;
use state::state_sync::{NoLocalStateSync, StateSync};
use text::font::Map;
use widget::{Id, Rectangle};
use widget::common_widget::CommonWidget;
use widget::primitive::Widget;
use widget::primitive::widget::WidgetExt;
use widget::render::Render;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};

pub static SCALE: f64 = -1.0;


#[derive(Debug, Clone)]
pub struct Frame<S> {
    id: Uuid,
    child: Box<dyn Widget<S>>,
    position: Point,
    dimension: Dimensions
}

impl<S: 'static> Frame<S> {
    pub fn init(width: Scalar, height: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, height]
        })
    }

    pub fn init_width(width: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, -1.0]
        })
    }

    pub fn init_height(height: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [-1.0, height]
        })
    }
}

impl<S: 'static + Clone> WidgetExt<S> for Frame<S> {}

impl<S> NoEvents for Frame<S> {}

impl<S> NoLocalStateSync for Frame<S> {}

impl<S> CommonWidget<S> for Frame<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<S> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::Proxy {
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

impl<S> Layout<S> for Frame<S> {
    fn flexibility(&self) -> u32 {
        9
    }

    fn calculate_size(&mut self, dimension: Dimensions, env: &Environment) -> Dimensions {
        let dimensions = self.dimension;
        let mut abs_dimensions = match (dimensions[0], dimensions[1]) {
            (x, y) if x < 0.0 && y < 0.0 => [dimension[0], dimension[1]],
            (x, y) if x < 0.0 => [dimension[0], self.dimension[1]],
            (x, y) if y < 0.0 => [self.dimension[0], dimension[1]],
            (x, y) => [x, y]
        };

        let child_dimensions = self.child.calculate_size(abs_dimensions, env);

        if dimensions[0] < 0.0 {
            self.dimension = [child_dimensions[0].abs().neg(), dimensions[1]]
        }

        if dimensions[1] < 0.0 {
            self.dimension = [self.dimension[0], child_dimensions[1].abs().neg()]
        }

        [self.dimension[0].abs(), self.dimension[1].abs()]
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = [self.dimension[0].abs(), self.dimension[1].abs()];


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl<S> Render<S> for Frame<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::rect_outline(Rect::new(self.position, [self.dimension[0].abs(), self.dimension[1].abs()]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(fonts);
        prims.extend(children);

        return prims;
    }
}