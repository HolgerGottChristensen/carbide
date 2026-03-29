use std::cmp::PartialEq;
use crate::common::flags::WidgetFlag;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::identifiable::Identifiable;
use crate::lifecycle::InitializationContext;
use crate::state::{AnyReadState, LocalState, State, StateContract};
use crate::widget::foreach_widget::Delegate as ForEachChildDelegate;
use crate::widget::foreach_widget::ForEachWidget;
use crate::widget::{AnyWidget, CommonWidget, Empty, Sequence as ForEachSequence, Widget, WidgetExt, WidgetId, WidgetProperties, WidgetSync};
use dyn_clone::DynClone;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use carbide::widget::properties::Kind;
use crate::random_access_collection::RandomAccessCollection;
use crate::widget::properties::{WidgetKind, WidgetKindProxy};

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
pub struct ForEach<T, M, U, W>
where
    T: StateContract + Identifiable,
    M: RandomAccessCollection<T>,
    W: Widget,
    U: Delegate<M, T, W>,
{
    #[id] id: WidgetId,

    model: M,
    delegate: U,

    widgets: HashMap<T::Id, W>,
    indices: HashMap<T::Id, LocalState<M::Idx>>,

    phantom: PhantomData<T>,
}

impl ForEach<(), Vec<()>, EmptyDelegate, Empty> {
    pub fn new<T: StateContract + Identifiable, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>>(model: M, delegate: U) -> ForEach<T, M, U, W> {
        ForEach {
            id: WidgetId::new(),
            model,
            delegate,
            widgets: HashMap::new(),
            indices: HashMap::new(),
            phantom: PhantomData::default()
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

impl<T, M, U, W> ForEach<T, M, U, W>
where
    T: StateContract + Identifiable,
    M: RandomAccessCollection<T>,
    W: Widget,
    U: Delegate<M, T, W>,
{
    fn ensure_exist(&mut self, index: M::Idx) {
        let id = self.model.id(index.clone());

        if let Some(i) = self.indices.get_mut(&id) {
            *i.value_mut() = index.clone();
        } else {
            self.indices.insert(id.clone(), LocalState::new(index.clone()));
        }

        if self.widgets.get(&id).is_none() {
            let item = self.model.index(index);
            let i = self.indices.get(&id).unwrap();

            let new = self.delegate.call(item, Box::new(i.clone()));

            self.widgets.insert(id, new);
        }
    }
}

impl<T: StateContract + Identifiable, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>> CommonWidget for ForEach<T, M, U, W> {

    fn flag(&self) -> WidgetFlag {
        WidgetFlag::PROXY
    }

    fn child(&self, index: usize) -> &dyn AnyWidget {
        /*if W::Kind::kind() == Kind::Simple {
            let idx = self.model.index_from_offset(index);
            self.ensure_exist(idx.clone());
            let id = self.model.id(idx);

            self.widgets.get(&id).unwrap()
        } else {
            let mut current_index = self.model.start_index();
            let end_index = self.model.end_index();
            let mut passed = 0;

            while current_index < end_index {
                let id = self.model.id(current_index.clone());

                self.ensure_exist(current_index.clone());

                let child = self.widgets.get(&id).unwrap();

                if child.is_ignore() {

                } else if child.is_proxy() {
                    let child_count = child.child_count();

                    if index < passed + child_count {
                        return child.child(index - passed);
                    }

                    passed += child_count;
                } else {
                    if passed == index {
                        return child;
                    }

                    passed += 1;
                }

                current_index = self.model.next_index(current_index);
            }

            panic!("Index out of bounds. Index: {}, Passed: {}, Count: {}", index, passed, self.child_count());
        }*/

        todo!()
    }

    fn child_mut(&mut self, index: usize) -> &mut dyn AnyWidget {
        if W::Kind::kind() == Kind::Simple {
            let idx = self.model.index_from_offset(index);
            self.ensure_exist(idx.clone());
            let id = self.model.id(idx);

            self.widgets.get_mut(&id).unwrap()
        } else {
            let mut current_index = self.model.start_index();
            let end_index = self.model.end_index();

            let mut passed = 0;

            while current_index < end_index {
                let id = self.model.id(current_index.clone());

                self.ensure_exist(current_index.clone());

                let child = self.widgets.get(&id).unwrap();

                if child.is_ignore() {

                } else if child.is_proxy() {
                    let child_count = child.child_count();

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
                let id = self.model.id(current_index.clone());

                let child = self.widgets.get_mut(&id).unwrap();

                if child.is_ignore() {

                } else if child.is_proxy() {
                    return child.child_mut(index - passed);
                } else {
                    return child;
                }
            }

            panic!("Index out of bounds. Index: {}, Passed: {}", index, passed);
        }
    }

    fn child_count(&self) -> usize {
        // We can special case when the widget is of kind simple, since we will know the count
        // of children produced, will be equal to the model.
        if W::Kind::kind() == Kind::Simple {
            self.model.len()
        } else {
            /*let mut current_index = self.model.start_index();
            let end_index = self.model.end_index();

            let mut count = 0;

            while current_index < end_index {
                let id = self.model.id(current_index.clone());

                self.ensure_exist(current_index.clone());

                let child = self.widgets.get(&id).unwrap();

                if child.is_ignore() {

                } else if child.is_proxy() {
                    count += child.child_count();
                } else {
                    count += 1;
                }

                current_index = self.model.next_index(current_index);
            }

            count*/
            todo!()
        }
    }

    fn foreach_child(&self, f: &mut dyn FnMut(&dyn AnyWidget)) {
        /*let mut current_index = self.model.start_index();
        let end_index = self.model.end_index();

        while current_index < end_index {
            let id = self.model.id(current_index.clone());

            self.ensure_exist(current_index.clone());

            let widget = self.widgets.get(&id).unwrap();

            if widget.is_ignore() {

            } else if widget.is_proxy() {
                widget.foreach_child(f);
            } else {
                f(widget);
            }

            current_index = self.model.next_index(current_index);
        }*/
    }

    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        let mut current_index = self.model.start_index();
        let end_index = self.model.end_index();

        while current_index < end_index {
            let id = self.model.id(current_index.clone());

            self.ensure_exist(current_index.clone());

            let widget = self.widgets.get_mut(&id).unwrap();

            if widget.is_ignore() {

            } else if widget.is_proxy() {
                widget.foreach_child_mut(f);
            } else {
                f(widget);
            }

            current_index = self.model.next_index(current_index);
        }
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        // If the end and start indices are equal, there are no elements in the collection
        if self.model.len() == 0 {
            return;
        }

        let mut current_index = self.model.end_index();
        let start_index = self.model.start_index();

        current_index = self.model.prev_index(current_index);

        loop {
            let id = self.model.id(current_index.clone());

            self.ensure_exist(current_index.clone());

            let widget = self.widgets.get_mut(&id).unwrap();

            if widget.is_ignore() {

            } else if widget.is_proxy() {
                widget.foreach_child_rev(f);
            } else {
                f(widget);
            }

            if current_index == start_index {
                break;
            }

            current_index = self.model.next_index(current_index);
        }
    }

    fn foreach_child_direct(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        /*let mut current_index = self.model.start_index();
        let end_index = self.model.end_index();

        while current_index < end_index {
            let id = self.model.id(current_index.clone());

            self.ensure_exist(current_index.clone());

            f(self.widgets.get_mut(&id).unwrap());

            current_index = self.model.next_index(current_index);
        }*/
    }

    fn foreach_child_direct_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        /*// If the end and start indices are equal, there are no elements in the collection
        if self.model.len() == 0 {
            return;
        }

        let mut current_index = self.model.end_index();
        let start_index = self.model.start_index();

        current_index = self.model.prev_index(current_index);

        loop {
            let id = self.model.id(current_index.clone());

            self.ensure_exist(current_index.clone());

            f(self.widgets.get_mut(&id).unwrap());

            if current_index == start_index {
                break;
            }

            current_index = self.model.next_index(current_index);
        }*/
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

impl<T, M, U, W> WidgetProperties for ForEach<T, M, U, W>
where
    T: StateContract + Identifiable,
    M: RandomAccessCollection<T>,
    W: Widget,
    U: Delegate<M, T, W>,
{
    type Kind = WidgetKindProxy;
}

impl<T: StateContract + Identifiable, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>> Debug for ForEach<T, M, U, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForEach32")
            .field("model", &self.model)
            .field("children", &self.widgets)
            .finish()
    }
}

impl<T: StateContract + Identifiable, M: RandomAccessCollection<T>, W: Widget, U: Delegate<M, T, W>> Clone for ForEach<T, M, U, W> {
    fn clone(&self) -> Self {
        ForEach {
            id: WidgetId::new(),
            model: self.model.clone(),
            delegate: self.delegate.clone(),
            widgets: HashMap::new(),
            indices: HashMap::new(),
            phantom: Default::default(),
        }
    }
}