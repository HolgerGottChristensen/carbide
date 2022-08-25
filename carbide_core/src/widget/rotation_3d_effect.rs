use cgmath::{Deg, Matrix4, Point3, Vector3};
use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::BasicLayouter;
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::state::{ReadState, StateSync, TState};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Rotation3DEffect {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    anchor: BasicLayouter,
    #[state]
    rotation_x: TState<f64>,
    #[state]
    rotation_y: TState<f64>,
    fov: f64,
}

impl Rotation3DEffect {
    #[carbide_default_builder]
    pub fn new(
        child: Box<dyn Widget>,
        rotation_x: impl Into<TState<f64>>,
        rotation_y: impl Into<TState<f64>>,
    ) -> Box<Self> {}

    pub fn new(
        child: Box<dyn Widget>,
        rotation_x: impl Into<TState<f64>>,
        rotation_y: impl Into<TState<f64>>,
    ) -> Box<Self> {
        Box::new(Rotation3DEffect {
            id: WidgetId::new(),
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

impl Render for Rotation3DEffect {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.capture_state(env);
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

impl WidgetExt for Rotation3DEffect {}
