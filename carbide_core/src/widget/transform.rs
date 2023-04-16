use cgmath::{Deg, Matrix4, SquareMatrix, Vector3};
use carbide_core::render::RenderContext;

use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::BasicLayouter;
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::state::{MapOwnedState, ReadState, StateSync, TState};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Transform {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    anchor: BasicLayouter,
    #[state]
    matrix: TState<Matrix4<f32>>,
}

impl Transform {
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

    #[carbide_default_builder]
    pub fn new(child: Box<dyn Widget>, matrix: impl Into<TState<Matrix4<f32>>>) -> Box<Self> {}

    pub fn new(child: Box<dyn Widget>, matrix: impl Into<TState<Matrix4<f32>>>) -> Box<Self> {
        Box::new(Transform {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix: matrix.into(),
        })
    }

    pub fn rotation(child: Box<dyn Widget>, rotation: impl Into<TState<f64>>) -> Box<Self> {
        let rotation_to_matrix =
            |rotation: &f64, _: &_, _: &_| Matrix4::from_angle_z(Deg(*rotation as f32));

        let matrix = MapOwnedState::new_with_default(
            rotation.into(),
            rotation_to_matrix,
            Matrix4::identity(),
        );

        Box::new(Transform {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix: matrix.into(),
        })
    }

    pub fn scale(child: Box<dyn Widget>, scale: impl Into<TState<f64>>) -> Box<Self> {
        let scale_to_matrix = |scale: &f64, _: &_, _: &_| Matrix4::from_scale(*scale as f32);

        let matrix =
            MapOwnedState::new_with_default(scale.into(), scale_to_matrix, Matrix4::identity());

        Box::new(Transform {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix: matrix.into(),
        })
    }

    pub fn scale_non_uniform(
        child: Box<dyn Widget>,
        scale: impl Into<TState<Dimension>>,
    ) -> Box<Self> {
        let scale_to_matrix = |scale: &Dimension, _: &_, _: &_| {
            Matrix4::from_nonuniform_scale(scale.width as f32, scale.height as f32, 1.0)
        };

        let matrix =
            MapOwnedState::new_with_default(scale.into(), scale_to_matrix, Matrix4::identity());

        Box::new(Transform {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix: matrix.into(),
        })
    }

    pub fn with_anchor(mut self, anchor: BasicLayouter) -> Box<Self> {
        self.anchor = anchor;
        Box::new(self)
    }
}

impl CommonWidget for Transform {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child(&self, f: &mut dyn FnMut(&dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }


    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
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

impl Render for Transform {
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
            for mut child in self.children_mut() {
                child.render(this, env);
            }
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

        for mut child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::DeTransform,
            bounding_box: Rect::new(self.position, self.dimension),
        });
    }
}

impl WidgetExt for Transform {}
