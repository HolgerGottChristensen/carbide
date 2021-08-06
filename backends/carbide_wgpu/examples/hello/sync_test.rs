use carbide_core::event_handler::KeyboardEvent;
use carbide_core::input::Key;
use carbide_core::prelude::*;
use carbide_core::widget::*;

#[derive(Debug, Clone, Widget)]
#[state_sync(insert_local_state)]
#[event(handle_keyboard_event)]
pub struct SyncTest<GS> where GS: GlobalStateContract {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] value: CommonState<String, GS>,
    #[state] fore: CommonState<Vec<Uuid>, GS>,
    show_overlay: bool,
}

impl<S: GlobalStateContract> SyncTest<S> {
    fn insert_local_state(&self, env: &mut Environment<S>) {
        if self.show_overlay {
            env.add_overlay("overlay_test", Rectangle::initialize(vec![])
                .fill(EnvironmentColor::Red))
        }
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<S>, global_state: &mut S) {
        match event {
            KeyboardEvent::Text(s, _) => {
                self.value.get_value_mut(env, global_state).push_str(s);
            }
            KeyboardEvent::Press(key, _modifier) => {
                match key {
                    Key::NumPadMultiply => {
                        self.show_overlay = !self.show_overlay;
                        println!("herjalkd");
                    }
                    Key::Backspace => {
                        self.value.get_value_mut(env, global_state).pop();
                    }
                    Key::NumPadPlus => {
                        self.fore.get_value_mut(env, global_state).push(Uuid::new_v4())
                    }
                    Key::NumPadMinus => {
                        if self.fore.get_value(env, global_state).len() > 1 {
                            let last = self.fore.get_value(env, global_state).len() - 1;
                            self.fore.get_value_mut(env, global_state).remove(last);
                        }
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }

    pub fn new(value: CommonState<String, S>) -> Box<SyncTest<S>> {
        let fore = CommonState::<Vec<Uuid>, S>::new_local_with_key(&(0..5).map(|_| Uuid::new_v4()).collect::<Vec<Uuid>>());

        let index_state = Box::new(CommonState::new_local_with_key(&1));

        let mapped_state = MappedState::new_local(index_state.clone(), |a: &usize| format!("{}", a), "0".to_string());

        Box::new(Self {
            id: Uuid::new_v4(),
            child: HStack::initialize(vec![
                Spacer::new(SpacerDirection::Horizontal),
                VStack::initialize(vec![
                    ForEach::new(
                        Box::new(fore.clone()),
                        Rectangle::initialize(vec![
                            Text::new(mapped_state)
                        ]).fill(EnvironmentColor::Red).frame(60.0, 30.0))
                        .index_state(index_state)
                ]),
                ForEach::new((0..5).map(|_| Uuid::new_v4()).collect::<Vec<Uuid>>(), Rectangle::initialize(vec![]).frame(10.0, 10.0)),
                Text::new(value.clone()),
                Spacer::new(SpacerDirection::Horizontal),
                Text::new(value.clone()),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            position: [100.0, 100.0],
            dimension: [100.0, 100.0],
            value,
            fore,
            show_overlay: false,
        })
    }
}

impl<S: GlobalStateContract> CommonWidget<S> for SyncTest<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(self.child.deref())
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(self.child.deref_mut())
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
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

impl<S: GlobalStateContract> ChildRender for SyncTest<S> {}

impl<S: GlobalStateContract> Layout<S> for SyncTest<S> {
    fn flexibility(&self) -> u32 {
        2
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
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

impl<GS: GlobalStateContract> WidgetExt<GS> for SyncTest<GS> {}