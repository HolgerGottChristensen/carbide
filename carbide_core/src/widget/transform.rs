use cgmath::{Deg, Matrix4, SquareMatrix};

use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Transform {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    anchor: BasicLayouter,
    #[state] matrix: TState<Matrix4<f32>>,
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

    pub fn new<P1: Into<TState<Matrix4<f32>>>>(child: Box<dyn Widget>, matrix: P1) -> Box<Self> {
        Box::new(Transform {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix: matrix.into(),
        })
    }

    pub fn rotation<P1: Into<F64State>>(child: Box<dyn Widget>, rotation: P1) -> Box<Self> {
        let rotation_to_matrix = |rotation: &f64, _: &_, _: &_| {
            Matrix4::from_angle_z(Deg(*rotation as f32))
        };

        let matrix = MapOwnedState::new_with_default(rotation.into(), rotation_to_matrix, Matrix4::identity());

        Box::new(Transform {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix: matrix.into(),
        })
    }

    pub fn scale<P1: Into<F64State>>(child: Box<dyn Widget>, scale: P1) -> Box<Self> {
        let scale_to_matrix = |scale: &f64, _: &_, _: &_| {
            Matrix4::from_scale(*scale as f32)
        };

        let matrix = MapOwnedState::new_with_default(scale.into(), scale_to_matrix, Matrix4::identity());

        Box::new(Transform {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            matrix: matrix.into(),
        })
    }

    pub fn scale_non_uniform<P1: Into<TState<Dimension>>>(child: Box<dyn Widget>, scale: P1) -> Box<Self> {
        let scale_to_matrix = |scale: &Dimension, _: &_, _: &_| {
            Matrix4::from_nonuniform_scale(scale.width as f32, scale.height as f32, 1.0)
        };

        let matrix = MapOwnedState::new_with_default(scale.into(), scale_to_matrix, Matrix4::identity());

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

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
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
