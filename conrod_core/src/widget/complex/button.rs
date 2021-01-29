use crate::prelude::*;
use crate::state::state::State;
use crate::event_handler::KeyboardEvent;
use crate::color::RED;
use crate::input::Key;
use crate::widget::{HStack, Text};
use crate::widget::primitive::spacer::Spacer;
use crate::widget::types::spacer_direction::SpacerDirection;
use crate::widget::primitive::v_stack::VStack;
use crate::widget::primitive::foreach::ForEach;
use crate::widget::complex::foreachtest::ForeachTest;
use crate::widget::render::ChildRender;

#[derive(Debug, Clone, Widget)]
#[state_sync(insert_local_state)]
#[event(handle_keyboard_event)]
pub struct SyncTest<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] value: State<String, GS>,
    #[state] fore: State<Vec<Uuid>, GS>,
    show_overlay: bool,
}

impl<S: GlobalState> SyncTest<S> {

    fn insert_local_state(&self, env: &mut Environment<S>) {
        if self.show_overlay {
            env.add_overlay("overlay_test", Rectangle::initialize(vec![]).fill(RED))
        }
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        match event {
            KeyboardEvent::Text(s, _) => {
                self.value.get_value_mut(global_state).push_str(s);
            }
            KeyboardEvent::Press(key, _modifier) => {
                match key {
                    Key::NumPadMultiply => {
                        self.show_overlay = !self.show_overlay;
                        println!("herjalkd");
                    }
                    Key::Backspace => {
                        self.value.get_value_mut(global_state).pop();
                    },
                    Key::NumPadPlus => {
                        self.fore.get_value_mut(global_state).push(Uuid::new_v4())
                    },
                    Key::NumPadMinus => {
                        if self.fore.get_value(global_state).len() > 1 {
                            let last = self.fore.get_value(global_state).len() - 1;
                            self.fore.get_value_mut(global_state).remove(last);
                        }

                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }

    pub fn new(value: State<String, S>) -> Box<SyncTest<S>> {
        let fore = State::<Vec<Uuid>, S>::new_local("a", &(0..5).map(|_| Uuid::new_v4()).collect::<Vec<Uuid>>());

        Box::new(Self {
            id: Uuid::new_v4(),
            child: HStack::initialize(vec![
                    Spacer::new(SpacerDirection::Horizontal),
                    VStack::initialize(vec![
                        ForEach::new(fore.clone(), ForeachTest::new())
                    ]),
                    ForEach::new((0..5).map(|_| Uuid::new_v4()).collect::<Vec<Uuid>>().into(), Rectangle::initialize(vec![]).frame(10.0,10.0)),
                    Text::initialize(value.clone()),
                    Spacer::new(SpacerDirection::Horizontal),
                    Text::initialize(value.clone()),
                    Spacer::new(SpacerDirection::Horizontal),
            ]),
            position: [100.0,100.0],
            dimension: [100.0,100.0],
            value,
            fore,
            show_overlay: false
        })
    }
}

impl<S: GlobalState> CommonWidget<S> for SyncTest<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::PROXY {
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

impl<S: GlobalState> ChildRender for SyncTest<S> {}

impl<S: GlobalState> Layout<S> for SyncTest<S> {
    fn flexibility(&self) -> u32 {
        2
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