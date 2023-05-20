use cgmath::{Deg, Matrix4, SquareMatrix, Vector3};
use carbide_core::render::RenderContext;
use carbide_core::state::RMap1;


use carbide_macro::{carbide_default_builder, carbide_default_builder2};

use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::layout::BasicLayouter;
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::state::{Map1, ReadState, StateSync, TState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Transform<W, M> where W: Widget + Clone, M: ReadState<T=Matrix4<f32>> + Clone {
    id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
    anchor: BasicLayouter,
    #[state]
    matrix: M,
}

impl Transform<Empty, Matrix4<f32>> {
    #[carbide_default_builder2]
    pub fn new<W: Widget + Clone, M: ReadState<T=Matrix4<f32>> + Clone>(child: W, matrix: M) -> Box<Transform<W, M>> {
        Box::new(Transform {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix,
        })
    }

    pub fn rotation<W: Widget + Clone, R: ReadState<T=f64> + Clone>(child: W, rotation: R) -> Box<Transform<W, RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R>>> {
        let matrix: RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R> = Map1::read_map(rotation, |r| {
            Matrix4::from_angle_z(Deg(*r as f32))
        });

        Self::new(child, matrix)
    }

    pub fn scale<W: Widget + Clone, R: ReadState<T=f64> + Clone>(child: W, scale: R) -> Box<Transform<W, RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R>>> {
        let matrix: RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R> = Map1::read_map(scale, |s| {
            Matrix4::from_scale(*s as f32)
        });

        Self::new(child, matrix)
    }

    pub fn scale_non_uniform<W: Widget + Clone, R: ReadState<T=Dimension> + Clone>(
        child: W,
        scale: R,
    ) -> Box<Transform<W, RMap1<fn(&Dimension) -> Matrix4<f32>, Dimension, Matrix4<f32>, R>>> {
        let matrix: RMap1<fn(&Dimension) -> Matrix4<f32>, Dimension, Matrix4<f32>, R> = Map1::read_map(scale, |s| {
            Matrix4::from_nonuniform_scale(s.width as f32, s.height as f32, 1.0)
        });

        Self::new(child, matrix)
    }


    /*pub fn affine_2d<P1: Into<Matrix3<f32>>>(child: Box<dyn Widget>, affine: P1) -> Box<Self> {
        let affine_2d_to_affine_3d = |affine: &Matrix3<f32>| {
            Matrix4::from(*affine)
        };

        let matrix = MapOwnedState::new_with_default(affine.into(), affine_2d_to_affine_3d, Matrix4::identity());

        Box::new(Transform {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix: matrix.into()
        })
    }*/

}

impl<W: Widget + Clone, M: ReadState<T=Matrix4<f32>> + Clone> Transform<W, M> {
    pub fn with_anchor(mut self, anchor: BasicLayouter) -> Box<Self> {
        self.anchor = anchor;
        Box::new(self)
    }
}

impl<W: Widget + Clone, M: ReadState<T=Matrix4<f32>> + Clone> CommonWidget for Transform<W, M> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_rev(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        f(&mut self.child);
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

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<W: Widget + Clone, M: ReadState<T=Matrix4<f32>> + Clone> Render for Transform<W, M> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        let bounding_box = Rect::new(self.position, self.dimension);
        let matrix = *self.matrix.value();

        let new_transform = match self.anchor {
            BasicLayouter::TopLeading => {
                let center_x = (bounding_box.position.x()) as f32;
                let center_y = (bounding_box.position.y()) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Top => {
                let center_x = (bounding_box.position.x() + bounding_box.dimension.width / 2.0) as f32;
                let center_y = (bounding_box.position.y()) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::TopTrailing => {
                let center_x = (bounding_box.position.x() + bounding_box.dimension.width) as f32;
                let center_y = (bounding_box.position.y()) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Leading => {
                let center_x = (bounding_box.position.x()) as f32;
                let center_y = (bounding_box.position.y() + bounding_box.dimension.height / 2.0) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Center => {
                let center_x = (bounding_box.position.x() + bounding_box.dimension.width / 2.0) as f32;
                let center_y = (bounding_box.position.y() + bounding_box.dimension.height / 2.0) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Trailing => {
                let center_x = (bounding_box.position.x() + bounding_box.dimension.width) as f32;
                let center_y = (bounding_box.position.y() + bounding_box.dimension.height / 2.0) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::BottomLeading => {
                let center_x = (bounding_box.position.x()) as f32;
                let center_y = (bounding_box.position.y() + bounding_box.dimension.height) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Bottom => {
                let center_x = (bounding_box.position.x() + bounding_box.dimension.width / 2.0) as f32;
                let center_y = (bounding_box.position.y() + bounding_box.dimension.height) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::BottomTrailing => {
                let center_x = (bounding_box.position.x() + bounding_box.dimension.width) as f32;
                let center_y = (bounding_box.position.y() + bounding_box.dimension.height) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
        };

        context.transform(new_transform, |this| {
            self.foreach_child_mut(&mut |child| {
                child.render(this, env);
            });
        })
    }

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.capture_state(env);
        let matrix = *self.matrix.value();

        primitives.push(Primitive {
            kind: PrimitiveKind::Transform(matrix, self.anchor.clone()),
            bounding_box: Rect::new(self.position, self.dimension),
        });

        self.release_state(env);

        self.foreach_child_mut(&mut |child| {
            child.process_get_primitives(primitives, env);
        });

        primitives.push(Primitive {
            kind: PrimitiveKind::DeTransform,
            bounding_box: Rect::new(self.position, self.dimension),
        });
    }
}

impl<W: Widget + Clone, M: ReadState<T=Matrix4<f32>> + Clone> WidgetExt for Transform<W, M> {}
