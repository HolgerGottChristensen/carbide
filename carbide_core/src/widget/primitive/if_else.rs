use crate::prelude::*;
use crate::widget::Frame;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct IfElse<GS> where GS: GlobalState {
    id: Uuid,
    when_true: Box<dyn Widget<GS>>,
    when_false: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] predicate: BoolState<GS>,
}

impl<GS: GlobalState> IfElse<GS> {
    pub fn new<B: Into<BoolState<GS>>>(predicate: B) -> Box<Self> {
        Box::new(IfElse {
            id: Uuid::new_v4(),
            predicate: predicate.into(),
            when_true: Frame::init(0.0.into(), 0.0.into(), Rectangle::initialize(vec![])),
            when_false: Frame::init(0.0.into(), 0.0.into(), Rectangle::initialize(vec![])),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
        })
    }

    pub fn when_true(mut self, when_true: Box<dyn Widget<GS>>) -> Box<Self> {
        self.when_true = when_true;
        Box::new(self)
    }

    pub fn when_false(mut self, when_false: Box<dyn Widget<GS>>) -> Box<Self> {
        self.when_false = when_false;
        Box::new(self)
    }
}

impl<GS: GlobalState> Layout<GS> for IfElse<GS> {
    fn flexibility(&self) -> u32 {
        if *self.predicate.get_latest_value() {
            self.when_true.flexibility()
        } else {
            self.when_false.flexibility()
        }
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        if *self.predicate.get_latest_value() {
            self.dimension = self.when_true.calculate_size(requested_size, env);
        } else {
            self.dimension = self.when_false.calculate_size(requested_size, env);
        }
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        if *self.predicate.get_latest_value() {
            positioning(position, dimension, &mut self.when_true);

            self.when_true.position_children();
        } else {
            positioning(position, dimension, &mut self.when_false);

            self.when_false.position_children();
        }
    }
}

impl<S: GlobalState> CommonWidget<S> for IfElse<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        if *self.predicate.get_latest_value() {
            if self.when_true.get_flag() == Flags::PROXY {
                self.when_true.get_children()
            } else {
                WidgetIter::single(self.when_true.deref())
            }
        } else {
            if self.when_false.get_flag() == Flags::PROXY {
                self.when_false.get_children()
            } else {
                WidgetIter::single(self.when_false.deref())
            }
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if *self.predicate.get_latest_value() {
            if self.when_true.get_flag() == Flags::PROXY {
                self.when_true.get_children_mut()
            } else {
                WidgetIterMut::single(self.when_true.deref_mut())
            }
        } else {
            if self.when_false.get_flag() == Flags::PROXY {
                self.when_false.get_children_mut()
            } else {
                WidgetIterMut::single(self.when_false.deref_mut())
            }
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        if *self.predicate.get_latest_value() {
            WidgetIterMut::single(self.when_true.deref_mut())
        } else {
            WidgetIterMut::single(self.when_false.deref_mut())
        }
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
        if *self.predicate.get_latest_value() {
            WidgetIterMut::single(self.when_true.deref_mut())
        } else {
            WidgetIterMut::single(self.when_false.deref_mut())
        }
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

impl<GS: GlobalState> Render<GS> for IfElse<GS> {
    fn get_primitives(&mut self, env: &Environment<GS>, global_state: &GS) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<GS>::debug_outline(OldRect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(env, global_state)).collect();
        prims.extend(children);
        return prims;
    }
}


impl<GS: GlobalState> WidgetExt<GS> for IfElse<GS> {}