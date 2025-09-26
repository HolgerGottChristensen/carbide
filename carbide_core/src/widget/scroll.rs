use crate::event::{WindowEvent, WindowEventContext};
use carbide_macro::carbide_default_builder2;

use crate::color::Color;
use crate::draw::{Alignment, Dimension, Position};
use crate::environment::EnvironmentColor;
use crate::event::{ModifierKey, MouseButton, MouseEvent, MouseEventContext, MouseEventHandler, WindowEventHandler};
use crate::common::flags::WidgetFlag;
use crate::layout::{Layout, LayoutContext};
use crate::render::{Render, RenderContext};
use crate::widget::{AnyWidget, Capsule, CommonWidget, Empty, Rectangle, Widget, WidgetExt, WidgetId};
use crate::widget::types::ScrollDirection;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, MouseEvent, WindowEvent, Layout)]
pub struct Scroll<W> where W: Widget {
    #[id] id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
    scroll_offset: Position,
    scroll_directions: ScrollDirection,
    scrollbar_horizontal: Box<dyn AnyWidget>,
    scrollbar_vertical: Box<dyn AnyWidget>,
    drag_started_on_vertical_scrollbar: bool,
    drag_started_on_horizontal_scrollbar: bool,
    vertical_scrollbar_hovered: bool,
    horizontal_scrollbar_hovered: bool,
    scrollbar_horizontal_background: Box<dyn AnyWidget>,
    scrollbar_vertical_background: Box<dyn AnyWidget>,
}

impl Scroll<Empty> {
    #[carbide_default_builder2]
    pub fn new<W: Widget>(child: W) -> Scroll<W> {
        Scroll {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            scroll_offset: Position::new(0.0, 0.0),
            scroll_directions: ScrollDirection::Both,
            scrollbar_horizontal: Capsule::new()
                .fill(EnvironmentColor::ThinLight)
                .stroke(EnvironmentColor::ThinDark)
                .stroke_style(1.0)
                .frame(100.0, 8.0)
                .boxed(),
            scrollbar_vertical: Capsule::new()
                .fill(EnvironmentColor::ThinLight)
                .stroke(EnvironmentColor::ThinDark)
                .stroke_style(1.0)
                .frame(8.0, 100.0)
                .boxed(),
            drag_started_on_vertical_scrollbar: false,
            drag_started_on_horizontal_scrollbar: false,
            vertical_scrollbar_hovered: false,
            horizontal_scrollbar_hovered: false,
            scrollbar_horizontal_background: Rectangle::new()
                .fill(Color::Rgba(0.0, 0.0, 0.0, 0.5))
                .frame(100.0, 10.0)
                .boxed(),
            scrollbar_vertical_background: Rectangle::new()
                .fill(Color::Rgba(0.0, 0.0, 0.0, 0.5))
                .frame(10.0, 100.0)
                .boxed(),
        }
    }
}

impl<W: Widget> Scroll<W> {
    pub fn with_scroll_direction(mut self, scroll_directions: ScrollDirection) -> Self {
        self.scroll_directions = scroll_directions;
        self
    }

    fn keep_y_within_bounds(&mut self) {
        if self.scroll_offset.y > 0.0 {
            self.scroll_offset = Position::new(self.scroll_offset.x, 0.0);
        }

        if self.child.height() > self.height() {
            if self.scroll_offset.y < -(self.child.height() - self.height()) {
                self.scroll_offset =
                    Position::new(self.scroll_offset.x, -(self.child.height() - self.height()));
            }
        } else {
            self.scroll_offset = Position::new(self.scroll_offset.x, 0.0);
        }
    }

    fn keep_x_within_bounds(&mut self) {
        if self.scroll_offset.x < 0.0 {
            self.scroll_offset = Position::new(0.0, self.scroll_offset.y);
        }

        if self.child.width() > self.width() {
            if self.scroll_offset.x > (self.child.width() - self.width()) {
                self.scroll_offset =
                    Position::new(self.child.width() - self.width(), self.scroll_offset.y);
            }
        } else {
            self.scroll_offset = Position::new(0.0, self.scroll_offset.y);
        }
    }
}

impl<W: Widget> MouseEventHandler for Scroll<W> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Scroll {
                x, y, modifiers, ..
            } => {
                if !self.is_inside(event.get_current_mouse_position()) {
                    return;
                }

                if self.scroll_directions == ScrollDirection::Both
                    || self.scroll_directions == ScrollDirection::Vertical
                {
                    let offset_multiplier = 1.0; //self.child.height() / self.height();
                    if modifiers.contains(ModifierKey::SHIFT) {
                        self.scroll_offset.y -= x * offset_multiplier;
                    } else {
                        self.scroll_offset.y -= y * offset_multiplier;
                    }

                    self.keep_y_within_bounds();
                }

                if self.scroll_directions == ScrollDirection::Both
                    || self.scroll_directions == ScrollDirection::Horizontal
                {
                    let offset_multiplier = 1.0; //self.child.width() / self.width();
                    if modifiers.contains(ModifierKey::SHIFT) {
                        self.scroll_offset.x += y * offset_multiplier;
                    } else {
                        self.scroll_offset.x -= x * offset_multiplier;
                    }

                    self.keep_x_within_bounds();
                }
            }
            MouseEvent::Release { .. } => {
                self.drag_started_on_vertical_scrollbar = false;
                self.drag_started_on_horizontal_scrollbar = false;
            }
            MouseEvent::Move { to, .. } => {
                self.vertical_scrollbar_hovered = self.scrollbar_vertical_background.is_inside(*to);
                self.horizontal_scrollbar_hovered =
                    self.scrollbar_horizontal_background.is_inside(*to);
            }
            MouseEvent::Press { button: MouseButton::Left, position: point, .. } => {
                if self.scrollbar_vertical_background.is_inside(*point)
                    && !self.scrollbar_vertical.is_inside(*point)
                {
                    let offset_multiplier = self.child.height() / self.height();

                    let middle_of_scrollbar =
                        self.scrollbar_vertical.y() + self.scrollbar_vertical.height() / 2.0;

                    let delta = point.y - middle_of_scrollbar;

                    self.scroll_offset.y -= delta * offset_multiplier;

                    self.keep_y_within_bounds();
                }

                if self.scrollbar_horizontal_background.is_inside(*point)
                    && !self.scrollbar_horizontal.is_inside(*point)
                {
                    let offset_multiplier = self.child.width() / self.width();

                    let middle_of_scrollbar =
                        self.scrollbar_horizontal.x() + self.scrollbar_horizontal.width() / 2.0;

                    let delta = point.x - middle_of_scrollbar;

                    self.scroll_offset.x += delta * offset_multiplier;

                    self.keep_x_within_bounds();
                }
            }
            MouseEvent::Drag {
                origin,
                to,
                delta_xy,
                ..
            } => {
                if !self.drag_started_on_vertical_scrollbar {
                    if self.scrollbar_vertical.is_inside(*origin) {
                        self.drag_started_on_vertical_scrollbar = true;
                    }
                } else {
                    if self.is_inside(Position::new(self.x(), to.y)) {
                        let offset_multiplier = self.child.height() / self.height();
                        self.scroll_offset.y -= delta_xy.y * offset_multiplier;

                        self.keep_y_within_bounds();
                    } else if to.y < self.y() {
                        self.scroll_offset.y = 0.0;
                    } else if to.y > self.y() + self.height() {
                        self.scroll_offset.y = f64::NEG_INFINITY;
                        self.keep_y_within_bounds();
                    }
                }

                if !self.drag_started_on_horizontal_scrollbar {
                    if self.scrollbar_horizontal.is_inside(*origin) {
                        self.drag_started_on_horizontal_scrollbar = true;
                    }
                } else {
                    if self.is_inside(Position::new(to.x, self.y())) {
                        let offset_multiplier = self.child.width() / self.width();
                        self.scroll_offset.x += delta_xy.x * offset_multiplier;
                        self.keep_x_within_bounds();
                    } else if to.x < self.x() {
                        self.scroll_offset.x = 0.0;
                    } else if to.x > self.x() + self.width() {
                        self.scroll_offset.x = f64::INFINITY;
                        self.keep_x_within_bounds();
                    }
                }
            }
            _ => {}
        }
    }
}

impl<W: Widget> WindowEventHandler for Scroll<W> {
    fn handle_window_event(&mut self, _: &WindowEvent, _: &mut WindowEventContext) {
        self.keep_y_within_bounds();
        self.keep_x_within_bounds();
    }
}

impl<W: Widget> Layout for Scroll<W> {
    fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
        self.child.calculate_size(requested_size, ctx);

        self.keep_y_within_bounds();
        self.keep_x_within_bounds();

        self.dimension = requested_size;

        if self.scroll_directions == ScrollDirection::Both
            || self.scroll_directions == ScrollDirection::Vertical
        {
            let min_height = 30.0;
            let max_height = requested_size.height;
            let horizontal_height = if self.scroll_directions == ScrollDirection::Both
                && self.child.width() > self.width()
            {
                self.scrollbar_horizontal.height()
            } else {
                0.0
            };
            let percent_height = max_height / self.child.height();

            let height = (max_height - min_height) * percent_height.min(1.0) + min_height
                - horizontal_height;

            self.scrollbar_vertical.set_height(height);
            self.scrollbar_vertical.calculate_size(requested_size, ctx);

            self.scrollbar_vertical_background
                .set_height(requested_size.height);
            self.scrollbar_vertical_background
                .calculate_size(requested_size, ctx);
        }

        if self.scroll_directions == ScrollDirection::Both
            || self.scroll_directions == ScrollDirection::Horizontal
        {
            let min_width = 30.0;
            let max_width = requested_size.width;
            let vertical_width = if self.scroll_directions == ScrollDirection::Both
                && self.child.height() > self.height()
            {
                self.scrollbar_vertical.width()
            } else {
                0.0
            };

            let percent_width = max_width / self.child.width();

            let width =
                (max_width - min_width) * percent_width.min(1.0) + min_width - vertical_width;

            self.scrollbar_horizontal.set_width(width);
            self.scrollbar_horizontal
                .calculate_size(requested_size, ctx);
            self.scrollbar_horizontal_background
                .set_width(requested_size.width);
            self.scrollbar_horizontal_background
                .calculate_size(requested_size, ctx);
        }

        requested_size
    }

    fn position_children(&mut self, ctx: &mut LayoutContext) {
        let position = self.position;
        let dimension = self.dimension;

        self.child.set_position(Alignment::TopLeading.position(position, dimension, self.child.dimension()));

        let child_position = self.child.position();

        self.child.set_position(Position::new(
            child_position.x - self.scroll_offset.x,
            child_position.y + self.scroll_offset.y,
        ));

        // Position scrollbars
        self.scrollbar_vertical.set_position(
            self.position()
                + Position::new(self.dimension.width - self.scrollbar_vertical.width(), 0.0),
        );
        self.scrollbar_vertical_background.set_position(
            self.position()
                + Position::new(self.dimension.width - self.scrollbar_vertical.width(), 0.0),
        );

        let scroll_vertical_percent = if self.child.height() - self.height() != 0.0 {
            self.scroll_offset.y / (self.child.height() - self.height())
        } else {
            0.0
        };

        let horizontal_height = if self.scroll_directions == ScrollDirection::Both
            && self.child.width() > self.width()
        {
            self.scrollbar_horizontal.height()
        } else {
            0.0
        };

        self.scrollbar_vertical.set_position(
            self.scrollbar_vertical.position()
                + Position::new(
                    0.0,
                    -(self.height() - horizontal_height - self.scrollbar_vertical.height())
                        * scroll_vertical_percent,
                ),
        );

        self.scrollbar_horizontal.set_position(
            self.position()
                + Position::new(
                    0.0,
                    self.dimension.height - self.scrollbar_horizontal.height(),
                ),
        );
        self.scrollbar_horizontal_background.set_position(
            self.position()
                + Position::new(
                    0.0,
                    self.dimension.height - self.scrollbar_horizontal.height(),
                ),
        );

        let scroll_horizontal_percent = if self.child.width() - self.width() != 0.0 {
            self.scroll_offset.x / (self.child.width() - self.width())
        } else {
            0.0
        };

        let vertical_width = if self.scroll_directions == ScrollDirection::Both
            && self.child.height() > self.height()
        {
            self.scrollbar_vertical.width()
        } else {
            0.0
        };

        self.scrollbar_horizontal.set_position(
            self.scrollbar_horizontal.position()
                + Position::new(
                    (self.width() - vertical_width - self.scrollbar_horizontal.width())
                        * scroll_horizontal_percent,
                    0.0,
                ),
        );

        self.scrollbar_vertical.position_children(ctx);
        self.scrollbar_horizontal.position_children(ctx);
        self.scrollbar_vertical_background.position_children(ctx);
        self.scrollbar_horizontal_background.position_children(ctx);
        self.child.position_children(ctx);
    }
}

impl<W: Widget> CommonWidget for Scroll<W> {
    fn flag(&self) -> WidgetFlag {
        WidgetFlag::EMPTY
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_rev(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn flexibility(&self) -> u32 {
        0
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<W: Widget> Render for Scroll<W> {
    fn render(&mut self, context: &mut RenderContext) {
        self.child.render(context);

        if (self.scroll_directions == ScrollDirection::Both
            || self.scroll_directions == ScrollDirection::Vertical)
            && self.child.height() > self.height()
        {
            if self.vertical_scrollbar_hovered || self.drag_started_on_vertical_scrollbar {
                self.scrollbar_vertical_background
                    .render(context);
            }

            self.scrollbar_vertical.render(context);
        }

        if (self.scroll_directions == ScrollDirection::Both
            || self.scroll_directions == ScrollDirection::Horizontal)
            && self.child.width() > self.width()
        {
            if self.horizontal_scrollbar_hovered || self.drag_started_on_horizontal_scrollbar {
                self.scrollbar_horizontal_background.render(context);
            }

            self.scrollbar_horizontal.render(context);
        }
    }
}