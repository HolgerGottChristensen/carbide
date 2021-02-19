use carbide_core::widget::*;
use carbide_core::event_handler::{MouseEvent, KeyboardEvent};
use carbide_core::state::state::State;
use carbide_core::widget::primitive::foreach::{ForEachDelegate, ForEach};
use carbide_core::color::{RED, BLUE};
use std::option::Option::Some;
use carbide_core::prelude::StateSync;

pub trait ListIndex: ForEachDelegate {}

impl<T> ListIndex for T where T: ForEachDelegate {}

#[event(handle_keyboard_event, handle_mouse_event)]
#[derive(Clone, Widget)]
#[state_sync(sync_state)]
pub struct List<GS, T> where GS: GlobalState, T: ListIndex + 'static {
    id: Id,
    child: Box<dyn Widget<GS>>,
    delegate: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] model: Box<dyn State<Vec<T>, GS>>,
    #[state] internal_model: Box<dyn State<Vec<T>, GS>>,
    #[state] index_offset: Box<dyn State<usize, GS>>,
    #[state] start_offset: Box<dyn State<f64, GS>>,
    #[state] end_offset: Box<dyn State<f64, GS>>,
    id_state: Box<dyn State<T, GS>>,
    index_state: Box<dyn State<usize, GS>>,
}

impl<GS: GlobalState, T: ListIndex + 'static> List<GS, T> {

    pub fn new(delegate: Box<dyn Widget<GS>>, model: Box<dyn State<Vec<T>, GS>>) -> Box<Self> {

        let index_offset_state = Box::new(CommonState::new_local_with_key(&0));

        let start_offset = CommonState::new_local_with_key(&-10.0);
        let end_offset = CommonState::new_local_with_key(&-10.0);

        let internal_model = Box::new(CommonState::new_local_with_key(model.get_latest_value()));

        Box::new(List {
            id: Id::new_v4(),
            child: Scroll::new(VStack::initialize(vec![
                Rectangle::initialize(vec![]).fill(RED).frame(SCALE.into(), Box::new(start_offset.clone())),
                ForEach::new(internal_model.clone(), delegate.clone())
                    .index_offset(index_offset_state.clone()),
                Rectangle::initialize(vec![]).fill(BLUE).frame(SCALE.into(), Box::new(end_offset.clone())),
            ])),
            delegate,
            position: [0.0,0.0],
            dimension: [0.0,0.0],
            model,
            internal_model,
            index_offset: index_offset_state,
            id_state: Box::new(CommonState::new_local("id", &T::default())),
            index_state: Box::new(CommonState::new_local("index", &0)),
            start_offset: Box::new(start_offset),
            end_offset: Box::new(end_offset)
        })
    }

    pub fn id_state(mut self, state: Box<dyn State<T, GS>>) -> Box<Self> {
        self.id_state = state;
        self.child = Scroll::new(VStack::initialize(vec![
            Rectangle::initialize(vec![]).fill(RED).frame(SCALE.into(), Box::new(self.start_offset.clone())),
            ForEach::new(self.internal_model.clone(), self.delegate.clone())
                .index_offset(self.index_offset.clone())
                .id_state(self.id_state.clone())
                .index_state(self.index_state.clone()),
            Rectangle::initialize(vec![]).fill(BLUE).frame(SCALE.into(), Box::new(self.end_offset.clone())),
        ]));
        Box::new(self)
    }

    pub fn index_state(mut self, state: Box<dyn State<usize, GS>>) -> Box<Self> {
        self.index_state = state;
        self.child = Scroll::new(VStack::initialize(vec![
            Rectangle::initialize(vec![]).fill(RED).frame(SCALE.into(), Box::new(self.start_offset.clone())),
            ForEach::new(self.internal_model.clone(), self.delegate.clone())
                .index_offset(self.index_offset.clone())
                .id_state(self.id_state.clone())
                .index_state(self.index_state.clone()),
            Rectangle::initialize(vec![]).fill(BLUE).frame(SCALE.into(), Box::new(self.end_offset.clone())),
        ]));
        Box::new(self)
    }

    fn handle_mouse_event(&mut self, _: &MouseEvent, _: &bool, _: &mut Environment<GS>, _: &mut GS) {

    }

    fn handle_keyboard_event(&mut self, _: &KeyboardEvent, _: &mut Environment<GS>, _: &mut GS) {

    }

    fn recalculate_visible_children(&mut self, env: &mut Environment<GS>, _: &GS) {
        // TODO: Handle when model changes.
        // If items in the internal model is removed, calculate new sizes, if items in between the items in the internal_model is added, do ???

        let spacing = 10.0;

        let dims = self.get_dimension();

        let self_y = self.get_y();
        let self_height = self.get_height();

        let mut self_start_offset = self.start_offset.clone();
        let mut self_end_offset = self.end_offset.clone();
        let mut self_internal_model = self.internal_model.clone();
        let self_model = self.model.clone();
        let mut self_index_offset = self.index_offset.clone();

        // Notice the scroll might not be the first when clip is added.
        if let Some(scroll_view) = self.get_children_mut().next() {

            // Calculate the size off all the children.
            scroll_view.calculate_size(dims, env);

            // Position all the children. This will not be their final position though, just relative
            // to the current position of the list view, but this does not matter.
            scroll_view.position_children();

            if let Some(vstack) = scroll_view.get_children_mut().next() {



                let internal_model_size = self_internal_model.get_latest_value().len();

                // Get the children of the scrollview
                let mut vstack_children = vstack.get_children_mut().enumerate();

                // The first child is the initial_offset Rectangle.
                vstack_children.next();


                // Handle remove items out of view
                while let Some((index, child)) = vstack_children.next() {
                    if index > internal_model_size {continue}

                    // Check if an items bottom is above the top of this view.
                    if child.get_y() + child.get_height() < self_y {
                        *self_start_offset.get_latest_value_mut() += child.get_height() + spacing;
                        self_internal_model.get_latest_value_mut().remove(0);
                        *self_index_offset.get_latest_value_mut() += 1;

                    }

                    if child.get_y() > self_y + self_height {
                        *self_end_offset.get_latest_value_mut() += child.get_height() + spacing;
                        self_internal_model.get_latest_value_mut().pop();
                    }


                }







                // Get the children of the scrollview
                let mut vstack_children = vstack.get_children_mut().enumerate();

                // TODO: Consider choosing -scroll_offset for this value.
                let (_, initial_child) = vstack_children.next()
                    .expect("Could not find the initial rectangle");

                let inital_y = initial_child.get_y();

                let mut min_height = 1000.0;

                while let Some((index, child)) = vstack_children.next() {
                    if index > internal_model_size {continue}
                    if child.get_height() < min_height {
                        min_height = child.get_height();
                    }
                }

                // Handle add items to view from the top
                while *self_start_offset.get_latest_value() + inital_y > self_y {
                    *self_start_offset.get_latest_value_mut() -= min_height + spacing;
                    *self_index_offset.get_latest_value_mut() -= 1;
                    let index_to_take_from = *self_index_offset.get_latest_value();


                    if self_model.get_latest_value().len() < index_to_take_from {
                        panic!("Can not take index from model, that is outside the bounds of said model")
                    }


                    self_internal_model.get_latest_value_mut().insert(0, self_model.get_latest_value()[index_to_take_from].clone());
                }


                let mut vstack_children = vstack.get_children_mut();

                let mut last_y = 0.0;

                while let Some(vstack_child) = vstack_children.next() {
                    last_y = vstack_child.get_y();
                }


                // Handle add items to view from the bottom
                while last_y < self_y + self_height {
                    last_y += (min_height + spacing);
                    *self_end_offset.get_latest_value_mut() -= min_height + spacing;
                    let index_to_take_from = *self_index_offset.get_latest_value() + self_internal_model.get_latest_value().len();


                    if self_model.get_latest_value().len() < index_to_take_from {
                        panic!("Can not take index from model, that is outside the bounds of said model")
                    }


                    self_internal_model.get_latest_value_mut().push(self_model.get_latest_value()[index_to_take_from].clone());
                }


            }

        }

        self.start_offset = self_start_offset;
        self.end_offset = self_end_offset;
        self.internal_model = self_internal_model;
        self.index_offset = self_index_offset;


        // If yes, expand the
        // initial_offset to the size of this view + 1 x spacing. Then remove this from the internal model.
        // Do this step while the item is above the top.

        // Check if the bottom of the initial_offset is below the top of the view. If yes and
        // its size is not 0, take the view from the model before the current top in the internal
        // model and add this to the front of the internal model. Recalculate children size and
        // subtract the first child in the internal_model's height + 1 x spacing from the inital_offset.
        // Recalculate the size of the initial_offset again.

        // We need an after_offset that simulates the currently known height of the lists content.
        // This can be inferred from one of two cases:
        // - The bottom point is known because the last element has been visited.
        // - The bottom point can be inferred by looking at the number of elements in the already
        //   seen part of the list and divide by the number of elements in the internal model + initial_index
        //   and multiply by the number of elements in the model.

        // If the top of the after_offset is above the bottom of the list, check if there is more
        // items in the model. If not, note the bottom has been reached, and its height.
        // If there are more items in the model add that item, calculate its size and subtract that
        // + 1 x spacing from the after_offset. If needed the after_offset can be inferred again.
        // Recalculate the size of the after_offset again.

        // Check if an item is below the bottom of the list. If that is the case, remove that
        // element from the internal_model and add its size + 1 x spacing to the after_offset.
        // Recalculate the height of the after offset.
        // To this step while there are views below the bottom.

    }

    fn sync_state(&mut self, env: &mut Environment<GS>, global_state: &GS) {
        self.update_all_widget_state(env, global_state);

        self.recalculate_visible_children(env, global_state);

        self.insert_local_state(env);

        for child in self.get_proxied_children() {
            child.sync_state(env, global_state)
        }

        self.update_local_widget_state(env);
    }

}

impl<GS: GlobalState, T: ListIndex> CommonWidget<GS> for List<GS, T> {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
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

impl<GS: GlobalState, T: ListIndex> ChildRender for List<GS, T> {}

impl<GS: GlobalState, T: ListIndex> Layout<GS> for List<GS, T> {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<GS>) -> Dimensions {

        if let Some(child) = self.get_children_mut().next() {
            child.calculate_size(requested_size, env);
        }

        self.set_dimension(requested_size);

        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.get_position();
        let dimension = self.get_dimension();

        if let Some(child) = self.get_children_mut().next() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl<GS: GlobalState, T: ListIndex + 'static> WidgetExt<GS> for List<GS, T> {}
