use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use dolly::drivers::Smooth;
use dolly::handedness::LeftHanded;
use dolly::prelude::{Arm, YawPitch};
use dolly::rig::CameraRig;
use carbide::animation::AnimationManager;
use carbide::CommonWidgetImpl;
use carbide::draw::{Dimension, Position};
use carbide::event::{AccessibilityEventHandler, EventHandler, Key, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEventHandler, OtherEventHandler, WindowEventHandler};
use carbide::focus::Focusable;
use carbide::lifecycle::{Update, UpdateContext};
use carbide::math::{Deg, Euler, Matrix4, Quaternion, Vector3};
use carbide::widget::{AnyWidget, CommonWidget, Identifiable, Widget, WidgetId, WidgetSync};
use crate::camera::Camera;
use crate::camera::camera_projection::CameraProjection;

#[derive(Clone, Debug)]
pub struct OrbitCamera {
    id: WidgetId,
    rig: Rc<RefCell<CameraRig::<LeftHanded>>>,
    last_frame_time: Option<Instant>,
    projection: CameraProjection,
}

impl OrbitCamera {
    pub fn new() -> OrbitCamera {
        let camera_rig = CameraRig::<LeftHanded>::builder()
            .with(YawPitch::new().yaw_degrees(0.0).pitch_degrees(0.0))
            //.with(Smooth::new_rotation(1.5))
            //.with(Arm::new(Vector3::unit_z() * 5.0))
            .build();

        OrbitCamera {
            id: WidgetId::new(),
            rig: Rc::new(RefCell::new(camera_rig)),
            last_frame_time: None,
            projection: CameraProjection::Perspective { vfov: 60.0, near: 0.05 },
        }
    }
}

impl MouseEventHandler for OrbitCamera {}

impl CommonWidget for OrbitCamera {
    CommonWidgetImpl!(self, child: ());

    fn position(&self) -> Position {
        todo!()
    }

    fn set_position(&mut self, position: Position) {
        todo!()
    }

    fn dimension(&self) -> Dimension {
        todo!()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        todo!()
    }
}

impl Identifiable for OrbitCamera {
    fn id(&self) -> WidgetId {
        self.id
    }
}

impl WidgetSync for OrbitCamera {}

impl Focusable for OrbitCamera {}

impl KeyboardEventHandler for OrbitCamera {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        match event {
            KeyboardEvent::Press { key: Key::ArrowLeft, .. } => {
                self.rig.borrow_mut().driver_mut::<YawPitch>().rotate_yaw_pitch(15.0, 0.0);
                println!("Yaw");
            }
            KeyboardEvent::Press { key: Key::ArrowRight, .. } => {
                self.rig.borrow_mut().driver_mut::<YawPitch>().rotate_yaw_pitch(-15.0, 0.0);
                println!("Yaw");
            }
            KeyboardEvent::Press { key: Key::ArrowUp, .. } => {
                self.rig.borrow_mut().driver_mut::<YawPitch>().rotate_yaw_pitch(0.0, -10.0);
                println!("Yaw");
            }
            KeyboardEvent::Press { key: Key::ArrowDown, .. } => {
                self.rig.borrow_mut().driver_mut::<YawPitch>().rotate_yaw_pitch(0.0, 10.0);
                println!("Yaw");
            }
            _ => ()
        }
    }
}

impl WindowEventHandler for OrbitCamera {}

impl AccessibilityEventHandler for OrbitCamera {}

impl OtherEventHandler for OrbitCamera {}

impl Update for OrbitCamera {
    fn update(&mut self, ctx: &mut UpdateContext) {
        AnimationManager::get(ctx.env, |manager| {
            manager.request_animation_frame();

            let frame_time = manager.frame_time();
            if let Some(last_frame_time) = &mut self.last_frame_time {
                let duration = frame_time.duration_since(*last_frame_time);
                let last_transform = self.rig.borrow().final_transform;
                let new_transform = self.rig.borrow_mut().update(duration.as_secs_f32());
                // We are still currently animating. Animations that has frames where it does not move, will stop here.
                if last_transform.position != new_transform.position || last_transform.rotation != new_transform.rotation {
                    manager.request_animation_frame();
                }
                *last_frame_time = frame_time;
            } else {
                self.last_frame_time = Some(frame_time);
                manager.request_animation_frame();
            }
        })
    }
}

impl Camera for OrbitCamera {
    fn view(&self) -> Matrix4<f32> {
        let transform = self.rig.borrow().final_transform;

        let rotation = Quaternion::new(transform.rotation.s, transform.rotation.v.x, transform.rotation.v.y, transform.rotation.v.z);

        let euler = Euler::from(rotation);

        println!("x: {:?}, y: {:?}, z: {:?}, rx: {:?}, ry: {:?}, rz: {:?}", transform.position.x, transform.position.y, transform.position.z, Deg::from(euler.x), Deg::from(euler.y), Deg::from(euler.z));

        let position = Vector3::new(transform.position.x, transform.position.y, transform.position.z);

        //Matrix4::from_translation(Vector3::new(-2.0, 0.0, 5.0))
        Matrix4::from_translation(position) * Matrix4::from(rotation)
    }

    fn projection(&self) -> CameraProjection {
        self.projection
    }
}
