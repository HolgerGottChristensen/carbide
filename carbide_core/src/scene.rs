use std::ops::Deref;

use dyn_clone::DynClone;

use carbide_core::widget::AnyWidget;

pub trait Scene: AnyWidget + DynClone + 'static {
    fn request_redraw(&self);
}

impl AnyWidget for Box<dyn Scene> {}

impl Scene for Box<dyn Scene> {
    fn request_redraw(&self) {
        self.deref().request_redraw()
    }
}

dyn_clone::clone_trait_object!(Scene);