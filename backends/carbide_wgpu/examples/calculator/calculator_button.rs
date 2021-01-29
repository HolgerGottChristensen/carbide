
use carbide_core::widget::primitive::Widget;
use carbide_core::{Point, Color};
use carbide_core::position::Dimensions;
use carbide_core::widget::{HStack, Rectangle, Image};
use carbide_core::widget::primitive::spacer::Spacer;
use carbide_core::color::{RED, rgb_bytes};
use carbide_core::widget::primitive::edge_insets::EdgeInsets;
use carbide_core::widget::common_widget::CommonWidget;
use crate::calculator::calculator_state::CalculatorState;
use carbide_core::flags::Flags;
use carbide_core::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use carbide_core::event::event::Event;
use carbide_core::event_handler::{MouseEvent, WidgetEvent, KeyboardEvent};
use carbide_core::state::state::LocalStateList;
use carbide_core::widget::render::ChildRender;
use carbide_core::layout::Layout;
use carbide_core::text::font::Map;
use carbide_core::layout::basic_layouter::BasicLayouter;
use carbide_core::widget::primitive::widget::WidgetExt;
use carbide_core::layout::layouter::Layouter;
use uuid::Uuid;
use carbide_core::image_map::Id;
use carbide_core::layout::layout::SingleChildLayout;
use carbide_core::state::state_sync::NoLocalStateSync;
use carbide_derive;

#[derive(Clone, Widget)]
#[global_state(CalculatorState)]
#[event(handle_mouse_event)]
pub struct CalculatorButton {
    id: Uuid,
    child: Box<dyn Widget<CalculatorState>>,
    position: Point,
    dimension: Dimensions,
    function: Option<fn(myself: &mut Self, global_state: &mut CalculatorState)>
}

impl CalculatorButton {
    pub fn new(display: Box<dyn Widget<CalculatorState>>) -> Box<CalculatorButton> {
        Box::new(CalculatorButton {
            id: Uuid::new_v4(),
            child: Rectangle::initialize(vec![
                display
            ]).fill(rgb_bytes(76,0,19)),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            function: None
        })
    }

    pub fn on_released(mut self, func: fn(&mut Self, &mut CalculatorState)) -> Box<Self>{
        self.function = Some(func);
        Box::new(self)
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut CalculatorState) {
        match event {
            MouseEvent::Release(_, _, _) => {
                match self.function {
                    None => {}
                    Some(f) => {
                        f(self, global_state)
                    }
                }
            }
            _ => ()
        }
    }
}

impl CommonWidget<CalculatorState> for CalculatorButton {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<CalculatorState> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<CalculatorState> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<CalculatorState> {
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


impl ChildRender for CalculatorButton {}

impl SingleChildLayout for CalculatorButton {
    fn flexibility(&self) -> u32 {
        10
    }
}
