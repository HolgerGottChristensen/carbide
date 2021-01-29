use crate::prelude::*;
use crate::widget::primitive::edge_insets::EdgeInsets;

pub static SCALE: f64 = -1.0;


#[derive(Debug, Clone, Widget)]
pub struct Padding<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    edge_insets: EdgeInsets
}

impl<GS: GlobalState> WidgetExt<GS> for Padding<GS> {}

impl<S: GlobalState> Padding<S> {
    pub fn init(edge_insets: EdgeInsets, child: Box<dyn Widget<S>>) -> Box<Self> {
        Box::new(Padding{
            id: Default::default(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            edge_insets
        })
    }
}

impl<S: GlobalState> CommonWidget<S> for Padding<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::single(&mut self.child)
    }


    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        [self.dimension[0].abs(), self.dimension[1].abs()]
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<S: GlobalState> Layout<S> for Padding<S> {
    fn flexibility(&self) -> u32 {
        9
    }

    fn calculate_size(&mut self, dimension: Dimensions, env: &Environment<S>) -> Dimensions {
        let dimensions = [dimension[0] - self.edge_insets.left - self.edge_insets.right, dimension[1] - self.edge_insets.top - self.edge_insets.bottom];

        let child_dimensions = self.child.calculate_size(dimensions, env);

        self.dimension = [child_dimensions[0] + self.edge_insets.left + self.edge_insets.right, child_dimensions[1] + self.edge_insets.top + self.edge_insets.bottom];

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = [self.position[0] + self.edge_insets.left, self.position[1] + self.edge_insets.top];
        let dimension = [self.dimension[0] - self.edge_insets.left - self.edge_insets.right, self.dimension[1] - self.edge_insets.top - self.edge_insets.bottom];

        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl<S: GlobalState> Render<S> for Padding<S> {

    fn get_primitives(&mut self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, [self.dimension[0], self.dimension[1]]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(fonts);
        prims.extend(children);

        return prims;
    }
}