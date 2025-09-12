//use std::cell::RefCell;
//use crate::camera::camera_projection::CameraProjection;
//use crate::camera::Camera;
//use carbide::animation::AnimationManager;
//use carbide::environment::Environment;
//use carbide::math::{Matrix4, Quaternion, Vector3};
//use carbide_core::time::Instant;
//use dolly::prelude::CameraRig;
//use std::rc::Rc;
//use dolly::handedness::LeftHanded;
//
//#[derive(Debug, Clone)]
//pub struct DollyCamera {
//    projection: CameraProjection,
//    rig: Rc<RefCell<CameraRig<LeftHanded>>>,
//    last_frame_time: Option<Instant>
//}
//
//impl DollyCamera {
//    pub fn new(rig: CameraRig<LeftHanded>, projection: CameraProjection) -> Self {
//        DollyCamera {
//            projection,
//            rig: Rc::new(RefCell::new(rig)),
//            last_frame_time: None,
//        }
//    }
//}
//
//impl Camera for DollyCamera {
//    fn view(&self) -> Matrix4<f32> {
//        let transform = self.rig.borrow().final_transform;
//
//        let rotation = Quaternion::new(transform.rotation.s, transform.rotation.v.x, transform.rotation.v.y, transform.rotation.v.z);
//
//        let position = Vector3::new(transform.position.x, transform.position.y, transform.position.z);
//
//        Matrix4::from(rotation) + Matrix4::from_translation(position)
//    }
//
//    fn projection(&self) -> CameraProjection {
//        self.projection
//    }
//
//    fn update(&mut self, env: &mut Environment) {
//        AnimationManager::get(env, |manager| {
//            manager.request_animation_frame();
//
//            let frame_time = manager.frame_time();
//
//            if let Some(last_frame_time) = &mut self.last_frame_time {
//                let duration = frame_time.duration_since(*last_frame_time);
//
//                let last_transform = self.rig.borrow().final_transform;
//                let new_transform = self.rig.borrow_mut().update(duration.as_secs_f32());
//
//                // We are still currently animating. Animations that has frames where it does not move, will stop here.
//                if last_transform.position != new_transform.position || last_transform.rotation != new_transform.rotation {
//                    manager.request_animation_frame();
//                }
//
//                *last_frame_time = frame_time;
//            } else {
//                self.last_frame_time = Some(frame_time);
//                manager.request_animation_frame();
//            }
//        });
//    }
//}