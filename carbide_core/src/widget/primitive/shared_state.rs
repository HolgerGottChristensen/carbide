use std::fmt::Debug;

use crate::prelude::*;

/// This widget is for containing shared state. This is very rarely needed as there always is a
/// source of truth. The state is always kept in a parent. If this is not the case we need this widget.
/// An example of its use is when we have a foreach in an overlay layer. Therefore the state is
/// further down the tree than the items in the foreach. The first item in the foreach if there is no
/// parent state, the first item in the foreach widget will override all the others state.
#[derive(Debug, Clone, Widget)]
pub struct SharedState<T, GS> where T: StateContract, GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] shared_state: TState<T, GS>,
}

impl<T: StateContract, GS: GlobalState> SharedState<T, GS> {
    pub fn new<S: Into<TState<T, GS>>>(shared_state: S, child: Box<dyn Widget<GS>>) -> Box<Self> {
        Box::new(SharedState {
            id: Uuid::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            shared_state: shared_state.into(),
        })
    }
}

impl<T: StateContract, GS: GlobalState> Layout<GS> for SharedState<T, GS> {
    fn flexibility(&self) -> u32 {
        self.child.flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        self.dimension = self.child.calculate_size(requested_size, env);
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);

        self.child.position_children();
    }
}

impl<T: StateContract, GS: GlobalState> CommonWidget<GS> for SharedState<T, GS> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }
    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<T: StateContract, GS: GlobalState> Render<GS> for SharedState<T, GS> {
    fn get_primitives(&mut self, env: &mut Environment<GS>, global_state: &GS) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<GS>::debug_outline(OldRect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(env, global_state)).collect();
        prims.extend(children);
        return prims;
    }
}


impl<T: 'static + StateContract, GS: GlobalState> WidgetExt<GS> for SharedState<T, GS> {}