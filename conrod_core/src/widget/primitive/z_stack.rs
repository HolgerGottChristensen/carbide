use uuid::Uuid;

use crate::{Color, Colorable, Point, Rect, Sizeable};
use crate::text;
use crate::flags::Flags;
use crate::layout::Layout;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::state::environment::Environment;
use crate::state::state_sync::NoLocalStateSync;
use crate::widget::Rectangle;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::Render;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::state::global_state::GlobalState;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct ZStack<GS> where GS: GlobalState {
    id: Uuid,
    children: Vec<Box<dyn Widget<GS>>>,
    position: Point,
    dimension: Dimensions,
}

impl<S: GlobalState> ZStack<S> {
    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<ZStack<S>> {
        Box::new(ZStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0]
        })
    }
}

impl<S: GlobalState> Layout<S> for ZStack<S> {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {

        let mut children_flexibilty: Vec<(u32, &mut Box<dyn Widget<S>>)> = self.get_children_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_width = 0.0;
        let mut max_height = 0.0;

        for (_, child) in children_flexibilty {
            let chosen_size = child.calculate_size(requested_size, env);

            if chosen_size[0] > max_width {
                max_width = chosen_size[0];
            }

            if chosen_size[1] > max_height {
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

impl<S: GlobalState> CommonWidget<S> for ZStack<S> {
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

impl<S: GlobalState> Render<S> for ZStack<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 0.5));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}


