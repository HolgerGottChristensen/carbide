use crate::identifiable::AnyIdentifiableWidget;
use crate::list::{IntoSelection, ListSelection, SelectableDelegate, MULTI_SELECTION_MODIFIER};
use carbide::draw::{Dimension, Position};
use carbide::identifiable::Identifiable;
use carbide::random_access_collection::RandomAccessCollection;
use carbide::state::{LocalState, ReadState, State, StateContract};
use carbide::widget::{AnyWidget, CommonWidget, Delegate, ForEach, LazyVStack, MouseArea, MouseAreaActionContext, Scroll, Sequence, Widget, WidgetExt, WidgetId};
use carbide::CommonWidgetImpl;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;


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

// Model based creation
impl List<(), ()> {
    pub fn new<T: StateContract + Identifiable, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>>(
        model: M,
        delegate: U
    ) -> List<ForEach<T, M, U, W, <T as Identifiable>::Id>, ()> {
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

    pub fn new_with_id<T: StateContract, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>, Id: Hash + Eq + Clone + Debug + 'static>(
        model: M,
        id: fn(&T)->Id,
        delegate: U
    ) -> List<ForEach<T, M, U, W, Id>, ()> {
        let content = ForEach::new_with_id(model, id, delegate);

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

    pub fn new_selectable<T: StateContract + Identifiable, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>>(
        model: M,
        selection: impl IntoSelection<<T as Identifiable>::Id>,
        delegate: U
    ) -> List<impl Sequence, <T as Identifiable>::Id> {
        let selection_delegate = SelectableDelegate {
            selection: selection.convert(),
            inner: delegate,
            model: Rc::new(model.clone()),
            ident: <T as Identifiable>::id,
            last_clicked: LocalState::new(None),
            phantom_model: Default::default(),
            phantom_item: Default::default(),
            phantom_widget: Default::default(),
        };

        let content = ForEach::new(model, selection_delegate);

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

    pub fn new_selectable_with_id<T: StateContract, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>, Id: Hash + Eq + Clone + Debug + 'static>(
        model: M,
        id: fn(&T)->Id,
        delegate: U
    ) -> List<impl Sequence, Id> {
        let content = ForEach::new_with_id(model, id, delegate);

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

// Content based creation
impl List<(), ()> {
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
    pub fn selection<SelectionValue2: StateContract + PartialEq + Hash + Eq>(self, selection: impl IntoSelection<SelectionValue2>) -> List<impl Sequence, SelectionValue2> where Content: Sequence<dyn AnyIdentifiableWidget<T=SelectionValue2>> {
        let selection = selection.convert();

        let content = ForEach::custom_widget(self.content, move |inner: &dyn AnyIdentifiableWidget<T=SelectionValue2>| {
            let identifier = inner.identifier().boxed();
            let selection = selection.clone();

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
                                if val == identifier && ctx.modifier_key.contains(MULTI_SELECTION_MODIFIER) {
                                    *id.value_mut() = None;
                                } else {
                                    *id.value_mut() = Some(identifier);
                                }
                            } else {
                                *id.value_mut() = Some(identifier);
                            }
                        }
                        ListSelection::Multi(selections) => {
                            // TODO: Currently not possible because we dont know the model. One possibility is providing self.content and iterating that. Selecting will then be linear in the amount of items selected, and all the selected items must/will be instantiated.
                            match ctx.modifier_key {
                                // If we are holding down GUI (on mac) or CTRL (on windows), add the item
                                // to the set if it does not already contain it. Otherwise remove it from
                                // the set.
                                MULTI_SELECTION_MODIFIER => {
                                    if !selections.value_mut().remove(&identifier) {
                                        selections.value_mut().insert(identifier);
                                    }
                                }
                                // If we are not holding it down, remove all elements from the set and add
                                // the newly clicked element.
                                _ => {
                                    selections.value_mut().clear();
                                    selections.value_mut().insert(identifier);
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