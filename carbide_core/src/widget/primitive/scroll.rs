use crate::draw::shape::vertex::Vertex;
use crate::event_handler::{MouseEvent, WidgetEvent};
use crate::input::MouseButton;
use crate::prelude::*;
use crate::state::global_state::GlobalStateContainer;
use crate::widget::render::RenderProcessor;
use crate::widget::types::scroll_direction::ScrollDirection;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
//#[event(handle_mouse_event, handle_other_event)]
//#[state_sync(update_all_widget_state)]
#[render(process_get_primitives)]
pub struct Scroll {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Point,
    dimension: Dimensions,
    scroll_offset: [f64; 2],
    scroll_directions: ScrollDirection,
    scrollbar_horizontal: Box<dyn Widget>,
    scrollbar_vertical: Box<dyn Widget>,
    drag_started_on_vertical_scrollbar: bool,
    drag_started_on_horizontal_scrollbar: bool,
    vertical_scrollbar_hovered: bool,
    horizontal_scrollbar_hovered: bool,
    scrollbar_horizontal_background: Box<dyn Widget>,
    scrollbar_vertical_background: Box<dyn Widget>,
}

impl Scroll {
    /*fn update_all_widget_state(&mut self, env: &mut Environment<GS>, global_state: &GS) {
        self.scrollbar_horizontal.sync_state(env, global_state);
        self.scrollbar_vertical.sync_state(env, global_state);
        self.scrollbar_horizontal_background.sync_state(env, global_state);
        self.scrollbar_vertical_background.sync_state(env, global_state);
    }*/

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

    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Self {
            id: Uuid::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            scroll_offset: [0.0, 0.0],
            scroll_directions: ScrollDirection::Both,
            scrollbar_horizontal: Rectangle::initialize(vec![])
                .fill(EnvironmentColor::Gray)
                .frame(100.0, 10.0),
            scrollbar_vertical: Rectangle::initialize(vec![])
                .fill(EnvironmentColor::Gray)
                .frame(10.0, 100.0),
            drag_started_on_vertical_scrollbar: false,
            drag_started_on_horizontal_scrollbar: false,
            vertical_scrollbar_hovered: false,
            horizontal_scrollbar_hovered: false,
            scrollbar_horizontal_background: Rectangle::initialize(vec![]).fill(Color::Rgba(0.0, 0.0, 0.0, 0.5)).frame(100.0, 10.0),
            scrollbar_vertical_background: Rectangle::initialize(vec![]).fill(Color::Rgba(0.0, 0.0, 0.0, 0.5)).frame(10.0, 100.0),
        })
    }

    /*fn handle_mouse_event(&mut self, event: &MouseEvent, _: &bool, _: &mut Environment<GS>, _: &mut GS) {
        match event {
            MouseEvent::Scroll { x, y, modifiers, .. } => {
                if !self.is_inside(event.get_current_mouse_position()) { return }

                if self.scroll_directions == ScrollDirection::Both ||
                    self.scroll_directions == ScrollDirection::Vertical {
                    let offset_multiplier = 1.0; //self.child.get_height() / self.get_height();
                    if modifiers.contains(piston_input::keyboard::ModifierKey::SHIFT) {
                        self.scroll_offset[1] -= x * offset_multiplier;
                    } else {
                        self.scroll_offset[1] -= y * offset_multiplier;
                    }

                    self.keep_y_within_bounds();
                }

                if self.scroll_directions == ScrollDirection::Both ||
                    self.scroll_directions == ScrollDirection::Horizontal {
                    let offset_multiplier = 1.0; //self.child.get_width() / self.get_width();
                    if modifiers.contains(piston_input::keyboard::ModifierKey::SHIFT) {
                        self.scroll_offset[0] += y * offset_multiplier;
                    } else {
                        self.scroll_offset[0] -= x * offset_multiplier;
                    }

                    self.keep_x_within_bounds();
                }
            }
            MouseEvent::Release(..) => {
                self.drag_started_on_vertical_scrollbar = false;
                self.drag_started_on_horizontal_scrollbar = false;
            }
            MouseEvent::Move { to, .. } => {
                self.vertical_scrollbar_hovered = self.scrollbar_vertical_background.is_inside(*to);
                self.horizontal_scrollbar_hovered = self.scrollbar_horizontal_background.is_inside(*to);
            }
            MouseEvent::Press(MouseButton::Left, point, ..) => {
                if self.scrollbar_vertical_background.is_inside(*point) && !self.scrollbar_vertical.is_inside(*point) {
                    let offset_multiplier = self.child.get_height() / self.get_height();

                    let middle_of_scrollbar = self.scrollbar_vertical.get_y() + self.scrollbar_vertical.get_height() / 2.0;

                    let delta = point[1] - middle_of_scrollbar;

                    self.scroll_offset[1] -= delta * offset_multiplier;

                    self.keep_y_within_bounds();
                }

                if self.scrollbar_horizontal_background.is_inside(*point) && !self.scrollbar_horizontal.is_inside(*point) {
                    let offset_multiplier = self.child.get_width() / self.get_width();

                    let middle_of_scrollbar = self.scrollbar_horizontal.get_x() + self.scrollbar_horizontal.get_width() / 2.0;

                    let delta = point[0] - middle_of_scrollbar;

                    self.scroll_offset[0] += delta * offset_multiplier;

                    self.keep_x_within_bounds();
                }
            }
            MouseEvent::Drag { origin, to, delta_xy, .. } => {
                if !self.drag_started_on_vertical_scrollbar {
                    if self.scrollbar_vertical.is_inside(*origin) {
                        self.drag_started_on_vertical_scrollbar = true;
                    }
                } else {
                    if self.is_inside([self.get_x(), to[1]]) {
                        let offset_multiplier = self.child.get_height() / self.get_height();
                        self.scroll_offset[1] -= delta_xy[1] * offset_multiplier;

                        self.keep_y_within_bounds();
                    } else if to[1] < self.get_y() {
                        self.scroll_offset[1] = 0.0;
                    } else if to[1] > self.get_y() + self.get_height() {
                        self.scroll_offset[1] = f64::NEG_INFINITY;
                        self.keep_y_within_bounds();
                    }
                }

                if !self.drag_started_on_horizontal_scrollbar {
                    if self.scrollbar_horizontal.is_inside(*origin) {
                        self.drag_started_on_horizontal_scrollbar = true;
                    }
                } else {
                    if self.is_inside([to[0], self.get_y()]) {
                        let offset_multiplier = self.child.get_width() / self.get_width();
                        self.scroll_offset[0] += delta_xy[0] * offset_multiplier;
                        self.keep_x_within_bounds();
                    } else if to[0] < self.get_x() {
                        self.scroll_offset[0] = 0.0;
                    } else if to[0] > self.get_x() + self.get_width() {
                        self.scroll_offset[0] = f64::INFINITY;
                        self.keep_x_within_bounds();
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_other_event(&mut self, event: &WidgetEvent, _: &mut Environment<GS>, _: &mut GS) {
        match event {
            WidgetEvent::Window(_) => {
                self.keep_y_within_bounds();
                self.keep_x_within_bounds();
            }
            _ => {}
        }
    }*/

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        primitives.extend(self.get_children_mut().flat_map(|f| f.get_primitives(env)));

        if (self.scroll_directions == ScrollDirection::Both ||
            self.scroll_directions == ScrollDirection::Vertical) && self.child.get_height() > self.get_height() {
            if self.vertical_scrollbar_hovered || self.drag_started_on_vertical_scrollbar {
                self.scrollbar_vertical_background.process_get_primitives(primitives, env);
            }

            self.scrollbar_vertical.process_get_primitives(primitives, env);
        }

        if (self.scroll_directions == ScrollDirection::Both ||
            self.scroll_directions == ScrollDirection::Horizontal) && self.child.get_width() > self.get_width() {
            if self.horizontal_scrollbar_hovered || self.drag_started_on_horizontal_scrollbar {
                self.scrollbar_horizontal_background.process_get_primitives(primitives, env);
            }

            self.scrollbar_horizontal.process_get_primitives(primitives, env);
        }
    }
}

impl Layout for Scroll {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        self.child.calculate_size(requested_size, env);

        self.keep_y_within_bounds();
        self.keep_x_within_bounds();

        self.dimension = requested_size;

        if self.scroll_directions == ScrollDirection::Both ||
            self.scroll_directions == ScrollDirection::Vertical {
            let min_height = 30.0;
            let max_height = requested_size[1];
            let horizontal_height = if self.scroll_directions == ScrollDirection::Both && self.child.get_width() > self.get_width() {
                self.scrollbar_horizontal.get_height()
            } else {
                0.0
            };
            let percent_height = max_height / self.child.get_height();

            let height = (max_height - min_height) * percent_height.min(1.0) + min_height - horizontal_height;

            self.scrollbar_vertical.set_height(height);
            self.scrollbar_vertical.calculate_size(requested_size, env);

            self.scrollbar_vertical_background.set_height(requested_size[1]);
            self.scrollbar_vertical_background.calculate_size(requested_size, env);
        }

        if self.scroll_directions == ScrollDirection::Both ||
            self.scroll_directions == ScrollDirection::Horizontal {
            let min_width = 30.0;
            let max_width = requested_size[0];
            let vertical_width = if self.scroll_directions == ScrollDirection::Both && self.child.get_height() > self.get_height() {
                self.scrollbar_vertical.get_width()
            } else {
                0.0
            };

            let percent_width = max_width / self.child.get_width();

            let width = (max_width - min_width) * percent_width.min(1.0) + min_width - vertical_width;

            self.scrollbar_horizontal.set_width(width);
            self.scrollbar_horizontal.calculate_size(requested_size, env);
            self.scrollbar_horizontal_background.set_width(requested_size[0]);
            self.scrollbar_horizontal_background.calculate_size(requested_size, env);
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
        self.scrollbar_vertical.set_position(self.get_position().add([self.dimension[0] - self.scrollbar_vertical.get_width(), 0.0]));
        self.scrollbar_vertical_background.set_position(self.get_position().add([self.dimension[0] - self.scrollbar_vertical.get_width(), 0.0]));


        let scroll_vertical_percent = if self.child.get_height() - self.get_height() != 0.0 {
            self.scroll_offset[1] / (self.child.get_height() - self.get_height())
        } else {
            0.0
        };

        let horizontal_height = if self.scroll_directions == ScrollDirection::Both && self.child.get_width() > self.get_width() {
            self.scrollbar_horizontal.get_height()
        } else {
            0.0
        };

        self.scrollbar_vertical.set_position(self.scrollbar_vertical.get_position().add([0.0, -(self.get_height() - horizontal_height - self.scrollbar_vertical.get_height()) * scroll_vertical_percent]));


        self.scrollbar_horizontal.set_position(self.get_position().add([0.0, self.dimension[1] - self.scrollbar_horizontal.get_height()]));
        self.scrollbar_horizontal_background.set_position(self.get_position().add([0.0, self.dimension[1] - self.scrollbar_horizontal.get_height()]));


        let scroll_horizontal_percent = if self.child.get_width() - self.get_width() != 0.0 {
            self.scroll_offset[0] / (self.child.get_width() - self.get_width())
        } else {
            0.0
        };

        let vertical_width = if self.scroll_directions == ScrollDirection::Both && self.child.get_height() > self.get_height() {
            self.scrollbar_vertical.get_width()
        } else {
            0.0
        };

        self.scrollbar_horizontal.set_position(self.scrollbar_horizontal.get_position().add([(self.get_width() - vertical_width - self.scrollbar_horizontal.get_width()) * scroll_horizontal_percent, 0.0]));


        self.scrollbar_vertical.position_children();
        self.scrollbar_horizontal.position_children();
        self.scrollbar_vertical_background.position_children();
        self.scrollbar_horizontal_background.position_children();
        self.child.position_children();
    }
}

impl CommonWidget for Scroll {
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

impl Render for Scroll {
    fn get_primitives(&mut self, _: &mut Environment) -> Vec<Primitive> {
        let mut prims = vec![];
        return prims;
    }
}

impl WidgetExt for Scroll {}




