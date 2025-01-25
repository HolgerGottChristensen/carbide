use cgmath::{Deg, Matrix4, Point3, Vector3};

use carbide_macro::carbide_default_builder2;

use crate::draw::{Alignment, Angle, Dimension, Position, Rect};
use crate::render::{Render, RenderContext};
use crate::state::{IntoReadState, ReadState};
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetSync};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Rotation3DEffect<R1, R2, C> where R1: ReadState<T = Angle>, R2: ReadState<T = Angle>, C: Widget {
    #[id] id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    anchor: Alignment,
    #[state] rotation_x: R1,
    #[state] rotation_y: R2,
    fov: f64,
}

impl Rotation3DEffect<Angle, Angle, Empty> {
    #[carbide_default_builder2]
    pub fn new<R1: IntoReadState<Angle>, R2: IntoReadState<Angle>, C: Widget>(
        child: C,
        rotation_x: R1,
        rotation_y: R2,
    ) -> Rotation3DEffect<R1::Output, R2::Output, C> {
        Rotation3DEffect {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            anchor: Alignment::Center,
            rotation_x: rotation_x.into_read_state(),
            rotation_y: rotation_y.into_read_state(),
            fov: 1.15,
        }
    }
}

impl<R1: ReadState<T = Angle>, R2: ReadState<T = Angle>, C: Widget> Rotation3DEffect<R1, R2, C> {
    pub fn with_anchor(mut self, anchor: Alignment) -> Self {
        self.anchor = anchor;
        self
    }

    /// Warning: The FOV is acting strange and the default is 1.15
    pub fn with_fov(mut self, fov: f64) -> Self {
        self.fov = fov;
        self
    }
}

impl<R1: ReadState<T = Angle> + Clone, R2: ReadState<T = Angle> + Clone, C: Widget> CommonWidget for Rotation3DEffect<R1, R2, C> {
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

impl<R1: ReadState<T = Angle>, R2: ReadState<T = Angle>, C: Widget> Render for Rotation3DEffect<R1, R2, C> {
    fn render(&mut self, context: &mut RenderContext) {
        self.sync(context.env);
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
        let matrix = Matrix4::from_angle_x(Deg(self.rotation_x.value().degrees() as f32));
        let matrix = matrix * Matrix4::from_angle_y(Deg(self.rotation_y.value().degrees() as f32));
        let matrix = perspective * view * matrix;
        let bounding_box = Rect::new(self.position, self.dimension);


        let new_transform = match self.anchor {
            Alignment::TopLeading => {
                let center_x = (bounding_box.position.x) as f32;
                let center_y = (bounding_box.position.y) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            Alignment::Top => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width / 2.0) as f32;
                let center_y = (bounding_box.position.y) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            Alignment::TopTrailing => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width) as f32;
                let center_y = (bounding_box.position.y) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            Alignment::Leading => {
                let center_x = (bounding_box.position.x) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height / 2.0) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            Alignment::Center => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width / 2.0) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height / 2.0) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            Alignment::Trailing => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height / 2.0) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            Alignment::BottomLeading => {
                let center_x = (bounding_box.position.x) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            Alignment::Bottom => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width / 2.0) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            Alignment::BottomTrailing => {
                let center_x = (bounding_box.position.x + bounding_box.dimension.width) as f32;
                let center_y = (bounding_box.position.y + bounding_box.dimension.height) as f32;
                Matrix4::from_translation(Vector3::new(center_x, center_y, 0.0))
                    * matrix
                    * Matrix4::from_translation(Vector3::new(-center_x, -center_y, 0.0))
            }
            Alignment::Custom(_, _) => {
                unimplemented!()
            }
        };

        context.transform(new_transform, |this| {
            self.child.render(this)
        });
    }
}