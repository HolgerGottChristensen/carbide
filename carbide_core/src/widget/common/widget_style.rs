use std::any::{type_name, TypeId};
use std::fmt::Debug;
use dyn_clone::DynClone;

pub trait WidgetStyle: Debug + DynClone + 'static {
    fn key(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn name(&self) -> &'static str {
        type_name::<Self>()
    }
}

impl<T> WidgetStyle for T where T: Clone + Debug + 'static {}