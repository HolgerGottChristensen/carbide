use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Border<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] color: ColorState<GS>,
    border_width: u32,
}

impl<GS: GlobalState> WidgetExt<GS> for Border<GS> {}

impl<GS: GlobalState> Layout<GS> for Border<GS> {
    fn flexibility(&self) -> u32 {
        self.child.flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        let border_width = self.border_width as f64;
        let dimensions = [requested_size[0] - border_width - border_width, requested_size[1] - border_width - border_width];

        let child_dimensions = self.child.calculate_size(dimensions, env);

        self.dimension = [child_dimensions[0] + border_width + border_width, child_dimensions[1] + border_width + border_width];

        self.dimension
    }

    fn position_children(&mut self) {
        let border_width = self.border_width as f64;
        let positioning = BasicLayouter::Center.position();
        let position = [self.position[0] + border_width, self.position[1] + border_width];
        let dimension = [self.dimension[0] - border_width - border_width, self.dimension[1] - border_width - border_width];

        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl<S: GlobalState> CommonWidget<S> for Border<S> {
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

impl<GS: GlobalState> Render<GS> for Border<GS> {
    fn get_primitives(&mut self, env: &Environment<GS>, global_state: &GS) -> Vec<Primitive> {
        let rect = OldRect::new(self.position, self.dimension);
        let (l, r, b, t) = rect.l_r_b_t();

        let width = self.border_width as f64;

        let left_border = OldRect::new([l, b], [width, rect.h()]);
        let right_border = OldRect::new([r - width, b], [width, rect.h()]);
        let top_border = OldRect::new([l + width, b], [rect.w() - width * 2.0, width]);
        let bottom_border = OldRect::new([l + width, t - width], [rect.w() - width * 2.0, width]);

        let border_color = self.color.get_latest_value();
        let mut prims = vec![
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone() },
                rect: left_border,
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone() },
                rect: right_border,
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone() },
                rect: top_border,
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone() },
                rect: bottom_border,
            },
        ];

        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(env, global_state)).collect();
        prims.extend(children);

        return prims;
    }
}

impl<GS: GlobalState> Border<GS> {
    pub fn color<C: Into<ColorState<GS>>>(mut self, color: C) -> Box<Self> {
        self.color = color.into();
        Box::new(self)
    }

    pub fn border_width(mut self, width: u32) -> Box<Self> {
        self.border_width = width;
        Box::new(self)
    }

    pub fn initialize(child: Box<dyn Widget<GS>>) -> Box<Self> {
        Box::new(Border {
            id: Uuid::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
            color: Color::random().into(),
            border_width: 2,
        })
    }
}
