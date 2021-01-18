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
use crate::event::event::Event;
use crate::flags::Flags;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::Layout;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::render::primitive_kind::PrimitiveKind;
use crate::state::environment::Environment;
use crate::state::state_sync::NoLocalStateSync;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::Render;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use crate::widget::Rectangle;
use crate::widget::types::scroll_direction::ScrollDirection;
use crate::color::GRAY;
use crate::draw::shape::vertex::Vertex;
use crate::state::global_state::GlobalState;


/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[event(handle_mouse_event, handle_other_event)]
pub struct Scroll<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    scroll_offset: [f64; 2], // Save as scroll percents instead of offsets
    scroll_directions: ScrollDirection,
    scrollbar_horizontal: Box<dyn Widget<GS>>,
    scrollbar_vertical: Box<dyn Widget<GS>>,
}

impl<S: GlobalState> Scroll<S> {

    pub fn set_scroll_direction(mut self, scroll_directions: ScrollDirection) -> Box<Self> {
        self.scroll_directions = scroll_directions;
        Box::new(self)
    }

    fn keep_y_within_bounds(&mut self) -> () {
        if self.scroll_offset[1] > 0.0 {
            self.scroll_offset = [self.scroll_offset[0], 0.0];
        }

        if self.child.get_height() > self.get_height() {
            if self.scroll_offset[1] < -(self.child.get_height() - self.get_height()) {
                self.scroll_offset = [self.scroll_offset[0], -(self.child.get_height() - self.get_height())];
            }
        } else {
            self.scroll_offset = [self.scroll_offset[0], 0.0];
        }
    }

    fn keep_x_within_bounds(&mut self) {
        if self.scroll_offset[0] < 0.0 {
            self.scroll_offset = [0.0, self.scroll_offset[1]];
        }

        if self.child.get_width() > self.get_width() {
            if self.scroll_offset[0] > (self.child.get_width() - self.get_width()) {
                self.scroll_offset = [(self.child.get_width() - self.get_width()), self.scroll_offset[1]];
            }
        } else {
            self.scroll_offset = [0.0, self.scroll_offset[1]];
        }
    }

    pub fn new(child: Box<dyn Widget<S>>) -> Box<Self> {
        Box::new(Self {
            id: Uuid::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            scroll_offset: [0.0, 0.0],
            scroll_directions: ScrollDirection::Both,
            scrollbar_horizontal: Rectangle::initialize(vec![]).fill(GRAY).frame(100.0,10.0),
            scrollbar_vertical: Rectangle::initialize(vec![]).fill(GRAY).frame(10.0,100.0)
        })
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut S) {
        match event {
            MouseEvent::Scroll { x, y, mouse_position, modifiers } => {

                if self.scroll_directions == ScrollDirection::Both ||
                    self.scroll_directions == ScrollDirection::Vertical {



                    if modifiers.contains(piston_input::keyboard::ModifierKey::SHIFT) {
                        self.scroll_offset[1] -= x;
                    } else {
                        self.scroll_offset[1] -= y;
                    }

                    self.keep_y_within_bounds();

                }

                if self.scroll_directions == ScrollDirection::Both ||
                    self.scroll_directions == ScrollDirection::Horizontal {


                    if modifiers.contains(piston_input::keyboard::ModifierKey::SHIFT) {
                        self.scroll_offset[0] += y;
                    } else {
                        self.scroll_offset[0] -= x;
                    }

                    self.keep_x_within_bounds();

                }

            }
            _ => {}
        }
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        match event {
            WidgetEvent::Window(_) => {
                self.keep_y_within_bounds();
                self.keep_x_within_bounds();
            }
            _ => {}
        }
    }
}

impl<S: GlobalState> Layout<S> for Scroll<S> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {
        self.child.calculate_size(requested_size, env);
        self.dimension = requested_size;

        if self.scroll_directions == ScrollDirection::Both ||
            self.scroll_directions == ScrollDirection::Vertical {

                let min_height = 30.0;
                let max_height = requested_size[1];
                let percent_height = requested_size[1] / self.child.get_height();

                let height = (max_height - min_height) * percent_height.min(1.0) + min_height;

                self.scrollbar_vertical.set_height(height);
                self.scrollbar_vertical.calculate_size(requested_size, env);
        }

        if self.scroll_directions == ScrollDirection::Both ||
            self.scroll_directions == ScrollDirection::Horizontal {
                let min_width = 30.0;
                let max_width = requested_size[0];
                let percent_width = requested_size[0] / self.child.get_width();

                let width = (max_width - min_width) * percent_width.min(1.0) + min_width;

                self.scrollbar_horizontal.set_width(width);
                self.scrollbar_horizontal.calculate_size(requested_size, env);
        }

        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::TopLeading.position(); // Top for center
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);

        let child_position = self.child.get_position();

        self.child.set_position([child_position[0] - self.scroll_offset[0], child_position[1] + self.scroll_offset[1]]);


        // Position scrollbars
        self.scrollbar_vertical.set_position(self.get_position().add([self.dimension[0]-self.scrollbar_vertical.get_width(), 0.0]));

        if self.scrollbar_vertical.get_height() - self.get_height() != 0.0 {
            let scroll_vertical_percent = self.scroll_offset[1] / (self.child.get_height() - self.get_height());
            self.scrollbar_vertical.set_position(self.scrollbar_vertical.get_position().add([0.0, -(self.get_height() - self.scrollbar_vertical.get_height()) * scroll_vertical_percent]));
        }

        self.scrollbar_horizontal.set_position(self.get_position().add([0.0, self.dimension[1]-self.scrollbar_horizontal.get_height()]));

        if self.scrollbar_horizontal.get_width() - self.get_width() != 0.0 {
            let scroll_horizontal_percent = self.scroll_offset[0] / (self.child.get_width() - self.get_width());
            self.scrollbar_horizontal.set_position(self.scrollbar_horizontal.get_position().add([(self.get_width() - self.scrollbar_horizontal.get_width()) * scroll_horizontal_percent, 0.0]));
        }


        self.scrollbar_vertical.position_children();
        self.scrollbar_horizontal.position_children();
        self.child.position_children();
    }
}

impl<S: GlobalState> CommonWidget<S> for Scroll<S> {
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

impl<S: GlobalState> Render<S> for Scroll<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        let child_prims = self.get_children().flat_map(|f| f.get_primitives(fonts));
        prims.extend(child_prims);

        if self.scroll_directions == ScrollDirection::Both ||
            self.scroll_directions == ScrollDirection::Vertical {
            prims.extend(self.scrollbar_vertical.get_primitives(fonts));
        }

        if self.scroll_directions == ScrollDirection::Both ||
            self.scroll_directions == ScrollDirection::Horizontal {
            prims.extend(self.scrollbar_horizontal.get_primitives(fonts));
        }

        return prims;
    }
}




