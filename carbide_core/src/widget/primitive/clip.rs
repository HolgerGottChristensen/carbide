use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;

#[derive(Debug, Clone, Widget)]
pub struct Clip<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
}

impl<GS: GlobalState> WidgetExt<GS> for Clip<GS> {}

impl<GS: GlobalState> Layout<GS> for Clip<GS> {
    fn flexibility(&self) -> u32 {
        self.child.flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        self.child.calculate_size(requested_size, env);
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);

        self.child.position_children();
    }
}

impl<S: GlobalState> CommonWidget<S> for Clip<S> {
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
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(self.child.deref())
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(self.child.deref_mut())
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::single(self.child.deref_mut())
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

impl<GS: GlobalState> Render<GS> for Clip<GS> {
    fn get_primitives(&mut self, env: &Environment<GS>, global_state: &GS) -> Vec<Primitive> {
        let mut prims = vec![
            Primitive {
                kind: PrimitiveKind::Clip,
                rect: OldRect::new(self.position, self.dimension),
            }
        ];
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(env, global_state)).collect();
        prims.extend(children);
        prims.extend(Rectangle::<GS>::debug_outline(OldRect::new(self.position, self.dimension), 1.0));

        prims.push(Primitive {
            kind: PrimitiveKind::UnClip,
            rect: OldRect::new(self.position, self.dimension),
        });

        return prims;
    }
}


impl<S: GlobalState> Clip<S> {
    pub fn new(child: Box<dyn Widget<S>>) -> Box<Self <>> {
        Box::new(Clip {
            id: Uuid::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
        })
    }

    /*pub fn body(&mut self) -> Box<dyn Widget<S>> {
        widget_body!(
            HStack (
                alignment: Aligment::Top,
                spacing: 10.0,
            ) {
                for i in $self.model {
                    Text("Item: {}, index: {}", $item, $index),
                }
            }
        )
    }*/
}
