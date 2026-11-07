use std::any::{type_name, Any};
use std::fmt::Debug;
use crate::any_debug::AnyDebug;
use crate::environment::environment::InternalEnvironmentValue;

/// # EnvironmentKey
/// A key into the environment that can be used to retrieve a value.
/// The value is determined by the associated type of the trait.
/// This allows us to insert multiple values of the same type into
/// the environment, as long as the key are different.
///
/// ## Defining a custom key
/// A common way of defining a custom key is by defining a unit struct
/// called `SomethingKey`. Often implementing Copy, Clone and Debug is
/// enough for using the key. It is also smart keep the key private,
/// and exposing accessor and insertion methods from your crate. This
/// allows you to control how a key is inserted into the environment.
///
/// ## Example implementation
/// ```
/// use carbide_core::environment::EnvironmentKey;
///
/// #[derive(Copy, Clone, Debug)]
/// struct CustomKey;
/// impl EnvironmentKey for CustomKey { type Value = u64; }
/// ```
///
pub trait EnvironmentKey: Any + Debug + 'static {
    /// The type of the value retrieved when getting the value by the key from
    /// the environment.
    type Value: Any + Debug + 'static;

    /// The mode of access of the value. This can either be shared (&) or exclusive (&mut).
    type Access: Access; // = Shared; #29661 #![feature(associated_type_defaults)]
}

pub trait EnvironmentKeyDefault: EnvironmentKey {
    fn default() -> Self::Value;
}

mod sealed {
    use carbide::environment::environment_key::{sealed, Shared, Exclusive};
    use crate::any_debug::AnyDebug;
    use crate::environment::environment::InternalEnvironmentValue;

    pub trait Sealed {
        /// `&'a T` for `Shared`, `&'a mut T` for `Exclusive`.
        type Ref<'a, T: ?Sized + 'a>;

        fn wrap<'a, T: AnyDebug + 'a>(name: &'static str, v: Self::Ref<'a, T>) -> InternalEnvironmentValue<'a>;
        fn extract<'a, 'e, T: AnyDebug>(v: &'a mut InternalEnvironmentValue<'e>) -> Option<Self::Ref<'a, T>>;
    }

    impl Sealed for Shared {
        type Ref<'a, T: ?Sized + 'a> = &'a T;

        fn wrap<'a, T: AnyDebug + 'a>(name: &'static str, v: &'a T) -> InternalEnvironmentValue<'a> {
            InternalEnvironmentValue::Value(name, v)
        }

        fn extract<'a, 'e, T: AnyDebug>(v: &'a mut InternalEnvironmentValue<'e>) -> Option<&'a T> {
            let v: &'a InternalEnvironmentValue<'e> = v;
            match v {
                InternalEnvironmentValue::Value(_, v) => v.downcast_ref::<T>(),
                InternalEnvironmentValue::ValueMut(_, v) => v.downcast_ref::<T>(),
            }
        }
    }
    impl Sealed for Exclusive {
        type Ref<'a, T: ?Sized + 'a> = &'a mut T;

        fn wrap<'a, T: AnyDebug + 'a>(name: &'static str, v: &'a mut T) -> InternalEnvironmentValue<'a> {
            InternalEnvironmentValue::ValueMut(name, v)
        }

        fn extract<'a, 'e, T: AnyDebug>(v: &'a mut InternalEnvironmentValue<'e>) -> Option<&'a mut T> {
            match v {
                InternalEnvironmentValue::ValueMut(_, v) => v.downcast_mut::<T>(),
                InternalEnvironmentValue::Value(_, _) => unreachable!("Extract for exclusive references can not be called on shared references, by construction"),
            }
        }
    }
}

/// Describes how a key's value is stored and handed out. Sealed: only
/// [`Shared`] and [`Exclusive`] exist.
pub trait Access: sealed::Sealed + 'static {}

/// The value is only available as `&Value`.
pub struct Shared;
/// The value is available as `&mut Value` (and `&Value` through `Environment::get`).
pub struct Exclusive;

impl Access for Shared {}
impl Access for Exclusive {}

/// `&Value` or `&mut Value`, depending on the key.
pub type EnvironmentValue<'a, K> = <<K as EnvironmentKey>::Access as sealed::Sealed>::Ref<'a, <K as EnvironmentKey>::Value>;

pub(super) fn extract_value<'a, 'e, K: EnvironmentKey + ?Sized>(
    v: &'a mut InternalEnvironmentValue<'e>,
) -> Option<EnvironmentValue<'a, K>> {
    <K::Access as sealed::Sealed>::extract::<K::Value>(v)
}

pub(super) fn wrap_value<K: EnvironmentKey + ?Sized>(
    v: EnvironmentValue<K>,
) -> InternalEnvironmentValue {
    <K::Access as sealed::Sealed>::wrap::<K::Value>(type_name::<K>(), v)
}