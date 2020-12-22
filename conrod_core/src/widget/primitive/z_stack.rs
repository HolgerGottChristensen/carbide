use std::any::Any;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;

use daggy::petgraph::graph::node_index;
use uuid::Uuid;

use {Color, Colorable, Point, Rect, Sizeable};
use ::{Scalar, widget};
use ::{Range, text};
use color::rgb;
use event::event::{Event, NoEvents};
use event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use flags::Flags;
use graph::Container;
use layout::{CrossAxisAlignment, Layout};
use layout::basic_layouter::BasicLayouter;
use layout::layouter::Layouter;
use position::Dimensions;
use render::owned_primitive::OwnedPrimitive;
use render::owned_primitive_kind::OwnedPrimitiveKind;
use render::primitive::Primitive;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use state::environment::Environment;
use state::state::LocalStateList;
use state::state_sync::NoLocalStateSync;
use text::font::Map;
use widget::{Id, Rectangle};
use widget::common_widget::CommonWidget;
use widget::primitive::Widget;
use widget::primitive::widget::WidgetExt;
use widget::render::Render;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone)]
pub struct ZStack<S> {
    id: Uuid,
    children: Vec<Box<dyn Widget<S>>>,
    position: Point,
    dimension: Dimensions
}

impl<S> ZStack<S> {
    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<ZStack<S>> {
        Box::new(ZStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0]
        })
    }
}

impl<S> NoEvents for ZStack<S> {}

impl<S> NoLocalStateSync for ZStack<S> {}

impl<S: 'static + Clone> WidgetExt<S> for ZStack<S> {}

impl<S> Layout<S> for ZStack<S> {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment) -> Dimensions {

        let mut children_flexibilty: Vec<(u32, &mut Box<dyn Widget<S>>)> = self.get_children_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_width = 0.0;
        let mut max_height = 0.0;

        for (_, child) in children_flexibilty {
            let chosen_size = child.calculate_size(requested_size, env);

            if (chosen_size[0] > max_width) {
                max_width = chosen_size[0];
            }

            if (chosen_size[1] > max_height) {
                max_height = chosen_size[1];
            }

        }

        self.dimension = [max_width, max_height];
        self.dimension

    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::TopLeading.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in self.get_children_mut() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl<S> CommonWidget<S> for ZStack<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<S> {
        self.children
            .iter()
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.get_flag() == Flags::Proxy {
                    WidgetIter::Multi(Box::new(x.get_children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        self.children
            .iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.get_flag() == Flags::Proxy {
                    WidgetIterMut::Multi(Box::new(x.get_children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        self.children.iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<S> Render<S> for ZStack<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::rect_outline(Rect::new(self.position, self.dimension), 0.5));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}


