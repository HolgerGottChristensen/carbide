use std::cmp::{max, min};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;

use carbide::a;
use carbide::draw::Rect;
use carbide::state::{AnyReadState, AnyState, Map1};
use carbide::widget::{AnyWidget, MouseArea};
use carbide::widget::canvas::Context;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::ModifierKey;
use carbide_core::state::{
    LocalState, ReadState, State, StateContract,
};
use carbide_core::state::IntoState;
use carbide_core::widget::{Empty, EmptyDelegate};
use carbide_core::widget::{
    CommonWidget, Delegate, EdgeInsets, ForEach, HStack, IfElse, Scroll, VStack, Widget,
    WidgetExt, WidgetId,
};
use carbide_core::widget::canvas::Canvas;

const MULTI_SELECTION_MODIFIER: ModifierKey = if cfg!(target_os = "macos") {
    ModifierKey::SUPER
} else {
    ModifierKey::CONTROL
};
const LIST_SELECTION_MODIFIER: ModifierKey = ModifierKey::SHIFT;

#[derive(Clone, Widget)]
pub struct List<T, M, W, U, I, G>
where
    T: StateContract,
    M: State<T=Vec<T>>,
    W: Widget,
    U: Delegate<T, W>,
    I: StateContract + PartialEq,
    G: Widget,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: Scroll<G>,

    #[state] model: M,
    delegate: U,
    spacing: f64,

    selection: Option<ListSelection<I>>, // TODO: should be marked as state right?
    #[state] last_index_clicked: LocalState<usize>, // Used to make shift selects
    #[allow(unused)]
    tree_disclosure: TreeDisclosure,

    phantom: PhantomData<T>,
    phantom_widget: PhantomData<W>,

    /*child: Box<dyn Widget>,
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
    tree_disclosure: TreeDisclosure,*/
}

impl List<(), Vec<()>, Empty, EmptyDelegate, (), Empty> {
    pub fn new<T: StateContract, M: IntoState<Vec<T>>, W: Widget, U: Delegate<T, W>>(model: M, delegate: U) -> List<T, M::Output, W, U, (), impl Widget> {
        let model = model.into_state();
        let spacing = 10.0;

        let child = Scroll::new(VStack::new(ForEach::new(model.clone(), delegate.clone())).spacing(spacing).boxed());

        List {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child,
            model,
            delegate,
            spacing,
            selection: None,
            last_index_clicked: LocalState::new(0),
            tree_disclosure: TreeDisclosure::Arrow,
            phantom: Default::default(),
            phantom_widget: Default::default(),
        }
    }
}

impl<T: StateContract, M: State<T=Vec<T>>, W: Widget, U: Delegate<T, W>, I: StateContract + PartialEq, G: Widget> List<T, M, W, U, I, G> {
    /// Returns a list selectable where the items within are selectable
    ///
    /// Consumes the `self` argument. It takes an `id` function from the item **T** to the
    /// [`Id`]. It also takes something that can be turned into a selection
    ///
    /// Examples of this is [`Option<Id>`] for single-selection and
    /// [`HashSet<Id>`] for multi-selection.
    pub fn selectable<I2: StateContract + PartialEq + Eq + Hash>(
        self,
        selection: impl Into<ListSelection<I2>>,
    ) -> List<T, M, Box<dyn AnyWidget>, SelectableListDelegate<T, M, W, U, I2>, I2, impl Widget> where T: Identifiable<I2> {
        let selection = selection.into();

        let new_delegate = SelectableListDelegate {
            selection: selection.clone(),
            inner_delegate: self.delegate.clone(),
            last_index_clicked: self.last_index_clicked.clone(),
            model: self.model.clone(),
            phantom: Default::default(),
        };

        let child = Scroll::new(VStack::new(ForEach::new(self.model.clone(), new_delegate.clone())).spacing(self.spacing));

        List {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            child,
            model: self.model,
            delegate: new_delegate,
            spacing: self.spacing,
            selection: Some(selection),
            last_index_clicked: self.last_index_clicked,
            tree_disclosure: TreeDisclosure::Arrow,
            phantom: Default::default(),
            phantom_widget: Default::default(),
        }
    }

    pub fn tree(
        self,
        tree_disclosure: impl Into<TreeDisclosure>,
    ) -> List<T, M, Box<dyn AnyWidget>, TreeListDelegate<T, W, U>, I, impl Widget> where Box<dyn AnyState<T=T>>: Treeable<T> {
        let tree_disclosure = tree_disclosure.into();

        let new_delegate = TreeListDelegate {
            tree_disclosure: tree_disclosure.clone(),
            inner_delegate: self.delegate.clone(),
            phantom: Default::default(),
            phantom2: Default::default(),
        };

        let child = Scroll::new(VStack::new(ForEach::new(self.model.clone(), new_delegate.clone())).spacing(self.spacing));

        List {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            child,
            model: self.model,
            delegate: new_delegate,
            spacing: self.spacing,
            selection: self.selection,
            last_index_clicked: self.last_index_clicked,
            tree_disclosure,
            phantom: Default::default(),
            phantom_widget: Default::default(),
        }
    }

    /*pub fn spacing(mut self, spacing: f64) -> Box<Self> {
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
    }*/

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

impl<T: StateContract, M: State<T=Vec<T>>, W: Widget, U: Delegate<T, W>, I: StateContract + PartialEq, G: Widget> CommonWidget for List<T, M, W, U, I, G> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<T: StateContract, M: State<T=Vec<T>>, W: Widget, U: Delegate<T, W>, I: StateContract + PartialEq, G: Widget> Debug for List<T, M, W, U, I, G> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("List").field("child", &self.child).finish()
    }
}

impl<T: StateContract, M: State<T=Vec<T>>, W: Widget, U: Delegate<T, W>, I: StateContract + PartialEq, G: Widget> WidgetExt for List<T, M, W, U, I, G> {}

pub trait Identifiable<I: StateContract + PartialEq> {
    fn identifier(&self) -> I;
}

impl<T: StateContract + PartialEq> Identifiable<T> for T {
    fn identifier(&self) -> T {
        self.clone()
    }
}

#[derive(Clone, Debug)]
pub enum ListSelection<T: StateContract> {
    Single(LocalState<Option<T>>),
    Multi(LocalState<HashSet<T>>),
}

impl<T: StateContract> Into<ListSelection<T>> for LocalState<Option<T>> {
    fn into(self) -> ListSelection<T> {
        ListSelection::Single(self)
    }
}

impl<T: StateContract> Into<ListSelection<T>> for LocalState<HashSet<T>> {
    fn into(self) -> ListSelection<T> {
        ListSelection::Multi(self)
    }
}

#[derive(Clone)]
pub struct SelectableListDelegate<T, M, W, U, I> where
    T: StateContract + Identifiable<I>,
    M: State<T=Vec<T>>,
    W: Widget,
    U: Delegate<T, W>,
    I: StateContract + PartialEq + Eq + Hash,
{
    selection: ListSelection<I>,
    inner_delegate: U,
    last_index_clicked: LocalState<usize>,
    model: M,
    phantom: PhantomData<W>,
}

impl<T: StateContract + Identifiable<I>, M: State<T=Vec<T>>, W: Widget, U: Delegate<T, W>, I: StateContract + PartialEq + Eq + Hash> Delegate<T, Box<dyn AnyWidget>> for SelectableListDelegate<T, M, W, U, I> {
    fn call(&self, item: Box<dyn AnyState<T=T>>, index: Box<dyn AnyReadState<T=usize>>) -> Box<dyn AnyWidget> {
        let selection = self.selection.clone();
        let last_index_clicked = self.last_index_clicked.clone();
        let model = self.model.clone();

        MouseArea::new(self.inner_delegate.call(item.clone(), index.clone()))
            .on_click(a!(|_, modifier: ModifierKey| {
                let mut selection = selection.clone();
                let identifier = item.value().identifier();

                let model = Clone::clone(&model);
                let model = carbide::state::ReadState::value(&model);

                match &mut selection {
                    // If we are in single selection mode
                    ListSelection::Single(id) => {
                        let val = id.value_mut().clone();

                        // If the value we clicked while holding down GUI (on mac) and Ctrl (on windows)
                        // is the same as already selected, deselect the value. Otherwise select the
                        // item clicked.
                        if let Some(val) = val {
                            if val == identifier && modifier == MULTI_SELECTION_MODIFIER {
                                *id.value_mut() = None;
                            } else {
                                *id.value_mut() = Some(identifier);
                            }
                        } else {
                            *id.value_mut() = Some(identifier);
                        }
                    }
                    ListSelection::Multi(selections) => {
                        match modifier {
                            // If we are holding down GUI (on mac) or CTRL (on windows), add the item
                            // to the set if it does not already contain it. Otherwise remove it from
                            // the set.
                            MULTI_SELECTION_MODIFIER => {
                                if !selections.value_mut().remove(&identifier) {
                                    selections.value_mut().insert(identifier);
                                }
                                *$last_index_clicked = *index.value();
                            }
                            LIST_SELECTION_MODIFIER => {
                                selections.value_mut().clear();
                                let min = min(*index.value(), *$last_index_clicked);
                                let max = max(*index.value(), *$last_index_clicked);

                                for val in min..=max {
                                    //dbg!(&internal_model);
                                    let id = (*model)[val].identifier();

                                    selections.value_mut().insert(id);
                                }
                            }
                            // If we are not holding it down, remove all elements from the set and add
                            // the newly clicked element.
                            _ => {
                                selections.value_mut().clear();
                                selections.value_mut().insert(identifier);
                                *$last_index_clicked = *index.value();
                            }
                        }
                    }
                }

            }))
            .boxed()
    }
}

pub trait Treeable<T: StateContract>: State<T=T> {
    fn children(&self) -> Box<dyn AnyState<T=Vec<T>>>;
}

#[derive(Clone)]
pub struct TreeListDelegate<T, W, U>
where
    T: StateContract,
    W: Widget,
    U: Delegate<T, W>,
{
    tree_disclosure: TreeDisclosure,
    inner_delegate: U,
    phantom: PhantomData<W>,
    phantom2: PhantomData<T>,
}

impl<T: StateContract, W: Widget, U: Delegate<T, W>> Delegate<T, Box<dyn AnyWidget>> for TreeListDelegate<T, W, U> where Box<dyn AnyState<T=T>>: Treeable<T> {
    fn call(&self, item: Box<dyn AnyState<T=T>>, index: Box<dyn AnyReadState<T=usize>>) -> Box<dyn AnyWidget> {
        let widget = self.inner_delegate.call(item.clone(), index.clone());
        let cloned = self.clone();

        let inner_delegate = move |item: Box<dyn AnyState<T=T>>, index: Box<dyn AnyReadState<T=usize>>| -> Box<dyn AnyWidget> {
            let view = cloned.clone().call(item, index);
            view.padding(EdgeInsets::single(0.0, 0.0, 20.0, 0.0)).boxed()
        };

        let opened = LocalState::new(false);

        let disclosure_item = match self.tree_disclosure {
            TreeDisclosure::Arrow => {
                let rotation = Map1::read_map(opened.clone(), |b| if *b { 90.0 } else { 0.0 });

                Canvas::new(|_: Rect, context: &mut Context, _: &mut Environment| {
                    context.move_to(8.0, 5.0);
                    context.line_to(13.0, 10.0);
                    context.line_to(8.0, 15.0);
                    context.set_stroke_style(EnvironmentColor::DarkText);
                    context.set_line_width(1.5);
                    context.stroke();
                })
                    .frame(20.0, 20.0)
                    .rotation_effect(rotation)
                    .boxed()
            }
            TreeDisclosure::Custom(f) => f(opened.clone()),
        };

        let disclosure = MouseArea::new(disclosure_item.clone())
            .on_click(a!(|_, _| {
                *$opened = !*$opened;
            }));

        let sub_tree_model = item.children();

        let sub_tree_model_empty = Map1::read_map(sub_tree_model.clone(), |s| s.is_empty());

        VStack::new((
            HStack::new((
                IfElse::new(sub_tree_model_empty)
                    .when_true(disclosure_item.hidden())
                    .when_false(disclosure),
                widget,
            )).spacing(0.0),
            IfElse::new(opened)
                .when_true(ForEach::new(sub_tree_model, inner_delegate)),
        )).boxed()
    }
}



#[derive(Clone, Debug)]
pub enum TreeDisclosure {
    Arrow,
    Custom(fn(LocalState<bool>) -> Box<dyn AnyWidget>),
}

impl Into<TreeDisclosure> for () {
    fn into(self) -> TreeDisclosure {
        TreeDisclosure::Arrow
    }
}

impl Into<TreeDisclosure> for fn(LocalState<bool>) -> Box<dyn AnyWidget> {
    fn into(self) -> TreeDisclosure {
        TreeDisclosure::Custom(self)
    }
}
