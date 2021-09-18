use crate::draw::{Dimension, Position};
use crate::prelude::*;

pub trait Delegate<T: StateContract>: Fn(TState<T>, UsizeState) -> Box<dyn Widget> + Clone {}

impl<I, T: StateContract> Delegate<T> for I where I: Fn(TState<T>, UsizeState) -> Box<dyn Widget> + Clone {}

#[derive(Clone, Widget)]
pub struct ForEach<T, U> where T: StateContract, U: Delegate<T> {
    id: Uuid,
    position: Position,
    dimension: Dimension,

    #[state] model: TState<Vec<T>>,
    delegate: U,

    children: Vec<Box<dyn Widget>>,
    #[state] index_offset: UsizeState,
}

impl<T: StateContract + 'static, U: Delegate<T>> ForEach<T, U> {
    pub fn new<K: Into<TState<Vec<T>>>>(model: K, delegate: U) -> Box<Self> {
        let model = model.into();
        let mut map: Vec<Box<dyn Widget>> = vec![];

        for (index, _element) in model.value().deref().iter().enumerate() {
            let index_state: UsizeState = ValueState::new(index).into();
            let item_state: MapState<Vec<T>, T, usize> = MapState::new(model.clone(),
                                                                       index,
                                                                       |a, index| {
                                                                           &a[index]
                                                                       },
                                                                       |a, index| {
                                                                           &mut a[index]
                                                                       });
            let widget = (delegate)(item_state.into(), index_state);
            map.push(Box::new(widget));
        }

        Box::new(Self {
            id: Uuid::new_v4(),
            position: Position::default(),
            dimension: Dimension::default(),
            model,
            delegate,
            children: map,
            index_offset: ValueState::new(0).into(),
        })
    }

    /*pub fn id_state(mut self, state: Box<dyn State<T, GS>>) -> Box<Self> {
        self.id_state = state;
        Box::new(self)
    }

    pub fn index_state(mut self, state: Box<dyn State<usize, GS>>) -> Box<Self> {
        self.index_state = state;
        Box::new(self)
    }

    pub fn index_offset(mut self, state: Box<dyn State<usize, GS>>) -> Box<Self> {
        self.index_offset = state;
        Box::new(self)
    }*/
}

impl<T: StateContract, U: Delegate<T>> CommonWidget for ForEach<T, U> {
    fn id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::PROXY
    }

    fn children(&self) -> WidgetIter {
        let contains_proxy = self.children.iter().fold(false, |a, b| a || b.flag() == Flags::PROXY);
        if !contains_proxy {
            WidgetIter::Vec(self.children.iter())
        } else {
            self.children
                .iter()
                .rfold(WidgetIter::Empty, |acc, x| {
                    if x.flag() == Flags::PROXY {
                        WidgetIter::Multi(Box::new(x.children()), Box::new(acc))
                    } else {
                        WidgetIter::Single(x, Box::new(acc))
                    }
                })
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        let contains_proxy = self.children.iter().fold(false, |a, b| a || b.flag() == Flags::PROXY);
        if !contains_proxy {
            WidgetIterMut::Vec(self.children.iter_mut())
        } else {
            self.children
                .iter_mut()
                .rfold(WidgetIterMut::Empty, |acc, x| {
                    if x.flag() == Flags::PROXY {
                        WidgetIterMut::Multi(Box::new(x.children_mut()), Box::new(acc))
                    } else {
                        WidgetIterMut::Single(x, Box::new(acc))
                    }
                })
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Vec(self.children.iter_mut())
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::VecRev(self.children.iter_mut().rev())
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

    fn set_dimension(&mut self, dimensions: Dimension) {
        self.dimension = dimensions
    }
}

impl<T: StateContract + 'static, U: Delegate<T> + 'static> WidgetExt for ForEach<T, U> {}
