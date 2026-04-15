use std::marker::PhantomData;
use std::rc::Rc;
use carbide::environment::EnvironmentKey;
use carbide::random_access_collection::RandomAccessCollection;
use carbide::state::{AnyReadState, StateContract};
use carbide::widget::{AnyWidget, Delegate, Styled, Widget, WidgetExt, WidgetProperties, WidgetStyle};
use carbide::widget::properties::WidgetKindSimple;
use crate::list::{ListStyle, ListStyleKey};

#[derive(Clone)]
pub(crate) struct RowStyled<D, T, M, W>
where
    D: Delegate<M, T, W>,
    T: StateContract,
    M: RandomAccessCollection<T>,
    W: Widget,
{
    inner: D,
    f: fn(&Box<dyn ListStyle>, W) -> Box<dyn AnyWidget>,
    phantom_data1: PhantomData<T>,
    phantom_data2: PhantomData<M>,
    phantom_data3: PhantomData<W>,
}

impl<
    D: Delegate<M, T, W>,
    T: StateContract,
    M: RandomAccessCollection<T>,
    W: Widget
> RowStyled<D, T, M, W> {
    pub fn new(inner: D, f: fn(&Box<dyn ListStyle>, W) -> Box<dyn AnyWidget>) -> RowStyled<D, T, M, W> {
        RowStyled {
            inner,
            f,
            phantom_data1: Default::default(),
            phantom_data2: Default::default(),
            phantom_data3: Default::default(),
        }
    }
}

impl<
    D: Delegate<M, T, W>,
    T: StateContract,
    M: RandomAccessCollection<T>,
    W: Widget + WidgetProperties<Kind=WidgetKindSimple>
> Delegate<M, T, Box<dyn AnyWidget>> for RowStyled<D, T, M, W> {
    fn call<'a>(&'a self, item: M::Item<'a>, index: Box<dyn AnyReadState<T=M::Idx>>) -> Box<dyn AnyWidget> {
        let inner = self.inner.call(item, index);

        Styled::<ListStyleKey, W>::new(inner, self.f).boxed()
    }
}