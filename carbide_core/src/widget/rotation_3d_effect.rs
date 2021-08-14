use cgmath::{Deg, Matrix4, Point3, Vector3};

use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Rotation3DEffect {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    anchor: BasicLayouter,
    #[state] rotation_x: F64State,
    #[state] rotation_y: F64State,
    fov: f64,
}

impl Rotation3DEffect {
    pub fn new<P1: Into<F64State>, P2: Into<F64State>>(child: Box<dyn Widget>, rotation_x: P1, rotation_y: P2) -> Box<Self> {
        Box::new(Rotation3DEffect {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            rotation_x: rotation_x.into(),
            rotation_y: rotation_y.into(),
            fov: 1.15,
        })
    }

    pub fn with_anchor(mut self, anchor: BasicLayouter) -> Box<Self> {
        self.anchor = anchor;
        Box::new(self)
    }

    /// Warning: The FOV is acting strange and the default is 1.15
    pub fn with_fov(mut self, fov: f64) -> Box<Self> {
        self.fov = fov;
        Box::new(self)
    }
}

impl CommonWidget for Rotation3DEffect {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(self.child.deref())
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(self.child.deref_mut())
        }
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
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

impl Render for Rotation3DEffect {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        // I do not understand why the fov needs to be 1.15, because my intuition says it should be 45deg
        let fov = self.fov as f32;
        let perspective = cgmath::perspective(Deg(fov), 1.0, 1.0, 10.0);
        let angle_to_screen_center = 90.0;

        let outer_angle = 180.0 - (fov / 2.0) - angle_to_screen_center;

        let z = outer_angle.to_radians().tan() as f32;
        let up: Vector3<f32> = cgmath::Vector3::unit_y();
        let target: Point3<f32> = Point3::new(0.0, 0.0, 0.0);
        let eye: Point3<f32> = Point3::new(0.0, 0.0, z);

        let view = Matrix4::look_at_rh(eye, target, up);
        let matrix = Matrix4::from_angle_x(Deg(*self.rotation_x.value() as f32));
        let matrix = matrix * Matrix4::from_angle_y(Deg(*self.rotation_y.value() as f32));
        let matrix = perspective * view * matrix;

        primitives.push(Primitive {
            kind: PrimitiveKind::Transform(matrix, self.anchor.clone()),
            rect: Rect::new(self.position, self.dimension),
        });

        for child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::DeTransform,
            rect: Rect::new(self.position, self.dimension),
        });
    }
}

impl WidgetExt for Rotation3DEffect {}
