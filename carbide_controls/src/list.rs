use std::fmt::{Debug, Formatter};

use carbide_core::color::{BLUE, RED, TRANSPARENT};
use carbide_core::draw::{Dimension, Position};
use carbide_core::flags::Flags;
use carbide_core::state::{F64State, LocalState, State, StateContract, TState, UsizeState};
use carbide_core::widget::{CommonWidget, Delegate, ForEach, Id, Rectangle, SCALE, Scroll, VStack, Widget, WidgetExt, WidgetIter, WidgetIterMut};

#[derive(Clone, Widget)]
pub struct List<T, U> where T: StateContract + 'static, U: Delegate<T> + 'static {
    id: Id,
    child: Box<dyn Widget>,
    delegate: U,
    position: Position,
    dimension: Dimension,
    spacing: f64,
    #[state]
    model: TState<Vec<T>>,
    #[state]
    internal_model: TState<Vec<T>>,
    #[state]
    index_offset: UsizeState,
    #[state]
    start_offset: F64State,
    #[state]
    end_offset: F64State,
}

impl<T: StateContract + 'static, U: Delegate<T> + 'static> List<T, U> {
    pub fn new<V: Into<TState<Vec<T>>>>(model: V, delegate: U) -> Box<Self> {
        let index_offset_state = LocalState::new(0 as usize);

        let start_offset = LocalState::new(-10.0);
        let end_offset = LocalState::new(-10.0);

        let internal_model = model.into();

        Box::new(List {
            id: Id::new_v4(),
            child: Scroll::new(
                VStack::new(vec![
                    Rectangle::new()
                        .fill(TRANSPARENT) // RED for debugging
                        .frame(SCALE, start_offset.clone()),
                    ForEach::new(internal_model.clone(), delegate.clone()),
                    //.index_offset(index_offset_state.clone()),
                    Rectangle::new()
                        .fill(TRANSPARENT)// BLUE for debugging
                        .frame(SCALE, end_offset.clone()),
                ])
                    .spacing(10.0),
            ),
            delegate,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            spacing: 10.0,
            model: internal_model.clone(),
            internal_model,
            index_offset: index_offset_state.into(),
            start_offset: start_offset.into(),
            end_offset: end_offset.into(),
        })
    }

    pub fn spacing(mut self, spacing: f64) -> Box<Self> {
        self.spacing = spacing;
        let start_offset = LocalState::new(-spacing);
        let end_offset = LocalState::new(-spacing);
        self.start_offset = start_offset.into();
        self.end_offset = end_offset.into();
        self.child = Scroll::new(
            VStack::new(vec![
                Rectangle::new()
                    .fill(TRANSPARENT)
                    .frame(SCALE, self.start_offset.clone()),
                ForEach::new(self.internal_model.clone(), self.delegate.clone()),
                //.index_offset(self.index_offset.clone()),
                Rectangle::new()
                    .fill(TRANSPARENT)
                    .frame(SCALE, self.end_offset.clone()),
            ])
                .spacing(spacing),
        );
        Box::new(self)
    }
    /*
        fn _recalculate_visible_children(&mut self, env: &mut Environment<GS>) {
            // TODO: Handle when model changes.
            // If items in the internal model is removed, calculate new sizes, if items in between the items in the internal_model is added, do ???

            let spacing = self.spacing;

            let dimension = self.dimension();

            // The y position of the list
            let y_position = self.y();

            // The height of the list
            let height = self.height();

            // The offset at the beginning of the list.
            let mut start_offset = self.start_offset.clone();

            // The offset at the end of the list
            let mut end_offset = self.end_offset.clone();

            // The internal model that is actually shown in the foreach
            let mut internal_model = self.internal_model.clone();

            // The model of all the items to scroll through in the list
            let model = self.model.clone();

            // Index offset for the first visible element in the foreach
            let mut index_offset = self.index_offset.clone();

            // Notice the scroll might not be the first when clip is added.
            if let Some(scroll_view) = self.children_mut().next() {
                // Calculate the size off all the children.
                scroll_view.calculate_size(dimension, env);

                // Position all the children. This will not be their final position though, just relative
                // to the current position of the list view, but this does not matter.
                scroll_view.position_children();

                if let Some(vstack) = scroll_view.children_mut().next() {
                    // The number of elements in currently in the internal model.
                    let internal_model_size = internal_model.get_latest_value().len();

                    // Get the children of the scrollview
                    let mut vstack_children = vstack.children_mut().enumerate();

                    // The first child is the initial_offset Rectangle.
                    vstack_children.next();

                    // Handle remove items out of view
                    while let Some((index, child)) = vstack_children.next() {
                        // Skip the last element in the v_stack, which is the end rectangle
                        if index > internal_model_size {
                            continue;
                        }

                        // Check if an items bottom is above the top of this view.
                        if child.y() + child.height() < y_position {
                            // Increase the start_offset height with the height of the child and spacing.
                            *start_offset.get_latest_value_mut() += child.height() + spacing;
                            // Remove the first element from the internal model.
                            internal_model.get_latest_value_mut().remove(0);
                            // Increase the start_index_offset with 1.
                            *index_offset.get_latest_value_mut() += 1;
                        }

                        // Check if an item is below the list
                        if child.y() > y_position + height {
                            // Increase the end_offset with the child's height and additional spacing
                            *end_offset.get_latest_value_mut() += child.height() + spacing;
                            // Pop the last element from the internal model
                            internal_model.get_latest_value_mut().pop();
                        }
                    }

                    // Get the children of the scrollview
                    let mut vstack_children = vstack.children_mut().enumerate();

                    let (_, initial_child) = vstack_children
                        .next()
                        .expect("Could not find the initial rectangle");

                    // TODO: Consider choosing -scroll_offset for this value.
                    let inital_y = initial_child.y();

                    let mut min_height = 1000.0;

                    // Calculate the minimum height of an index currently in the list.
                    while let Some((index, child)) = vstack_children.next() {
                        if index > internal_model_size {
                            continue;
                        }
                        if child.height() < min_height {
                            min_height = child.height();
                        }
                    }

                    // Handle add items to view from the top
                    while *start_offset.get_latest_value() + inital_y > y_position {
                        *start_offset.get_latest_value_mut() -= min_height + spacing;
                        *index_offset.get_latest_value_mut() -= 1;
                        let index_to_take_from = *index_offset.get_latest_value();

                        if model.get_latest_value().len() < index_to_take_from {
                            panic!("Can not take index from model, that is outside the bounds of said model")
                        }

                        internal_model
                            .get_latest_value_mut()
                            .insert(0, model.get_latest_value()[index_to_take_from].clone());
                    }

                    let mut vstack_children = vstack.children_mut();

                    let bottom_of_list_widget = y_position + height;

                    let mut top_of_end_rectangle = 0.0;

                    while let Some(vstack_child) = vstack_children.next() {
                        top_of_end_rectangle = vstack_child.y();
                        //println!("{}", top_of_end_rectangle);
                    }

                    // Handle add items to view from the bottom
                    while top_of_end_rectangle < bottom_of_list_widget {
                        //println!("--------------");
                        //println!("Minimum height: {}", min_height);
                        //println!("Top of end rectangle: {}", top_of_end_rectangle);
                        //println!("Bottom of list: {}", bottom_of_list_widget);

                        //println!("Top of end rectangle AFTER: {}", top_of_end_rectangle);

                        let index_to_take_from =
                            *index_offset.get_latest_value() + internal_model.get_latest_value().len();

                        //println!("Index offset: {}", *self_index_offset.get_latest_value());
                        //println!("Internal model length: {}", internal_model.get_latest_value().len());

                        //if model.get_latest_value().len() < index_to_take_from {
                        //    panic!("Can not take index from model, that is outside the bounds of said model")
                        //}

                        if model.get_latest_value().len() <= index_to_take_from {
                            break;
                        } else {
                            top_of_end_rectangle += min_height + spacing;
                            *end_offset.get_latest_value_mut() -= min_height + spacing;
                            internal_model
                                .get_latest_value_mut()
                                .push(model.get_latest_value()[index_to_take_from].clone());
                        }
                    }
                }
            }

            self.start_offset = start_offset;
            self.end_offset = end_offset;
            self.internal_model = internal_model;
            self.index_offset = index_offset;

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
    */
}

impl<T: StateContract + 'static, U: Delegate<T> + 'static> CommonWidget for List<T, U> {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<T: StateContract, U: Delegate<T>> Debug for List<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("List")
            .field("child", &self.child)
            .finish()
    }
}

impl<T: StateContract + 'static, U: Delegate<T> + 'static> WidgetExt for List<T, U> {}
