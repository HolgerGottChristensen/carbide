use std::ops::Deref;

use dyn_clone::DynClone;
use carbide_core::widget::AnyWidget;

pub trait AnyScene: AnyWidget + DynClone + 'static {
    fn request_redraw(&self) -> bool;

    fn has_application_focus(&self) -> bool;

    fn is_daemon(&self) -> bool;
}

impl AnyWidget for Box<dyn AnyScene> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}

impl AnyScene for Box<dyn AnyScene> {
    fn request_redraw(&self) -> bool {
        self.deref().request_redraw()
    }

    fn has_application_focus(&self) -> bool {
        self.deref().has_application_focus()
    }

    fn is_daemon(&self) -> bool {
        self.deref().is_daemon()
    }
}

dyn_clone::clone_trait_object!(AnyScene);


pub trait Scene: AnyScene + Clone + private::Sealed {}

impl<T> Scene for T where T: AnyScene + Clone {}

mod private {
    use crate::scene::AnyScene;

    // This disallows implementing Scene manually, and requires something to implement
    // AnyScene to implement Scene.
    pub trait Sealed {}

    impl<T> Sealed for T where T: AnyScene {}
}