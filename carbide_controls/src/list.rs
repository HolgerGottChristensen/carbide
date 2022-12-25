use std::borrow::Borrow;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use carbide_core::color::TRANSPARENT;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::ModifierKey;
use carbide_core::flags::Flags;
use carbide_core::state::{
    LocalState, ReadState, State, StateContract, StateExt, TState, ValueState,
};
use carbide_core::widget::{
    CommonWidget, Delegate, EdgeInsets, ForEach, HStack, IfElse, Rectangle, Scroll, VStack, Widget,
    WidgetExt, WidgetId, WidgetIter, WidgetIterMut,
};
use carbide_core::widget::canvas::Canvas;

use crate::PlainButton;

const MULTI_SELECTION_MODIFIER: ModifierKey = if cfg!(target_os = "macos") {
    ModifierKey::GUI
} else {
    ModifierKey::CTRL
};
const LIST_SELECTION_MODIFIER: ModifierKey = ModifierKey::SHIFT;

#[derive(Clone, Widget)]
pub struct List<T, U>
where
    T: StateContract,
    U: Delegate<T> + 'static,
{
    id: WidgetId,
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
    index_offset: TState<usize>,
    #[state]
    start_offset: TState<f64>,
    #[state]
    end_offset: TState<f64>,
    item_id_function: Option<fn(&T) -> WidgetId>,
    selection: Option<Selection>,
    #[state]
    last_index_clicked: TState<usize>,
    sub_tree_function: Option<fn(TState<T>) -> TState<Option<Vec<T>>>>,
    tree_disclosure: TreeDisclosure,
}

impl<T: StateContract, U: Delegate<T> + 'static> List<T, U> {
    pub fn new(model: impl Into<TState<Vec<T>>>, delegate: U) -> Box<Self> {
        let index_offset_state = LocalState::new(0 as usize);

        let start_offset = LocalState::new(-10.0);
        let end_offset = LocalState::new(-10.0);

        let internal_model = model.into();

        Box::new(List {
            id: WidgetId::new(),
            child: Scroll::new(
                VStack::new(vec![
                    Rectangle::new()
                        .fill(TRANSPARENT) // RED for debugging
                        .frame(0.0, start_offset.clone())
                        .expand_width(),
                    ForEach::new(internal_model.clone(), delegate.clone()),
                    //.index_offset(index_offset_state.clone()),
                    Rectangle::new()
                        .fill(TRANSPARENT) // BLUE for debugging
                        .frame(0.0, end_offset.clone())
                        .expand_width(),
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
            item_id_function: None,
            selection: None,
            last_index_clicked: ValueState::new(0),
            sub_tree_function: None,
            tree_disclosure: TreeDisclosure::Arrow,
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
                    .frame(0.0, self.start_offset.clone())
                    .expand_width(),
                ForEach::new(self.internal_model.clone(), self.delegate.clone()),
                //.index_offset(self.index_offset.clone()),
                Rectangle::new()
                    .fill(TRANSPARENT)
                    .frame(0.0, self.end_offset.clone())
                    .expand_width(),
            ])
            .spacing(spacing),
        );
        Box::new(self)
    }

    pub fn tree(
        mut self,
        children: fn(TState<T>) -> TState<Option<Vec<T>>>,
        tree_disclosure: impl Into<TreeDisclosure>,
    ) -> Box<List<T, TreeListDelegate<T, U>>> {
        let tree_disclosure = tree_disclosure.into();

        let new_delegate = TreeListDelegate {
            sub_tree_function: children,
            tree_disclosure: tree_disclosure.clone(),
            inner_delegate: self.delegate,
        };

        let child = Scroll::new(
            VStack::new(vec![
                Rectangle::new()
                    .fill(TRANSPARENT)
                    .frame(0.0, self.start_offset.clone())
                    .expand_width(),
                ForEach::new(self.internal_model.clone(), new_delegate.clone()),
                Rectangle::new()
                    .fill(TRANSPARENT)
                    .frame(0.0, self.end_offset.clone())
                    .expand_width(),
            ])
            .spacing(self.spacing),
        );

        Box::new(List {
            id: self.id,
            child,
            delegate: new_delegate,
            position: Default::default(),
            dimension: Default::default(),
            spacing: self.spacing,
            model: self.model,
            internal_model: self.internal_model,
            index_offset: self.index_offset,
            start_offset: self.start_offset,
            end_offset: self.end_offset,
            item_id_function: self.item_id_function,
            selection: self.selection,
            last_index_clicked: self.last_index_clicked,
            sub_tree_function: Some(children),
            tree_disclosure,
        })
    }

    /// Returns a list selectable where the items within are selectable
    ///
    /// Consumes the `self` argument. It takes an `id` function from the item **T** to the
    /// [`Id`]. It also takes something that can be turned into a selection
    ///
    /// Examples of this is [`Option<Id>`] for single-selection and
    /// [`HashSet<Id>`] for multi-selection.
    pub fn selectable(
        mut self,
        id: fn(&T) -> WidgetId,
        selection: impl Into<Selection>,
    ) -> Box<List<T, SelectableListDelegate<T, U>>> {
        let selection = selection.into();
        let last_index_clicked = LocalState::new(0);

        let new_delegate = SelectableListDelegate {
            item_id_function: id,
            selection: selection.clone(),
            inner_delegate: self.delegate,
            last_selected_index: last_index_clicked.clone(),
            internal_model: self.internal_model.clone(),
        };

        let child = Scroll::new(
            VStack::new(vec![
                Rectangle::new()
                    .fill(TRANSPARENT)
                    .frame(0.0, self.start_offset.clone())
                    .expand_width(),
                ForEach::new(self.internal_model.clone(), new_delegate.clone()),
                Rectangle::new()
                    .fill(TRANSPARENT)
                    .frame(0.0, self.end_offset.clone())
                    .expand_width(),
            ])
            .spacing(self.spacing),
        );

        Box::new(List {
            id: self.id,
            child,
            delegate: new_delegate,
            position: Default::default(),
            dimension: Default::default(),
            spacing: self.spacing,
            model: self.model,
            internal_model: self.internal_model,
            index_offset: self.index_offset,
            start_offset: self.start_offset,
            end_offset: self.end_offset,
            item_id_function: Some(id),
            selection: Some(selection.clone()),
            last_index_clicked,
            sub_tree_function: None,
            tree_disclosure: self.tree_disclosure,
        })
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

impl<T: StateContract, U: Delegate<T> + 'static> CommonWidget for List<T, U> {
    fn id(&self) -> WidgetId {
        self.id
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
        f.debug_struct("List").field("child", &self.child).finish()
    }
}

impl<T: StateContract, U: Delegate<T> + 'static> WidgetExt for List<T, U> {}

#[derive(Clone)]
pub struct TreeListDelegate<T, U>
where
    T: StateContract,
    U: Delegate<T> + 'static,
{
    sub_tree_function: fn(TState<T>) -> TState<Option<Vec<T>>>,
    tree_disclosure: TreeDisclosure,
    inner_delegate: U,
}

impl<T: StateContract, U: Delegate<T> + 'static> Delegate<T> for TreeListDelegate<T, U> {
    fn call(&self, item: TState<T>, index: TState<usize>) -> Box<dyn Widget> {
        let widget = self.inner_delegate.call(item.clone(), index.clone());

        let cloned = self.clone();
        let inner_delegate = move |item: TState<T>, index: TState<usize>| -> Box<dyn Widget> {
            let view = cloned.clone().call(item, index);
            view.padding(EdgeInsets::single(0.0, 0.0, 20.0, 0.0))
        };

        let opened = LocalState::new(false);

        let disclosure_item: Box<dyn Widget> = match self.tree_disclosure {
            TreeDisclosure::Arrow => {
                let rotation = opened.mapped(|b: &bool| if *b { 90.0 } else { 0.0 });

                Canvas::new(|_, mut context, _| {
                    context.move_to(8.0, 5.0);
                    context.line_to(13.0, 10.0);
                    context.line_to(8.0, 15.0);
                    context.set_stroke_style(EnvironmentColor::DarkText);
                    context.set_line_width(1.5);
                    context.stroke();

                    context
                })
                .frame(20.0, 20.0)
                .rotation_effect(rotation)
            }
            TreeDisclosure::Custom(f) => f(opened.clone()),
        };

        let disclosure = PlainButton::new(disclosure_item.clone())
            .on_click(capture!([opened], |env: &mut Environment| {
                *opened = !*opened
            }));

        let sub_tree_model = (self.sub_tree_function)(item);

        VStack::new(vec![
            HStack::new(vec![
                IfElse::new(sub_tree_model.is_some().ignore_writes())
                    .when_true(disclosure)
                    .when_false(disclosure_item.hidden()),
                widget,
            ])
            .spacing(0.0),
            IfElse::new(opened).when_true(ForEach::new(
                sub_tree_model.unwrap_or_default(),
                inner_delegate,
            )),
        ])
    }
}

#[derive(Clone)]
pub struct SelectableListDelegate<T, U>
where
    T: StateContract,
    U: Delegate<T> + 'static,
{
    item_id_function: fn(&T) -> WidgetId,
    selection: Selection,
    inner_delegate: U,
    last_selected_index: TState<usize>,
    internal_model: TState<Vec<T>>,
}

impl<T: StateContract, U: Delegate<T> + 'static> Delegate<T> for SelectableListDelegate<T, U> {
    fn call(&self, item: TState<T>, index: TState<usize>) -> Box<dyn Widget> {
        let selection = self.selection.clone();
        let last_selected_index = self.last_selected_index.clone();
        let internal_model = self.internal_model.clone();
        let id_function = self.item_id_function;

        PlainButton::new(self.inner_delegate.call(item.clone(), index.clone())).on_click(
            move |env: &mut Environment, modifier: ModifierKey| {
                let mut selection = selection.clone();
                let mut last_selected_index = last_selected_index.clone();
                let value = id_function(&*item.value());

                match &mut selection {
                    // If we are in single selection mode
                    Selection::Single(id) => {
                        let val = id.value_mut().clone();

                        // If the value we clicked while holding down GUI (on mac) and Ctrl (on windows)
                        // is the same as already selected, deselect the value. Otherwise select the
                        // item clicked.
                        if let Some(val) = val {
                            if val == value && modifier == MULTI_SELECTION_MODIFIER {
                                *id.value_mut() = None;
                            } else {
                                *id.value_mut() = Some(value);
                            }
                        } else {
                            *id.value_mut() = Some(value);
                        }
                    }
                    // If we are in multi-select mode
                    Selection::Multi(selections) => {
                        match modifier {
                            // If we are holding down GUI (on mac) or CTRL (on windows), add the item
                            // to the set if it does not already contain it. Otherwise remove it from
                            // the set.
                            MULTI_SELECTION_MODIFIER => {
                                if !selections.value_mut().remove(&value) {
                                    selections.value_mut().insert(value);
                                }
                                *last_selected_index.value_mut() = *index.value();
                            }
                            LIST_SELECTION_MODIFIER => {
                                selections.value_mut().clear();
                                let min = min(*index.value(), *last_selected_index.value());
                                let max = max(*index.value(), *last_selected_index.value());

                                for val in min..=max {
                                    //dbg!(&internal_model);
                                    let id = id_function(&internal_model.value()[val]);

                                    selections.value_mut().insert(id);
                                }
                            }
                            // If we are not holding it down, remove all elements from the set and add
                            // the newly clicked element.
                            _ => {
                                selections.value_mut().clear();
                                selections.value_mut().insert(value);
                                *last_selected_index.value_mut() = *index.value();
                            }
                        }
                    }
                }
            },
        )
    }
}

#[derive(Clone, Debug)]
pub enum Selection {
    Single(TState<Option<WidgetId>>),
    Multi(TState<HashSet<WidgetId>>),
}

impl Into<Selection> for TState<Option<WidgetId>> {
    fn into(self) -> Selection {
        Selection::Single(self)
    }
}

impl Into<Selection> for TState<HashSet<WidgetId>> {
    fn into(self) -> Selection {
        Selection::Multi(self)
    }
}

#[derive(Clone, Debug)]
pub enum TreeDisclosure {
    Arrow,
    Custom(fn(TState<bool>) -> Box<dyn Widget>),
}

impl Into<TreeDisclosure> for () {
    fn into(self) -> TreeDisclosure {
        TreeDisclosure::Arrow
    }
}

impl Into<TreeDisclosure> for fn(TState<bool>) -> Box<dyn Widget> {
    fn into(self) -> TreeDisclosure {
        TreeDisclosure::Custom(self)
    }
}
