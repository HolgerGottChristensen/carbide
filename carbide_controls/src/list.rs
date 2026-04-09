use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use carbide::CommonWidgetImpl;
use carbide::draw::{Dimension, Position};
use carbide::event::ModifierKey;
use carbide::identifiable::Identifiable;
use carbide::random_access_collection::RandomAccessCollection;
use carbide::state::{LocalState, ReadState, State, StateContract};
use carbide::widget::{AnyWidget, CommonWidget, Delegate, ForEach, LazyVStack, MouseArea, MouseAreaActionContext, Scroll, Sequence, Widget, WidgetExt, WidgetId};
use crate::identifiable::AnyIdentifiableWidget;

const MULTI_SELECTION_MODIFIER: ModifierKey = if cfg!(target_os = "macos") {
    ModifierKey::SUPER
} else {
    ModifierKey::CONTROL
};
const LIST_SELECTION_MODIFIER: ModifierKey = ModifierKey::SHIFT;

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

impl<T: StateContract + Hash + Eq> Into<ListSelection<T>> for LocalState<HashSet<T>> {
    fn into(self) -> ListSelection<T> {
        ListSelection::Multi(self)
    }
}

#[derive(Clone, Widget)]
pub struct List<Content, SelectionValue>
where
    Content: Sequence,
    SelectionValue: StateContract + PartialEq,
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    content: Content,
    child: Box<dyn AnyWidget>,

    spacing: f64,

    phantom_data: PhantomData<SelectionValue>
}

impl List<(), ()> {
    pub fn new<T: StateContract + Identifiable, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>>(model: M, delegate: U) -> List<ForEach<T, M, U, W, <T as Identifiable>::Id>, ()> {
        let content = ForEach::new(model, delegate);

        let child = Scroll::new(
            LazyVStack::new(
                content.clone()
            ).spacing(1.0)
        ).clip();

        List {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            content,
            child: child.boxed(),
            spacing: 1.0,
            phantom_data: Default::default(),
        }
    }

    pub fn new_content<Content: Sequence>(content: Content) -> List<Content, ()> {
        let child = Scroll::new(
            LazyVStack::new(
                content.clone()
            ).spacing(1.0)
        ).clip();

        List {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            content,
            child: child.boxed(),
            spacing: 1.0,
            phantom_data: Default::default(),
        }
    }
}

impl<Content: Sequence> List<Content, ()> { // Using () as the type here, we ensure you only can call the .selection() once

    pub fn selection<SelectionValue2: StateContract + PartialEq + Hash + Eq>(self, selection: impl Into<ListSelection<SelectionValue2>>) -> List<impl Sequence, SelectionValue2> where Content: Sequence<dyn AnyIdentifiableWidget<T=SelectionValue2>> {
        let selection = selection.into();

        let last_selected_index = LocalState::new(0usize);

        let content = ForEach::custom_widget(self.content, move |inner: &dyn AnyIdentifiableWidget<T=SelectionValue2>| {
            let identifier = inner.identifier().boxed();
            let selection = selection.clone();
            let last_selected_index = last_selected_index.clone();

            MouseArea::new(inner.as_widget().boxed())
                .on_click(move |ctx: MouseAreaActionContext| {
                    let mut selection = selection.clone();
                    let identifier = identifier.value().clone();

                    match &mut selection {
                        // If we are in single selection mode
                        ListSelection::Single(id) => {
                            let val = id.value_mut().clone();

                            // If the value we clicked while holding down GUI (on mac) and Ctrl (on windows)
                            // is the same as already selected, deselect the value. Otherwise select the
                            // item clicked.
                            if let Some(val) = val {
                                if val == identifier && ctx.modifier_key == MULTI_SELECTION_MODIFIER {
                                    *id.value_mut() = None;
                                } else {
                                    *id.value_mut() = Some(identifier);
                                }
                            } else {
                                *id.value_mut() = Some(identifier);
                            }
                        }
                        ListSelection::Multi(selections) => {
                            match ctx.modifier_key {
                                // If we are holding down GUI (on mac) or CTRL (on windows), add the item
                                // to the set if it does not already contain it. Otherwise remove it from
                                // the set.
                                MULTI_SELECTION_MODIFIER => {
                                    if !selections.value_mut().remove(&identifier) {
                                        selections.value_mut().insert(identifier);
                                    }
                                    //*$last_index_clicked = *index.value();
                                }
                                LIST_SELECTION_MODIFIER => {
                                    // TODO: Currently not possible because we dont know the model. One possibility is providing self.content and iterating that. Selecting will then be linear in the amount of items selected, and all the selected items must be instantiated.
                                    /*selections.value_mut().clear();
                                    let min = min(*index.value(), *$last_index_clicked);
                                    let max = max(*index.value(), *$last_index_clicked);
                                    for val in min..=max {
                                        //dbg!(&internal_model);
                                        let id = (*model)[val].id();
                                    }
                                    selections.value_mut().insert(id);
                                    }*/
                                }
                                // If we are not holding it down, remove all elements from the set and add
                                // the newly clicked element.
                                _ => {
                                    selections.value_mut().clear();
                                    selections.value_mut().insert(identifier);
                                    //*$last_index_clicked = *index.value();
                                }
                            }
                        }
                    }
                })
        });

        let child = Scroll::new(
            LazyVStack::new(
                content.clone()
            ).spacing(self.spacing)
        ).clip();

        List {
            id: self.id,
            position: self.position,
            dimension: self.dimension,
            content,
            child: child.boxed(),
            spacing: self.spacing,
            phantom_data: Default::default(),
        }
    }
}

impl<SelectionValue: StateContract + PartialEq, Content: Sequence> CommonWidget for List<Content, SelectionValue> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}

impl<SelectionValue: StateContract + PartialEq, Content: Sequence> Debug for List<Content, SelectionValue> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("List")
            .field("child", &self.child)
            .finish()
    }
}