use cgmath::{Deg, Matrix4, Point3, Vector3};

use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::layout::BasicLayouter;
use crate::render::Render;
use crate::state::{ReadState, StateSync};
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Rotation3DEffect<R1, R2, C> where R1: ReadState<T = f64>, R2: ReadState<T = f64>, C: Widget {
    id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    anchor: BasicLayouter,
    #[state]
    rotation_x: R1,
    #[state]
    rotation_y: R2,
    fov: f64,
}

impl Rotation3DEffect<f64, f64, Empty> {
    #[carbide_default_builder2]
    pub fn new<R1: ReadState<T = f64>, R2: ReadState<T = f64>, C: Widget>(
        child: C,
        rotation_x: R1,
        rotation_y: R2,
    ) -> Rotation3DEffect<R1, R2, C> {
        Rotation3DEffect {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: BasicLayouter::Center,
            rotation_x,
            rotation_y,
            fov: 1.15,
        }
    }
}

impl<R1: ReadState<T = f64>, R2: ReadState<T = f64>, C: Widget> Rotation3DEffect<R1, R2, C> {
    pub fn with_anchor(mut self, anchor: BasicLayouter) -> Self {
        self.anchor = anchor;
        self
    }

    /// Warning: The FOV is acting strange and the default is 1.15
    pub fn with_fov(mut self, fov: f64) -> Self {
        self.fov = fov;
        self
    }
}

impl<R1: ReadState<T = f64> + Clone, R2: ReadState<T = f64> + Clone, C: Widget> CommonWidget for Rotation3DEffect<R1, R2, C> {
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

impl<R1: ReadState<T = f64>, R2: ReadState<T = f64>, C: Widget> Render for Rotation3DEffect<R1, R2, C> {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
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
        let bounding_box = Rect::new(self.position, self.dimension);


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

        context.transform(new_transform, |this| {
            self.child.render(this, env)
        });
    }
}

impl<R1: ReadState<T = f64>, R2: ReadState<T = f64>, C: Widget> WidgetExt for Rotation3DEffect<R1, R2, C> {}
