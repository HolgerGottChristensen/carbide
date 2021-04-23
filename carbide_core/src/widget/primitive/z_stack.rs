use crate::prelude::*;
use std::ops::Deref;
use crate::widget::ChildRender;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct ZStack<GS> where GS: GlobalState {
    id: Uuid,
    children: Vec<Box<dyn Widget<GS>>>,
    position: Point,
    dimension: Dimensions,
    alignment: BasicLayouter,
}

impl<GS: GlobalState> WidgetExt<GS> for ZStack<GS> {}

impl<S: GlobalState> ZStack<S> {
    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<ZStack<S>> {
        Box::new(ZStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            alignment: BasicLayouter::Center
        })
    }

    pub fn alignment(mut self, alignment: BasicLayouter) -> Box<Self> {
        self.alignment = alignment;
        Box::new(self)
    }
}

impl<S: GlobalState> Layout<S> for ZStack<S> {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {

        let mut children_flexibilty: Vec<(u32, &mut dyn Widget<S>)> = self.get_children_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_width = 0.0;
        let mut max_height = 0.0;

        for (_, child) in children_flexibilty {
            let chosen_size = child.calculate_size(requested_size, env);

            if chosen_size[0] > max_width {
                max_width = chosen_size[0];
            }

            if chosen_size[1] > max_height {
                max_height = chosen_size[1];
            }

        }

        self.dimension = [max_width, max_height];
        self.dimension

    }

    fn position_children(&mut self) {
        let positioning = self.alignment.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in self.get_children_mut() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl<S: GlobalState> CommonWidget<S> for ZStack<S> {
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
        self.children
            .iter()
            .map(|x| x.deref())
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.get_flag() == Flags::PROXY {
                    WidgetIter::Multi(Box::new(x.get_children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        self.children
            .iter_mut()
            .map(|x| x.deref_mut())
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.get_flag() == Flags::PROXY {
                    WidgetIterMut::Multi(Box::new(x.get_children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        self.children.iter_mut()
            .map(|x| x.deref_mut())
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
        self.children.iter_mut()
            .map(|x| x.deref_mut())
            .fold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
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

impl<S: GlobalState> ChildRender for ZStack<S> {}


