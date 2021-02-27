use crate::prelude::*;



#[derive(Debug, Clone, Widget)]
#[state_sync(sync_state)]
pub struct OverlaidLayer<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    overlay: Option<Box<dyn Widget<GS>>>,
    overlay_id: String,
    position: Point,
    dimension: Dimensions,
}

impl<GS: GlobalState> WidgetExt<GS> for OverlaidLayer<GS> {}

impl<S: GlobalState> Layout<S> for OverlaidLayer<S> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {
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

impl<S: GlobalState> CommonWidget<S> for OverlaidLayer<S> {
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

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
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

impl<S: GlobalState> Render<S> for OverlaidLayer<S> {

    fn get_primitives(&mut self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children_mut()
            .flat_map(|f| f.get_primitives(fonts))
            .collect();
        prims.extend(children);

        if let Some(t) = &mut self.overlay {
            let overlay_prims = t.get_primitives(fonts);
            prims.extend(overlay_prims);
        }

        return prims;
    }
}



impl<S: GlobalState> OverlaidLayer<S> {

    fn sync_state(&mut self, env: &mut Environment<S>, global_state: &S) {
        // This might not be the prettiest place to retrieve things from the env
        self.update_all_widget_state(env, global_state);

        self.insert_local_state(env);

        for child in self.get_proxied_children() {
            child.sync_state(env, global_state)
        }

        // Check if env contains an overlay widget with the specified id
        self.overlay = env.get_overlay(&self.overlay_id);

        if let Some(overlay) = &mut self.overlay {
            overlay.sync_state(env, global_state);
        }


        self.update_local_widget_state(env);
    }

    pub fn new(overlay_id: &str, child: Box<dyn Widget<S>>) -> Box<Self> {
        Box::new(Self {
            id: Uuid::new_v4(),
            child,
            overlay: None,
            overlay_id: overlay_id.to_string(),
            position: [0.0,0.0],
            dimension: [0.0,0.0],
        })
    }
}
