use std::cmp::{max, min};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;
use carbide::random_access_collection::RandomAccessCollection;
use carbide::state::{AnyReadState, LocalState, ReadState, State, StateContract};
use carbide::widget::{Delegate, MouseAreaAction, MouseAreaActionContext, Widget, WidgetProperties};
use carbide::widget::properties::WidgetKindSimple;
use crate::list::{ListSelection, MULTI_SELECTION_MODIFIER, LIST_SELECTION_MODIFIER, LIST_SELECTION_AND_MULTI_SELECTION_MODIFIER};
use crate::list::list_row::ListRow;

#[derive(Clone)]
pub(crate) struct SelectableDelegate<D, M, T, W, Id> where
    D: Delegate<M, T, W>,
    T: StateContract,
    M: RandomAccessCollection<T>,
    W: Widget + WidgetProperties<Kind=WidgetKindSimple>,
    Id: Hash + Eq + Clone + Debug + 'static
{
    pub selection: ListSelection<Id>,
    pub inner: D,
    pub model: Rc<M>,
    pub ident: fn(&T) -> Id,
    pub last_clicked: LocalState<Option<M::Idx>>,

    pub phantom_model: PhantomData<M>,
    pub phantom_item: PhantomData<T>,
    pub phantom_widget: PhantomData<W>
}

impl<D, M, T, W, Id> Delegate<M, T, ListRow<Id, T, M, W>> for SelectableDelegate<D, M, T, W, Id>
where
    D: Delegate<M, T, W>,
    T: StateContract,
    M: RandomAccessCollection<T>,
    W: Widget + WidgetProperties<Kind=WidgetKindSimple>,
    Id: Hash + Eq + Clone + Debug + 'static
{
    fn call<'a>(&'a self, item: M::Item<'a>, index: Box<dyn AnyReadState<T=M::Idx>>) -> ListRow<Id, T, M, W> {
        let inner_widget = self.inner.call(item, index.clone());

        ListRow {
            selection: self.selection.clone(),
            ident: self.ident,
            model: self.model.clone(),
            index,
            last_clicked: self.last_clicked.clone(),

            hovered: LocalState::new(false),
            pressed: LocalState::new(false),
            child: inner_widget,
        }
    }
}

#[derive(Clone)]
pub(crate) struct SelectableMouseAreaAction<Id, T, M> where Id: Hash + Eq + Clone + Debug + 'static, T: StateContract, M: RandomAccessCollection<T> {
    pub(crate) selection: ListSelection<Id>,
    pub(crate) ident: fn(&T) -> Id,
    pub(crate) model: Rc<M>,
    pub(crate) index: Box<dyn AnyReadState<T=M::Idx>>,
    pub(crate) last_clicked: LocalState<Option<M::Idx>>,
}

impl<Id: Hash + Eq + Clone + Debug + 'static, T: StateContract, M: RandomAccessCollection<T>> MouseAreaAction for SelectableMouseAreaAction<Id, T, M> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        let index = self.index.value().clone();
        let identifier = self.model.map(index.clone(), self.ident);

        match &mut self.selection {
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

                *self.last_clicked.value_mut() = Some(index);
            }
            ListSelection::Multi(selections) => {
                match ctx.modifier_key {
                    // If we are holding down GUI (on mac) or CTRL (on windows), add the item
                    // to the set if it does not already contain it. Otherwise remove it from
                    // the set.
                    MULTI_SELECTION_MODIFIER | LIST_SELECTION_AND_MULTI_SELECTION_MODIFIER => {
                        // Consider: LIST_SELECTION_AND_MULTI_SELECTION_MODIFIER could deselect range
                        if !selections.value_mut().remove(&identifier) {
                            selections.value_mut().insert(identifier);
                        }
                        *self.last_clicked.value_mut() = Some(index);
                    }
                    LIST_SELECTION_MODIFIER => {
                        if let Some(last_clicked) = self.last_clicked.value().clone() {
                            let model_start = self.model.start_index();
                            let model_end = self.model.end_index();
                            let selections = &mut *selections.value_mut();

                            // Deselect old selection range by removing the connected range above and below

                            if last_clicked.clone() <= index.clone() { // Clear upward
                                let mut current = self.model.prev_index(last_clicked.clone());

                                while current >= model_start {
                                    let identifier = self.model.map(current.clone(), self.ident);

                                    if selections.contains(&identifier) {
                                        selections.remove(&identifier);
                                    } else {
                                        break;
                                    }

                                    current = self.model.prev_index(current);
                                }
                            }

                            if last_clicked.clone() >= index.clone() { // Clear downward
                                let mut current = last_clicked.clone();

                                while current < model_end {
                                    let identifier = self.model.map(current.clone(), self.ident);

                                    if selections.contains(&identifier) {
                                        selections.remove(&identifier);
                                    } else {
                                        break;
                                    }

                                    current = self.model.next_index(current);
                                }
                            }



                            // Select new items
                            let start = min(min(index.clone(), last_clicked.clone()), self.model.end_index());
                            let end = max(max(self.model.next_index(index), self.model.next_index(last_clicked)), self.model.start_index());

                            let mut current = start;

                            while current < end {
                                let identifier = self.model.map(current.clone(), self.ident);
                                selections.insert(identifier);
                                current = self.model.next_index(current);
                            }
                        } else {
                            selections.value_mut().clear();
                            selections.value_mut().insert(identifier);
                            *self.last_clicked.value_mut() = Some(index);
                        }
                    }
                    // If we are not holding it down, remove all elements from the set and add
                    // the newly clicked element.
                    _ => {
                        selections.value_mut().clear();
                        selections.value_mut().insert(identifier);
                        *self.last_clicked.value_mut() = Some(index);
                    }
                }
            }
        }
    }
}