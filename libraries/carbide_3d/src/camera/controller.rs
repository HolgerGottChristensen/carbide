use std::fmt::Debug;

use dyn_clone::DynClone;

use carbide::draw::{Position, Rect};
use carbide::event::{KeyboardEvent, KeyboardEventContext, MouseEvent, MouseEventContext};
use carbide::lifecycle::UpdateContext;

use crate::camera::Camera;

/// Moves a view's camera in response to input.
///
/// A [`Scene3d`](crate::Scene3d) has at most one controller. It passes every mouse event, every
/// keyboard event and every update to it, together with the view's own camera, and for mouse
/// events the view's bounds. A controller moves the camera with
/// [`Camera::set_view_matrix`].
///
/// A controller keeps only its settings, and starts every event from the camera's current view
/// matrix, so a view matrix set by the app, or by another controller, needs no special handling.
/// A controller that animates between events requests animation frames only while it is moving.
pub trait CameraController: DynClone + Debug + 'static {
    /// Handle a mouse event. `bounds` is the view's area: events outside it are seen too, so a
    /// drag that started inside can continue outside.
    fn handle_mouse_event(&mut self, _event: &MouseEvent, _bounds: Rect, _camera: &mut Camera, _ctx: &mut MouseEventContext) {}

    /// Handle a keyboard event.
    fn handle_keyboard_event(&mut self, _event: &KeyboardEvent, _camera: &mut Camera, _ctx: &mut KeyboardEventContext) {}

    /// Called on every update, before the view renders.
    fn update(&mut self, _camera: &mut Camera, _ctx: &mut UpdateContext) {}
}

dyn_clone::clone_trait_object!(CameraController);

/// Whether `position` is inside `bounds`, edges included.
pub fn contains(bounds: Rect, position: Position) -> bool {
    position.x >= bounds.left()
        && position.x <= bounds.right()
        && position.y >= bounds.bottom().min(bounds.top())
        && position.y <= bounds.bottom().max(bounds.top())
}
