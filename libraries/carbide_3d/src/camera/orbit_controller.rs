use carbide::draw::Rect;
use carbide::event::{Key, KeyboardEvent, KeyboardEventContext, MouseButton, MouseEvent, MouseEventContext};
use carbide::math::{EuclideanSpace, InnerSpace, Point3, Vector3};
use carbide::state::{AnyReadState, IntoReadState, ReadState};

use crate::camera::controller::contains;
use crate::camera::{look_at, Camera, CameraController};

/// The highest and lowest the camera can orbit to, in degrees above or below the target. Short
/// of straight up or down, where the camera's up direction would be undefined.
const MAX_ELEVATION: f32 = 89.0;

/// Orbits the camera around a target point.
///
/// - Dragging with the left button inside the view turns the camera around the target: sideways
///   around the vertical axis, and up or down, stopping short of straight above or below.
/// - Scrolling inside the view turns the camera as dragging the same distance would.
/// - Pinching (a scale gesture) inside the view moves the camera towards or away from the
///   target.
/// - The arrow keys turn the camera in steps.
///
/// The camera always looks at the target. Every event starts from the camera's current position,
/// so the orbit follows a view matrix set from elsewhere.
#[derive(Debug, Clone)]
pub struct OrbitController {
    target: Box<dyn AnyReadState<T=Vector3<f32>>>,
    /// Degrees of turn per pixel dragged or scrolled.
    rotate_speed: f32,
    /// How strongly a scale gesture zooms: the distance is multiplied by `exp(-delta * speed)`.
    zoom_speed: f32,
    min_distance: f32,
    max_distance: f32,
}

impl OrbitController {
    /// Orbit around the origin.
    pub fn new() -> OrbitController {
        OrbitController {
            target: Box::new(Vector3::new(0.0, 0.0, 0.0)),
            rotate_speed: 0.4,
            zoom_speed: 1.0,
            min_distance: 0.01,
            max_distance: f32::MAX,
        }
    }

    /// Orbit around `target`, which can be a state.
    pub fn target<T: IntoReadState<Vector3<f32>>>(mut self, target: T) -> OrbitController {
        self.target = Box::new(target.into_read_state());
        self
    }

    /// Keep the camera between `min` and `max` from the target when zooming.
    pub fn distance(mut self, min: f32, max: f32) -> OrbitController {
        self.min_distance = min;
        self.max_distance = max;
        self
    }

    fn target_point(&self) -> Point3<f32> {
        Point3::from_vec(*self.target.value())
    }

    /// Turn the camera around the target: `azimuth` degrees around the vertical axis, and
    /// `elevation` degrees up. Positive azimuth moves the camera to its left, so the scene appears
    /// to turn right.
    pub fn orbit(&self, camera: &mut Camera, azimuth: f32, elevation: f32) {
        let target = self.target_point();
        let offset = camera.position() - target;
        let distance = offset.magnitude();

        if distance <= f32::EPSILON {
            return;
        }

        let current_azimuth = offset.x.atan2(offset.z).to_degrees();
        let current_elevation = (offset.y / distance).clamp(-1.0, 1.0).asin().to_degrees();

        let new_azimuth = (current_azimuth + azimuth).to_radians();
        let new_elevation = (current_elevation + elevation).clamp(-MAX_ELEVATION, MAX_ELEVATION).to_radians();

        let offset = Vector3::new(
            new_elevation.cos() * new_azimuth.sin(),
            new_elevation.sin(),
            new_elevation.cos() * new_azimuth.cos(),
        ) * distance;

        camera.set_view_matrix(look_at(target + offset, target));
    }

    /// Move the camera towards the target, to `factor` times its current distance, within the
    /// distance limits.
    pub fn zoom(&self, camera: &mut Camera, factor: f32) {
        let target = self.target_point();
        let offset = camera.position() - target;
        let distance = offset.magnitude();

        if distance <= f32::EPSILON {
            return;
        }

        let new_distance = (distance * factor).clamp(self.min_distance, self.max_distance);
        camera.set_view_matrix(look_at(target + offset * (new_distance / distance), target));
    }
}

impl Default for OrbitController {
    fn default() -> Self {
        OrbitController::new()
    }
}

impl CameraController for OrbitController {
    fn handle_mouse_event(&mut self, event: &MouseEvent, bounds: Rect, camera: &mut Camera, _ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Drag { button: MouseButton::Left, origin, delta_xy, .. } if contains(bounds, *origin) => {
                // Dragging right turns the scene right, bringing its left side into view, and
                // dragging down shows more of its top.
                self.orbit(camera, delta_xy.x as f32 * self.rotate_speed, delta_xy.y as f32 * self.rotate_speed);
            }
            MouseEvent::Scroll { x, y, mouse_position, .. } if contains(bounds, *mouse_position) => {
                // Scrolling orbits as dragging the content the same distance would. `x` is
                // positive when the content moves right, and `y` (flipped by carbide) negative
                // when it moves down.
                self.orbit(camera, *x as f32 * self.rotate_speed, -*y as f32 * self.rotate_speed);
            }
            MouseEvent::Scale(delta, position, _) if contains(bounds, *position) => {
                // A pinch outwards (magnify) arrives as a positive delta, and moves the camera
                // closer.
                self.zoom(camera, (-*delta as f32 * self.zoom_speed).exp());
            }
            _ => {}
        }
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, camera: &mut Camera, _ctx: &mut KeyboardEventContext) {
        match event {
            KeyboardEvent::Press { key: Key::ArrowLeft, .. } => self.orbit(camera, -15.0, 0.0),
            KeyboardEvent::Press { key: Key::ArrowRight, .. } => self.orbit(camera, 15.0, 0.0),
            KeyboardEvent::Press { key: Key::ArrowUp, .. } => self.orbit(camera, 0.0, -10.0),
            KeyboardEvent::Press { key: Key::ArrowDown, .. } => self.orbit(camera, 0.0, 10.0),
            _ => {}
        }
    }
}
