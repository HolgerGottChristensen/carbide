//! A simple, non-interactive rectangle shape widget.
//!
//! Due to the frequency of its use in GUIs, the `Rectangle` gets its own widget to allow backends
//! to specialise their rendering implementations.
use {Color, Colorable, Point, Rect, Sizeable};
use super::Style as Style;
use ::{widget, Scalar};
use widget::render::Render;
use graph::Container;
use widget::Id;
use color::{rgb, YELLOW, PURPLE, LIGHT_BLUE};
use render::primitive::Primitive;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use widget::common_widget::CommonWidget;
use uuid::Uuid;
use widget::primitive::Widget;
use position::Dimensions;
use daggy::petgraph::graph::node_index;
use ::{Range, text};
use render::owned_primitive::OwnedPrimitive;
use render::owned_primitive_kind::OwnedPrimitiveKind;

use std::convert::TryFrom;
use std::error::Error;
use std::collections::HashMap;
use std::any::Any;

use text::font::Map;
use layout::basic_layouter::BasicLayouter;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;
use input::Key;
use state::state::{StateList};
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};
use std::slice::{Iter, IterMut};
use draw::shape::triangle::Triangle;
use layout::Layout;
use layout::layouter::Layouter;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, WidgetCommon_)]
pub struct Rectangle<S> {
    id: Uuid,
    children: Vec<Box<dyn Widget<S>>>,
    position: Point,
    dimension: Dimensions,
    /// Data necessary and common for all widget builder render.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the **Rectangle**.
    pub style: Style,
    color: Color
}

impl<K: 'static + Clone> WidgetExt<K> for Rectangle<K> {}

impl<K> Event<K> for Rectangle<K> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut K) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut K) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList, global_state: &mut K) -> StateList {
        self.process_mouse_event_default(event, consumed, state, global_state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut K) -> StateList {
        self.process_keyboard_event_default(event, state, global_state)
    }

    fn get_state(&self, current_state: StateList) -> StateList {
        current_state
    }

    fn apply_state(&mut self, states: StateList, _: &K) -> StateList {
        states
    }

    fn sync_state(&mut self, states: StateList, global_state: &K) {
        self.sync_state_default(states, global_state);
    }
}

impl<K> Layout<K> for Rectangle<K> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        for child in &mut self.children {
            child.calculate_size(requested_size, fonts);
        }
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in &mut self.children {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl<S> CommonWidget<S> for Rectangle<S> {
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

impl<S> Render<S> for Rectangle<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: self.color},
                scizzor: Rect::new(self.position, self.dimension),
                rect: Rect::new(self.position, self.dimension)
            }
        ];
        prims.extend(Rectangle::<S>::rect_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}

/// Unique state for the Rectangle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    kind: Kind,
}

/// Whether the rectangle is drawn as an outline or a filled color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    /// Only the outline of the rectangle is drawn.
    Outline,
    /// The rectangle area is filled with some color.
    Fill,
}


impl<S> Rectangle<S> {

    pub fn fill(mut self, color: Color) -> Box<Self> {
        self.color = color;
        Box::new(self)
    }

    pub fn rect_outline(rect: Rect, width: Scalar) -> Vec<Primitive> {
        let (l, r, b, t) = rect.l_r_b_t();

        let left_border = Rect::new([l,b], [width, rect.h()]);
        let right_border = Rect::new([r-width,b], [width, rect.h()]);
        let top_border = Rect::new([l+width,b], [rect.w()-width*2.0, width]);
        let bottom_border = Rect::new([l+width,t-width], [rect.w()-width*2.0, width]);

        let border_color = Color::Rgba(0.0 / 255.0, 255.0 / 255.0, 251.0 / 255.0, 1.0);//Color::random();
        vec![
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                scizzor: left_border,
                rect: left_border
            },
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                scizzor: right_border,
                rect: right_border
            },
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                scizzor: top_border,
                rect: top_border
            },
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                scizzor: bottom_border,
                rect: bottom_border
            },
        ]
    }

    /// Build a rectangle with the dimensions and style.
    pub fn styled(dim: Dimensions, style: Style) -> Self {
        Rectangle {
            id: Uuid::new_v4(),
            children: vec![],
            position: [1.0, 1.0],
            dimension: [1.0, 1.0],
            common: widget::CommonBuilder::default(),
            style,
            color: Color::random()
        }//.wh(dim)
    }

    /// Build a new filled rectangle.
    pub fn fill_old(dim: Dimensions) -> Self {
        Rectangle::styled(dim, Style::fill())
    }

    /// Build a new filled rectangle widget filled with the given color.
    pub fn fill_with(dim: Dimensions, color: Color) -> Self {
        Rectangle::styled(dim, Style::fill_with(color))
    }

    /// Build a new outlined rectangle widget.
    pub fn outline(dim: Dimensions) -> Self {
        Rectangle::styled(dim, Style::outline())
    }

    /// Build an outlined rectangle rather than a filled one.
    pub fn outline_styled(dim: Dimensions, line_style: widget::line::Style) -> Self {
        Rectangle::styled(dim, Style::outline_styled(line_style))
    }

    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<Rectangle<S>> {
        Box::new(Rectangle {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            common: widget::CommonBuilder::default(),
            style: Style::fill(),
            color: Color::random()
        })
    }

    pub fn new(position: Point, dimension: Dimensions, children: Vec<Box<dyn Widget<S>>>) -> Box<Rectangle<S>> {
        Box::new(Rectangle {
            id: Uuid::new_v4(),
            children,
            position,
            dimension,
            common: widget::CommonBuilder::default(),
            style: Style::fill(),
            color: Color::random()
        })
    }
}

/*impl<S> OldWidget<S> for Rectangle<S> {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            kind: Kind::Fill,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the Rectangle.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, style, .. } = args;

        let kind = match *style {
            Style::Fill(_) => Kind::Fill,
            Style::Outline(_) => Kind::Outline,
        };

        if state.kind != kind {
            state.update(|state| state.kind = kind);
        }
    }

}
*/

impl<S> Colorable for Rectangle<S> {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}


/// The two triangles that describe the given `Rect`.
pub fn triangles(rect: Rect) -> (Triangle<Point>, Triangle<Point>) {
    let (l, r, b, t) = rect.l_r_b_t();
    let quad = [[l, t], [r, t], [r, b], [l, b]];
    widget::triangles::from_quad(quad)
}
