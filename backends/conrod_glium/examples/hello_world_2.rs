//! A simple example that demonstrates using conrod within a basic `winit` window loop, using
//! `glium` to render the `conrod_core::render::Primitives` to screen.

#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
#[macro_use]
extern crate conrod_winit;
extern crate glium;
extern crate uuid;

use std::ops::{Deref, DerefMut};

use glium::Surface;
use uuid::Uuid;

use conrod_core::{Colorable, Point, Positionable, widget};
use conrod_core::color::{GREEN, LIGHT_BLUE, RED};
use conrod_core::event::event::Event;
use conrod_core::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use conrod_core::flags::Flags;
use conrod_core::layout::basic_layouter::BasicLayouter;
use conrod_core::position::Dimensions;
use conrod_core::state::state::{State, StateList};
use conrod_core::text::font::Map;
use conrod_core::widget::{Frame, Image, Line, Oval, Rectangle, SCALE, Text, ZStack};
use conrod_core::widget::common_widget::CommonWidget;
use conrod_core::widget::complex::button::SyncTest;
use conrod_core::layout::Layout;
use conrod_core::widget::oval::Full;
use conrod_core::widget::primitive::edge_insets::EdgeInsets;
use conrod_core::widget::primitive::h_stack::HStack;
use conrod_core::widget::primitive::spacer::{Spacer, SpacerDirection};
use conrod_core::widget::primitive::v_stack::VStack;
use conrod_core::widget::primitive::Widget;
use conrod_core::widget::primitive::widget::WidgetExt;
use conrod_core::widget::render::ChildRender;
use conrod_core::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use conrod_glium::Window;
use conrod_core::layout::layouter::Layouter;

mod support;

const WIDTH: u32 = 750 / 2;
const HEIGHT: u32 = 1334 / 2;

fn main() {
    let mut window = Window::new("Hello world 2".to_string(), WIDTH, HEIGHT, GState {
        s: String::from("Hejsa")
    });

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    let rust_image = window.add_image("images/rust_hover.png").unwrap();

    window.widgets = Some(Box::new(|ui| {}));

    // Rectangle::new(params!(alignment: Alignment::Leading))

    let sync_state = State::new("K", &"Hello".to_string());

    window.set_widgets(
        VStack::initialize(vec![
            Text::initialize("Hello".into(), vec![]),
            Text::initialize("world! \nHvad sker der i denne verden og vil den layoute rigtigt når der er en lang tekst".into(), vec![]),
            Image::new(rust_image, [100.0,100.0], vec![]),
            Rectangle::initialize(vec![
                SyncTest::new(sync_state)
            ]).fill(GREEN),
            HStack::initialize(vec![
                Image::new(rust_image, [100.0, 100.0], vec![]),
                //ZStack::initialize(vec![
                Rectangle::initialize(vec![])
                    .fill(LIGHT_BLUE)
                    .frame(SCALE, 120.0),
                //Image::new(rust_image, [100.0,100.0], vec![])
                // ])
            ]),
            CustomWidget::new(),
        ])
    );
    window.draw()
}

#[derive(Clone)]
struct GState {
    pub s: String,
}

#[derive(Clone)]
pub struct CustomWidget {
    id: Uuid,
    child: Box<dyn Widget<GState>>,
    position: Point,
    dimension: Dimensions,
}

impl CustomWidget {
    pub fn new() -> Box<CustomWidget> {
        Box::new(CustomWidget {
            id: Uuid::new_v4(),
            child: HStack::initialize(vec![
                Spacer::new(SpacerDirection::Horizontal),
                Oval::initialize(vec![])
                    .fill(RED)
                    .padding(EdgeInsets::all(10.0))
                    .frame(150.0, 150.0),
                Spacer::new(SpacerDirection::Horizontal),
                Spacer::new(SpacerDirection::Horizontal)
            ]),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
        })
    }
}

impl CommonWidget<GState> for CustomWidget {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<GState> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GState> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GState> {
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

impl Event<GState> for CustomWidget {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut GState) {
        match event {
            KeyboardEvent::Text(st, _) => {
                global_state.s.push_str(st.as_str());
                println!("Global state says: {}", global_state.s);
            },
            _ => {},
        }
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList) -> StateList {
        self.process_mouse_event_default(event, consumed, state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut GState) -> StateList {
        self.process_keyboard_event_default(event, state, global_state)
    }

    fn get_state(&self, current_state: StateList) -> StateList {
        current_state
    }

    fn apply_state(&mut self, states: StateList) -> StateList {
        states
    }

    fn sync_state(&mut self, states: StateList) {
        self.sync_state_default(states)
    }
}

impl ChildRender for CustomWidget {}

impl Layout for CustomWidget {
    fn flexibility(&self) -> u32 {
        2
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        self.dimension = self.child.calculate_size(requested_size, fonts);
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

impl WidgetExt<GState> for CustomWidget {}