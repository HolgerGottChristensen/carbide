use crate::common::flags::WidgetFlag;
use crate::draw::{Dimension, Position};
use crate::identifiable::Identifiable;
use crate::random_access_collection::RandomAccessCollection;
use crate::state::{AnyReadState, LocalState, State, StateContract};
use crate::widget::foreach_widget::Delegate as ForEachChildDelegate;
use crate::widget::foreach_widget::ForEachWidget;
use crate::widget::properties::{WidgetKind, WidgetKindProxy};
use crate::widget::{AnySequence, AnyWidget, CommonWidget, Empty, Sequence as ForEachSequence, Widget, WidgetExt, WidgetId, WidgetProperties, WidgetSync};
use carbide::widget::properties::Kind;
use dyn_clone::DynClone;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;

pub trait Delegate<M: RandomAccessCollection<T>, T: StateContract, O: Widget>: Clone + 'static {
    fn call<'a>(&'a self, item: M::Item<'a>, index: Box<dyn AnyReadState<T=M::Idx>>) -> O;
}

impl<M: RandomAccessCollection<T>, T: StateContract, K, O: Widget> Delegate<M, T, O> for K where K: Fn(M::Item<'_>, Box<dyn AnyReadState<T=M::Idx>>) -> O + Clone + 'static {
    fn call<'b>(&self, item: M::Item<'b>, index: Box<dyn AnyReadState<T=M::Idx>>) -> O {
        self(item, index)
    }
}

#[derive(Clone)]
pub struct EmptyDelegate;

impl Delegate<Vec<()>, (), Empty> for EmptyDelegate {
    fn call(&self, _: &(), _: Box<dyn AnyReadState<T=usize>>) -> Empty {
        Empty::new()
    }
}


#[derive(Widget)]
#[carbide_exclude(Properties)]
pub struct ForEach<T, M, U, W, Id>
where
    T: StateContract,
    M: RandomAccessCollection<T>,
    W: Widget,
    U: Delegate<M, T, W>,
    Id: Hash + Eq + Clone + Debug + 'static
{
    #[id] id: WidgetId,

    model: M,
    delegate: U,

    widgets: HashMap<Id, W>,
    indices: HashMap<Id, LocalState<M::Idx>>,

    phantom: PhantomData<T>,
    ident: fn(&T) -> Id,
}

impl ForEach<(), Vec<()>, EmptyDelegate, Empty, ()> {
    pub fn new<T: StateContract + Identifiable, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>>(model: M, delegate: U) -> ForEach<T, M, U, W, T::Id> {
        ForEach {
            id: WidgetId::new(),
            model,
            delegate,
            widgets: HashMap::new(),
            indices: HashMap::new(),
            phantom: PhantomData::default(),
            ident: T::id,
        }
    }

    pub fn new_with_id<T: StateContract, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>, Id: Hash + Eq + Clone + Debug + 'static>(model: M, id: fn(&T)->Id, delegate: U) -> ForEach<T, M, U, W, Id> {
        ForEach {
            id: WidgetId::new(),
            model,
            delegate,
            widgets: HashMap::new(),
            indices: HashMap::new(),
            phantom: PhantomData::default(),
            ident: id,
        }
    }

    pub fn widget<Sequence: ForEachSequence, Output: Widget, Delegate: ForEachChildDelegate<dyn AnyWidget, Output>>(of: Sequence, with: Delegate) -> ForEachWidget<Sequence, Output, Delegate, dyn AnyWidget> {
        ForEachWidget::new(of, with)
    }

    pub fn identity<Sequence: ForEachSequence>(of: Sequence) -> ForEachWidget<Sequence, Box<dyn AnyWidget>, fn(&dyn AnyWidget)->Box<dyn AnyWidget>, dyn AnyWidget> {
        fn identity(child: &dyn AnyWidget) -> Box<dyn AnyWidget> {
            child.boxed()
        }
        ForEachWidget::new(of, identity)
    }

    pub fn custom_widget<Item: ?Sized + Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static, Sequence: ForEachSequence<Item>, Output: Widget, Delegate: ForEachChildDelegate<Item, Output>>(of: Sequence, with: Delegate) -> ForEachWidget<Sequence, Output, Delegate, Item> {
        ForEachWidget::new(of, with)
    }
}

impl<T, M, U, W, Id> ForEach<T, M, U, W, Id>
where
    T: StateContract,
    M: RandomAccessCollection<T>,
    W: Widget,
    U: Delegate<M, T, W>,
    Id: Hash + Eq + Clone + Debug + 'static
{
    fn ensure_exist(&mut self, index: M::Idx) {
        let id = self.model.map(index.clone(), self.ident);

        if let Some(i) = self.indices.get_mut(&id) {
            *i.value_mut() = index.clone();
        } else {
            self.indices.insert(id.clone(), LocalState::new(index.clone()));
        }

        if self.widgets.get(&id).is_none() {
            let item = self.model.index(self.indices.get(&id).unwrap().clone());
            let i = self.indices.get(&id).unwrap();

            let new = self.delegate.call(item, Box::new(i.clone()));

            self.widgets.insert(id, new);
        }
    }
}

impl<T: StateContract, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>, Id: Hash + Eq + Clone + Debug + 'static> ForEach<T, M, U, W, Id> {
    pub fn child<A: ?Sized>(&mut self, index: usize) -> &mut A where W: AnySequence<A> {
        if W::Kind::kind() == Kind::Simple {
            let idx = self.model.index_from_offset(index);
            self.ensure_exist(idx.clone());
            let id = self.model.map(idx, self.ident);

            <W as AnySequence<A>>::index(self.widgets.get_mut(&id).unwrap(), 0)
        } else {
            let mut current_index = self.model.start_index();
            let end_index = self.model.end_index();

            let mut passed = 0;

            while current_index < end_index {
                let id = self.model.map(current_index.clone(), self.ident);

                self.ensure_exist(current_index.clone());

                let child = self.widgets.get_mut(&id).unwrap();

                if child.is_ignore() {

                } else if child.is_proxy() {
                    let child_count = child.count();

                    if index < passed + child_count {
                        break;
                    }

                    passed += child_count;
                } else {
                    if passed == index {
                        break;
                    }

                    passed += 1;
                }

                current_index = self.model.next_index(current_index);
            }

            if current_index < end_index {
                let id = self.model.map(current_index.clone(), self.ident);

                let child = self.widgets.get_mut(&id).unwrap();

                if child.is_ignore() {

                } else if child.is_proxy() {
                    return <W as AnySequence<A>>::index(child, index - passed);
                } else {
                    return <W as AnySequence<A>>::index(child, 0);
                }
            }

            panic!("Index out of bounds. Index: {}, Passed: {}", index, passed);
        }
    }

    pub fn foreach_child<A: ?Sized>(&mut self, f: &mut dyn FnMut(&mut A)) where W: AnySequence<A> {
        let mut current_index = self.model.start_index();
        let end_index = self.model.end_index();

        while current_index < end_index {
            let id = self.model.map(current_index.clone(), self.ident);

            self.ensure_exist(current_index.clone());

            let widget = self.widgets.get_mut(&id).unwrap();

            <W as AnySequence<A>>::foreach(widget, f);

            current_index = self.model.next_index(current_index);
        }
    }

    pub fn foreach_child_rev<A: ?Sized>(&mut self, f: &mut dyn FnMut(&mut A)) where W: AnySequence<A> {
        // If the end and start indices are equal, there are no elements in the collection
        if self.model.len() == 0 {
            return;
        }

        let mut current_index = self.model.end_index();
        let start_index = self.model.start_index();

        current_index = self.model.prev_index(current_index);

        loop {
            let id = self.model.map(current_index.clone(), self.ident);

            self.ensure_exist(current_index.clone());

            let widget = self.widgets.get_mut(&id).unwrap();

            <W as AnySequence<A>>::foreach_rev(widget, f);

            if current_index == start_index {
                break;
            }

            current_index = self.model.next_index(current_index);
        }
    }
}

impl<T: StateContract, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>, Id: Hash + Eq + Clone + Debug + 'static> CommonWidget for ForEach<T, M, U, W, Id> {
    fn flag(&self) -> WidgetFlag {
        WidgetFlag::PROXY
    }

    fn child(&mut self, index: usize) -> &mut dyn AnyWidget {
        self.child::<dyn AnyWidget>(index)
    }

    fn child_count(&mut self) -> usize {
        // We can special case when the widget is of kind simple, since we will know the count
        // of children produced, will be equal to the model.
        if W::Kind::kind() == Kind::Simple {
            self.model.len()
        } else {
            let mut current_index = self.model.start_index();
            let end_index = self.model.end_index();

            let mut count = 0;

            while current_index < end_index {
                let id = self.model.map(current_index.clone(), self.ident);

                self.ensure_exist(current_index.clone());

                let child = self.widgets.get_mut(&id).unwrap();

                if child.is_ignore() {

                } else if child.is_proxy() {
                    count += child.child_count();
                } else {
                    count += 1;
                }

                current_index = self.model.next_index(current_index);
            }

            count
        }
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        self.foreach_child::<dyn AnyWidget>(f)
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        self.foreach_child_rev::<dyn AnyWidget>(f)
    }

    fn position(&self) -> Position {
        unimplemented!()
    }

    fn set_position(&mut self, position: Position) {
        unimplemented!()
    }

    fn dimension(&self) -> Dimension {
        unimplemented!()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        unimplemented!()
    }
}

impl<T, M, U, W, Id> WidgetProperties for ForEach<T, M, U, W, Id>
where
    T: StateContract,
    M: RandomAccessCollection<T>,
    W: Widget,
    U: Delegate<M, T, W>,
    Id: Hash + Eq + Clone + Debug + 'static
{
    type Kind = WidgetKindProxy;
}

impl<T: StateContract, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>, Id: Hash + Eq + Clone + Debug + 'static> Debug for ForEach<T, M, U, W, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEach")
            .field("model", &self.model)
            .field("children", &self.widgets)
            .finish()
    }
}

impl<T: StateContract, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>, Id: Hash + Eq + Clone + Debug + 'static> Clone for ForEach<T, M, U, W, Id> {
    fn clone(&self) -> Self {
        ForEach {
            id: WidgetId::new(),
            model: self.model.clone(),
            delegate: self.delegate.clone(),
            widgets: HashMap::new(),
            indices: HashMap::new(),
            phantom: Default::default(),
            ident: self.ident,
        }
    }
}