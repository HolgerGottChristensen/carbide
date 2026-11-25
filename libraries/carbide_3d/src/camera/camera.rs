use std::fmt::{Debug, Formatter};

use carbide::environment::Environment;
use carbide::math::{EuclideanSpace, Matrix4, Point3, SquareMatrix, Vector3};
use carbide::state::{AnyState, LocalState, ReadState, StateSync};

use crate::camera::{CameraSpec, Projection};

/// The camera of one view of a scene: where it looks from, and how it projects.
///
/// Both parts are writable states, so a camera controller can move the camera and zoom it, and
/// the app can read and set where it looks from and how it projects.
///
/// Cloning a camera copies its current view matrix and projection into new states, also when the
/// camera was given states with [`view`](Camera::view) or [`projection`](Camera::projection): two
/// views never move or zoom each other's camera by accident. To have two cameras share one view
/// matrix or projection, give each of them the same state.
pub struct Camera {
    view: Box<dyn AnyState<T=Matrix4<f32>>>,
    projection: Box<dyn AnyState<T=Projection>>,
}

impl Camera {
    /// A camera with the given projection, at the origin looking along +Z, with +Y up.
    pub fn new(projection: Projection) -> Camera {
        Camera {
            view: LocalState::new(Matrix4::identity()).as_dyn(),
            projection: LocalState::new(projection).as_dyn(),
        }
    }

    /// A perspective camera with a 60° vertical field of view.
    pub fn perspective() -> Camera {
        Camera::new(Projection::default())
    }

    /// An orthographic camera showing `height` world units from the bottom to the top of the
    /// view.
    pub fn orthographic(height: f32) -> Camera {
        Camera::new(Projection::Orthographic { height })
    }

    /// Use `projection` as the camera's projection. A controller writes into this state when it
    /// zooms an orthographic camera.
    pub fn projection<S: AnyState<T=Projection> + Clone>(mut self, projection: S) -> Camera {
        self.projection = projection.as_dyn();
        self
    }

    /// Make the camera a perspective camera with a vertical field of view of `vfov` degrees,
    /// keeping its near plane if it was a perspective camera already.
    pub fn fov(self, vfov: f32) -> Camera {
        let near = match *self.projection.value() {
            Projection::Perspective { near, .. } => near,
            _ => 0.1,
        };

        self.projection(LocalState::new(Projection::Perspective { vfov, near }))
    }

    /// Place the camera at `eye`, looking at `target`, with +Y up.
    pub fn look_at(mut self, eye: Point3<f32>, target: Point3<f32>) -> Camera {
        self.view = LocalState::new(look_at(eye, target)).as_dyn();
        self
    }

    /// Use `view` as the camera's view matrix, the transform from world to camera space. A
    /// controller writes into this state when it moves the camera.
    pub fn view<S: AnyState<T=Matrix4<f32>> + Clone>(mut self, view: S) -> Camera {
        self.view = view.as_dyn();
        self
    }

    /// The view matrix: the transform from world space to camera space.
    pub fn view_matrix(&self) -> Matrix4<f32> {
        *self.view.value()
    }

    /// Set the view matrix, writing through to the state the camera was given.
    pub fn set_view_matrix(&mut self, view: Matrix4<f32>) {
        self.view.set_value_dyn(view);
    }

    /// The camera's position in world space.
    pub fn position(&self) -> Point3<f32> {
        let world = self.view_matrix().invert().unwrap_or(Matrix4::identity());
        Point3::from_vec(world.w.truncate())
    }

    /// The camera's current projection.
    pub fn projection_value(&self) -> Projection {
        *self.projection.value()
    }

    /// Set the projection, writing through to the state the camera was given.
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection.set_value_dyn(projection);
    }

    /// The matrices a render context draws with, for a layer with the given width divided by
    /// height.
    pub fn spec(&self, aspect_ratio: f32) -> CameraSpec {
        let view = self.view_matrix();

        CameraSpec {
            view,
            view_projection: self.projection_value().matrix(aspect_ratio) * view,
        }
    }
}

/// The view matrix of a camera at `eye`, looking at `target`, with +Y up.
pub fn look_at(eye: Point3<f32>, target: Point3<f32>) -> Matrix4<f32> {
    Matrix4::look_at_lh(eye, target, Vector3::unit_y())
}

impl Default for Camera {
    fn default() -> Self {
        Camera::perspective()
    }
}

impl Clone for Camera {
    fn clone(&self) -> Self {
        Camera {
            view: LocalState::new(self.view_matrix()).as_dyn(),
            projection: LocalState::new(self.projection_value()).as_dyn(),
        }
    }
}

impl Debug for Camera {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Camera")
            .field("view", &self.view_matrix())
            .field("projection", &self.projection_value())
            .finish()
    }
}

impl StateSync for Camera {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.view.sync(env) | self.projection.sync(env)
    }
}
