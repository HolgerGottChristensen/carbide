use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;
use crate::state::environment_color::EnvironmentColor;


/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Rectangle<GS> where GS: GlobalState {
    id: Uuid,
    children: Vec<Box<dyn Widget<GS>>>,
    position: Point,
    dimension: Dimensions,
    #[state] color: ColorState<GS>,
    shrink_to_fit: bool,
}

impl<GS: GlobalState> Rectangle<GS> {

    pub fn fill(mut self, color: ColorState<GS>) -> Box<Self> {
        self.color = color;
        Box::new(self)
    }

    pub fn shrink_to_fit(mut self) -> Box<Self> {
        self.shrink_to_fit = true;
        Box::new(self)
    }

    pub fn position(mut self, position: Point) -> Box<Self> {
        self.position = position;
        Box::new(self)
    }

    //#[cfg(not(feature = "debug-outline"))]
    pub fn debug_outline(_rect: Rect, _width: Scalar) -> Vec<Primitive> {
        vec![]
    }

    //#[cfg(feature = "debug-outline")]
    /*pub fn debug_outline(rect: Rect, width: Scalar) -> Vec<Primitive> {
        let (l, r, b, t) = rect.l_r_b_t();

        let left_border = Rect::new([l,b], [width, rect.h()]);
        let right_border = Rect::new([r-width,b], [width, rect.h()]);
        let top_border = Rect::new([l+width,b], [rect.w()-width*2.0, width]);
        let bottom_border = Rect::new([l+width,t-width], [rect.w()-width*2.0, width]);

        let border_color = Color::Rgba(0.0 / 255.0, 255.0 / 255.0, 251.0 / 255.0, 1.0);//Color::random();
        vec![
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: left_border
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: right_border
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: top_border
            },
            Primitive {
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                rect: bottom_border
            },
        ]
    }*/

    pub fn initialize(children: Vec<Box<dyn Widget<GS>>>) -> Box<Rectangle<GS>> {
        Box::new(Rectangle {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            color: EnvironmentColor::Blue.into(),
            shrink_to_fit: false
        })
    }
}



impl<GS: GlobalState> Layout<GS> for Rectangle<GS> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<GS>) -> Dimensions {
        let mut max_child_size = [0.0, 0.0];

        for child in &mut self.children {
            let child_size = child.calculate_size(requested_size, env);

            if child_size[0] > max_child_size[0] {
                max_child_size[0] = child_size[0];
            }

            if child_size[1] > max_child_size[1] {
                max_child_size[1] = child_size[1];
            }
        }

        if self.shrink_to_fit {
            self.dimension = max_child_size;
        } else {
            self.dimension = requested_size;
        }

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in &mut self.children {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl<GS: GlobalState> CommonWidget<GS> for Rectangle<GS> {
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
        self.children
            .iter()
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.get_flag() == Flags::PROXY {
                    WidgetIter::Multi(Box::new(x.get_children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        self.children
            .iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.get_flag() == Flags::PROXY {
                    WidgetIterMut::Multi(Box::new(x.get_children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        self.children.iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
        self.children.iter_mut()
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

impl<S: GlobalState> Render<S> for Rectangle<S> {

    fn get_primitives(&mut self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![
            Primitive {
                kind: PrimitiveKind::Rectangle { color: self.color.get_latest_value().clone()},
                rect: Rect::new(self.position, self.dimension)
            }
        ];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}

/// Whether the rectangle is drawn as an outline or a filled color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    /// Only the outline of the rectangle is drawn.
    Outline,
    /// The rectangle area is filled with some color.
    Fill,
}

impl<GS: GlobalState> WidgetExt<GS> for Rectangle<GS> {}
