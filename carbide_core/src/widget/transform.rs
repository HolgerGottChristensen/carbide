use cgmath::{Deg, Matrix4, Vector3};

use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position, Rect};
use crate::layout::BasicLayouter;
use crate::render::Render;
use crate::render::RenderContext;
use crate::state::{Map1, ReadState, StateSync};
use crate::state::RMap1;
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Transform<W, M> where W: AnyWidget + Clone, M: ReadState<T=Matrix4<f32>> {
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
    pub fn new<W: AnyWidget + Clone, M: ReadState<T=Matrix4<f32>>>(child: W, matrix: M) -> Transform<W, M> {
        Transform {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix,
        }
    }

    pub fn rotation<W: AnyWidget + Clone, R: ReadState<T=f64> + Clone>(child: W, rotation: R) -> Transform<W, RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R>> {
        let matrix: RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R> = Map1::read_map(rotation, |r| {
            Matrix4::from_angle_z(Deg(*r as f32))
        });

        Self::new(child, matrix)
    }

    pub fn scale<W: AnyWidget + Clone, R: ReadState<T=f64> + Clone>(child: W, scale: R) -> Transform<W, RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R>> {
        let matrix: RMap1<fn(&f64) -> Matrix4<f32>, f64, Matrix4<f32>, R> = Map1::read_map(scale, |s| {
            Matrix4::from_scale(*s as f32)
        });

        Self::new(child, matrix)
    }

    pub fn scale_non_uniform<W: AnyWidget + Clone, R: ReadState<T=Dimension> + Clone>(
        child: W,
        scale: R,
    ) -> Transform<W, RMap1<fn(&Dimension) -> Matrix4<f32>, Dimension, Matrix4<f32>, R>> {
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

impl<W: AnyWidget + Clone, M: ReadState<T=Matrix4<f32>>> Transform<W, M> {
    pub fn with_anchor(mut self, anchor: BasicLayouter) -> Self {
        self.anchor = anchor;
        self
    }
}

impl<W: AnyWidget + Clone, M: ReadState<T=Matrix4<f32>>> CommonWidget for Transform<W, M> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_rev(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
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

impl<W: AnyWidget + Clone, M: ReadState<T=Matrix4<f32>>> Render for Transform<W, M> {
    fn render(&mut self, context: &mut RenderContext) {
        self.capture_state(context.env);
        let bounding_box = Rect::new(self.position, self.dimension);
        let matrix = *self.matrix.value();

        let new_transform = match self.anchor {
            BasicLayouter::TopLeading => {
                let center_x = (bounding_box.position.x) as f32;
                let center_y = (bounding_box.position.y) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Top => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width / 2.0) as f32;
                let center_y = (bounding_box.position.y) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::TopTrailing => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width) as f32;
                let center_y = (bounding_box.position.y) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Leading => {
                let center_x = (bounding_box.position.x) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height / 2.0) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Center => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width / 2.0) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height / 2.0) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Trailing => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height / 2.0) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::BottomLeading => {
                let center_x = (bounding_box.position.x) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::Bottom => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width / 2.0) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            BasicLayouter::BottomTrailing => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
        };


        self.release_state(context.env);

        context.transform(new_transform, |this| {
            self.foreach_child_mut(&mut |child| {
                child.render(this);
            });
        })
    }
}

impl<W: AnyWidget + Clone, M: ReadState<T=Matrix4<f32>>> WidgetExt for Transform<W, M> {}
