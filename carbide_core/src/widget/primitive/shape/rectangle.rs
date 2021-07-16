use lyon::algorithms::math::rect;
use lyon::algorithms::path::builder::PathBuilder;
use lyon::algorithms::path::Winding;

use crate::color::Rgba;
use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;
use crate::widget::primitive::shape::{Shape, tessellate};
use crate::widget::types::shape_style::ShapeStyle;
use crate::widget::types::stroke_style::StrokeStyle;
use crate::widget::types::triangle_store::TriangleStore;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Default, Widget)]
pub struct Rectangle<GS> where GS: GlobalState {
    id: Uuid,
    children: Vec<Box<dyn Widget<GS>>>,
    position: Point,
    dimension: Dimensions,
    #[state] fill_color: ColorState<GS>,
    #[state] stroke_color: ColorState<GS>,
    shrink_to_fit: bool,
    style: ShapeStyle,
    stroke_style: StrokeStyle,
    // Store the triangles for the border
    triangle_store: TriangleStore,
}

impl<GS: GlobalState> Rectangle<GS> {
    pub fn fill<C: Into<ColorState<GS>>>(mut self, color: C) -> Box<Self> {
        self.fill_color = color.into();
        self.style += ShapeStyle::Fill;
        Box::new(self)
    }

    pub fn stroke<C: Into<ColorState<GS>>>(mut self, color: C) -> Box<Self> {
        self.stroke_color = color.into();
        self.style += ShapeStyle::Stroke;
        Box::new(self)
    }

    pub fn stroke_style(mut self, line_width: f64) -> Box<Self> {
        self.stroke_style = StrokeStyle::Solid { line_width };
        self.style += ShapeStyle::Stroke;
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
    pub fn debug_outline(_rect: OldRect, _width: Scalar) -> Vec<Primitive> {
        vec![]
    }

    pub fn debug_outline_special(rect: OldRect, width: Scalar) -> Vec<Primitive> {
        let (l, r, b, t) = rect.l_r_b_t();

        let left_border = OldRect::new([l, b], [width, rect.h()]);
        let right_border = OldRect::new([r - width, b], [width, rect.h()]);
        let top_border = OldRect::new([l + width, b], [rect.w() - width * 2.0, width]);
        let bottom_border = OldRect::new([l + width, t - width], [rect.w() - width * 2.0, width]);

        let border_color = Color::Rgba(0.0 / 255.0, 255.0 / 255.0, 251.0 / 255.0, 1.0);//Color::random();
        vec![
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
        ]
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
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
            fill_color: EnvironmentColor::Blue.into(),
            stroke_color: EnvironmentColor::Blue.into(),
            shrink_to_fit: false,
            style: ShapeStyle::Default,
            stroke_style: StrokeStyle::Solid { line_width: 2.0 },
            triangle_store: TriangleStore::new(),
        })
    }
}


impl<GS: GlobalState> Layout<GS> for Rectangle<GS> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
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

impl<GS: GlobalState> Shape<GS> for Rectangle<GS> {
    fn get_triangle_store_mut(&mut self) -> &mut TriangleStore {
        &mut self.triangle_store
    }

    fn get_stroke_style(&self) -> StrokeStyle {
        self.stroke_style.clone()
    }

    fn get_shape_style(&self) -> ShapeStyle {
        ShapeStyle::Stroke
    }
}

impl<GS: GlobalState> Render<GS> for Rectangle<GS> {
    fn get_primitives(&mut self, env: &mut Environment<GS>, global_state: &GS) -> Vec<Primitive> {
        let mut prims = vec![];

        match self.style {
            ShapeStyle::Default => {
                prims.push(Primitive {
                    kind: PrimitiveKind::Rectangle { color: self.fill_color.get_latest_value().clone() },
                    rect: OldRect::new(self.position, self.dimension),
                });
            }
            ShapeStyle::Fill => {
                prims.push(Primitive {
                    kind: PrimitiveKind::Rectangle { color: self.fill_color.get_latest_value().clone() },
                    rect: OldRect::new(self.position, self.dimension),
                });
            }
            ShapeStyle::Stroke => {
                let rect = rect(
                    self.get_x() as f32,
                    self.get_y() as f32,
                    self.get_width() as f32,
                    self.get_height() as f32,
                );
                tessellate(self, &rect, &|builder, rectangle| {
                    builder.add_rectangle(
                        rectangle,
                        Winding::Positive,
                    )
                });

                let stroke_triangles = self.triangle_store.stroke_triangles.clone();

                prims.push(Primitive {
                    kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(*self.stroke_color.get_latest_value()), triangles: stroke_triangles },
                    rect: OldRect::new(self.position, self.dimension),
                });
            }
            ShapeStyle::FillAndStroke => {
                prims.push(Primitive {
                    kind: PrimitiveKind::Rectangle { color: self.fill_color.get_latest_value().clone() },
                    rect: OldRect::new(self.position, self.dimension),
                });

                let rect = rect(
                    self.get_x() as f32,
                    self.get_y() as f32,
                    self.get_width() as f32,
                    self.get_height() as f32,
                );
                tessellate(self, &rect, &|builder, rectangle| {
                    builder.add_rectangle(
                        rectangle,
                        Winding::Positive,
                    )
                });

                let stroke_triangles = self.triangle_store.stroke_triangles.clone();

                prims.push(Primitive {
                    kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(*self.stroke_color.get_latest_value()), triangles: stroke_triangles },
                    rect: OldRect::new(self.position, self.dimension),
                });
            }
        }

        prims.extend(Rectangle::<GS>::debug_outline(OldRect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(env, global_state)).collect();
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
