use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::draw::{Dimension, Position};
use carbide::identifiable::Identifiable;
use carbide::random_access_collection::RandomAccessCollection;
use carbide::state::{AnyReadState, LocalState, ReadStateExtNew, StateContract};
use carbide::widget::{AnyWidget, CommonWidget, Delegate, ForEach, MouseArea, ReverseAnySequence, Widget, WidgetExt, WidgetId, WidgetProperties, WidgetSync};
use carbide::widget::properties::WidgetKindSimple;
use crate::list::{ListSelection, SelectableMouseAreaAction};

pub trait AnyListRow: Identifiable<Id=WidgetId> + WidgetSync + DynClone + 'static {
    fn selected(&self) -> Box<dyn AnyReadState<T=bool>>;
    fn hovered(&self) -> Box<dyn AnyReadState<T=bool>>;
    fn pressed(&self) -> Box<dyn AnyReadState<T=bool>>;

    fn clickable(&self, child: Box<dyn AnyWidget>) -> Box<dyn AnyWidget>;

    fn widget(&self) -> Box<dyn AnyWidget>;
}

clone_trait_object!(AnyListRow);


#[derive(Debug, Clone, Widget)]
pub(crate) struct ListRow<Id, T, M, W> where Id: Hash + Eq + Clone + Debug + 'static, T: StateContract, M: RandomAccessCollection<T>, W: Widget + WidgetProperties<Kind=WidgetKindSimple> {
    pub selection: ListSelection<Id>,
    pub ident: fn(&T) -> Id,
    pub model: Rc<M>,
    pub index: Box<dyn AnyReadState<T=M::Idx>>,
    pub last_clicked: LocalState<Option<M::Idx>>,

    pub hovered: LocalState<bool>,
    pub pressed: LocalState<bool>,
    pub child: W
}

impl<Id: Hash + Eq + Clone + Debug + 'static, T: StateContract, M: RandomAccessCollection<T>, W: Widget + WidgetProperties<Kind=WidgetKindSimple>> AnyListRow for ListRow<Id, T, M, W> {
    fn selected(&self) -> Box<dyn AnyReadState<T=bool>> {
        todo!()
    }

    fn hovered(&self) -> Box<dyn AnyReadState<T=bool>> {
        self.hovered.as_dyn_read()
    }

    fn pressed(&self) -> Box<dyn AnyReadState<T=bool>> {
        self.pressed.as_dyn_read()
    }

    fn clickable(&self, child: Box<dyn AnyWidget>) -> Box<dyn AnyWidget> {
        let action = SelectableMouseAreaAction {
            selection: self.selection.clone(),
            ident: self.ident,
            model: self.model.clone(),
            index: self.index.clone(),
            last_clicked: self.last_clicked.clone(),
        };

        MouseArea::new(child)
            .custom_on_click(action)
            .hovered(self.hovered.clone())
            .pressed(self.pressed.clone())
            .boxed()
    }

    fn widget(&self) -> Box<dyn AnyWidget> {
        self.child.clone().boxed()
    }
}

impl<Id: Hash + Eq + Clone + Debug + 'static, T: StateContract, M: RandomAccessCollection<T>, W: Widget + WidgetProperties<Kind=WidgetKindSimple>> Identifiable for ListRow<Id, T, M, W> {
    type Id = WidgetId;

    fn id(&self) -> Self::Id {
        self.child.id()
    }
}

impl<Id: Hash + Eq + Clone + Debug + 'static, T: StateContract, M: RandomAccessCollection<T>, W: Widget + WidgetProperties<Kind=WidgetKindSimple>> CommonWidget for ListRow<Id, T, M, W> {
    fn child(&mut self, index: usize) -> &mut dyn AnyWidget {
        todo!()
    }

    fn child_count(&mut self) -> usize {
        todo!()
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        todo!()
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        todo!()
    }

    fn position(&self) -> Position {
        todo!()
    }

    fn set_position(&mut self, position: Position) {
        todo!()
    }

    fn dimension(&self) -> Dimension {
        todo!()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        todo!()
    }
}

impl<T: StateContract,
    M: RandomAccessCollection<T>,
    W: AnyListRow + Widget,
    U: Delegate<M, T, W>,
    Id: Hash + Eq + Clone + Debug + 'static> ReverseAnySequence<ForEach<T, M, U, W, Id>> for dyn AnyListRow {
    fn index(value: &mut ForEach<T, M, U, W, Id>, index: usize) -> &mut dyn AnyListRow {
        /*let (   W1   ) = value;

        let mut passed = 0;

        if W1.is_ignore() {} else if W1.is_proxy() {
            let child_count = W1.child_count();
            if index < passed + child_count {
                return AnySelectableWidget::child(W1, index - passed);
            }

            passed += child_count;
        } else {
            if index == passed {
                return W1;
            }

            passed += 1;
        }

        panic!("Index out of bounds. Index: {}, Passed: {}", index, passed);*/
        todo!()

    }

    fn count(value: &mut ForEach<T, M, U, W, Id>) -> usize {
        value.child_count()
    }

    fn foreach(value: &mut ForEach<T, M, U, W, Id>, f: &mut dyn FnMut(&mut dyn AnyListRow)) {
        /*let (   W1   ) = value;
        if W1.is_ignore() {} else if W1.is_proxy() {
            AnySelectableWidget::foreach_child(W1, f);
        } else {
            f(W1);
        }*/
    }

    fn foreach_rev(value: &mut ForEach<T, M, U, W, Id>, f: &mut dyn FnMut(&mut dyn AnyListRow)) {
        /*let (   W1    ) = value;
        if W1.is_ignore() {} else if W1.is_proxy() {
            AnySelectableWidget::foreach_child_rev(W1, f);
        } else {
            f(W1);
        }*/
    }
}