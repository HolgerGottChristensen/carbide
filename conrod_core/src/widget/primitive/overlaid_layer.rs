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
use daggy::petgraph::graph::node_index;
use crate::widget::Rectangle;
use crate::color::RED;

#[derive(Debug, Clone)]
pub struct OverlaidLayer<S> {
    id: Uuid,
    child: Box<dyn Widget<S>>,
    overlay: Option<Box<dyn Widget<S>>>,
    overlay_id: String,
    position: Point,
    dimension: Dimensions,
}

impl<S: 'static + Clone> WidgetExt<S> for OverlaidLayer<S> {}

impl<S> NoEvents for OverlaidLayer<S> {}

impl<S: Clone + 'static> StateSync<S> for OverlaidLayer<S> {
    fn insert_local_state(&self, _env: &mut Environment<S>) {}

    fn update_all_widget_state(&mut self, _env: &Environment<S>, _global_state: &S) {}

    fn update_local_widget_state(&mut self, _env: &Environment<S>) {}

    fn sync_state(&mut self, env: &mut Environment<S>, global_state: &S) {
        // This might not be the prettiest place to retrieve things from the env
        self.update_all_widget_state(env, global_state);

        self.insert_local_state(env);

        for child in self.get_proxied_children() {
            child.sync_state(env, global_state)
        }

        // Check if env contains an overlay widget with the specified id
        self.overlay = env.get_overlay(&self.overlay_id);


        self.update_local_widget_state(env);
    }
}

impl<S> Layout<S> for OverlaidLayer<S> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {
        self.dimension = self.child.calculate_size(requested_size, env);
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl<S> CommonWidget<S> for OverlaidLayer<S> {
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

impl<S> Render<S> for OverlaidLayer<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children()
            .flat_map(|f| f.get_primitives(fonts))
            .collect();
        prims.extend(children);

        if let Some(t) = &self.overlay {
            let overlay_prims = t.get_primitives(fonts);
            prims.extend(overlay_prims);
        }

        return prims;
    }
}



impl<S: Clone + 'static> OverlaidLayer<S> {

    pub fn new(child: Box<dyn Widget<S>>, overlay_id: &str) -> Box<Self> {
        Box::new(Self {
            id: Uuid::new_v4(),
            child,
            overlay: None,
            overlay_id: overlay_id.to_string(),
            position: [0.0,0.0],
            dimension: [0.0,0.0],
        })
    }
}
