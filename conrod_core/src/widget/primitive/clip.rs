//! A simple, non-interactive rectangle shape widget.
//!
//! Due to the frequency of its use in GUIs, the `Rectangle` gets its own widget to allow backends
//! to specialise their rendering implementations.






use daggy::petgraph::graph::node_index;
use uuid::Uuid;

use crate::{Color, Colorable, Point, Rect, Sizeable};
use crate::{Scalar, widget};
use crate::text;
use crate::draw::shape::triangle::Triangle;
use crate::event::event::NoEvents;
use crate::flags::Flags;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::Layout;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::render::primitive_kind::PrimitiveKind;
use crate::state::environment::Environment;
use crate::state::state_sync::{NoLocalStateSync, StateSync};
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::Render;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::widget::Rectangle;

use conrod_derive::Widget;
use crate::state::state::State;
use crate::state::global_state::GlobalState;

#[derive(Debug, Clone, Widget)]
#[state_sync(sync_state)]
pub struct Clip<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
}

impl<S: GlobalState> NoEvents for Clip<S> {}

impl<S: GlobalState> Layout<S> for Clip<S> {
    fn flexibility(&self) -> u32 {
        self.child.flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {
        self.child.calculate_size(requested_size, env);
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);

        self.child.position_children();
    }
}

impl<S: GlobalState> CommonWidget<S> for Clip<S> {
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
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<S: GlobalState> Render<S> for Clip<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Clip,
                scizzor: Rect::new(self.position, self.dimension),
                rect: Rect::new(self.position, self.dimension)
            }
        ];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        prims.push(Primitive {
            id: node_index(0),
            kind: PrimitiveKind::UnClip,
            scizzor: Rect::new(self.position, self.dimension),
            rect: Rect::new(self.position, self.dimension)
        });

        return prims;
    }
}


impl<S: GlobalState> Clip<S> {
    pub fn new(child: Box<dyn Widget<S>>) -> Box<Self<>> {
        Box::new(Clip {
            id: Uuid::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
        })
    }
}
