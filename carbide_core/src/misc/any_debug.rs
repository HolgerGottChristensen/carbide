use std::any::Any;
use std::fmt::Debug;

pub trait AnyDebug: Any + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Gets the type name of `self`
    fn type_name(&self) -> &'static str;
}

impl<T> AnyDebug for T where T: Any + Debug {
    #[inline(always)]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline(always)]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline(always)]
    fn type_name(&self) -> &'static str {
        core::any::type_name::<T>()
    }
}

impl dyn AnyDebug {
    #[inline(always)]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }

    #[inline(always)]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}