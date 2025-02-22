use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hasher};
use std::marker::PhantomData;
use std::mem::transmute;
use crate::misc::any_debug::AnyDebug;

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
}


pub trait EnvironmentKeyable: Debug + 'static {
    type Output: Any + Debug + 'static;

    fn get(&self, stack: &Environment) -> Option<Self::Output>;

    fn with(&self, value: &Self::Output, stack: &mut Environment, f: impl FnOnce(&mut Environment));

    fn with_all(values: &[(Self, Self::Output)], stack: &mut Environment, f: impl FnOnce(&mut Environment)) where Self: Sized {
        match values {
            [(key, value), rest @ ..] => {
                key.with(value, stack, |inner| {
                    Self::with_all(rest, inner, f)
                })
            }
            [] => {
                f(stack)
            }
        }
    }
}

/// # Environment
///
/// The environment contains scoped state at for a given widget, that can be provided
/// by widgets further up the hierarchy of widgets. The environment is useful for many
/// different aspects of the framework. It can store environment specified colors, font sizes,
/// styles, managers and such.
///
/// ## Scoping
/// The environment contains scoped values, and allows for shadowing. This is for example
/// used when defining styles for carbide controls. Suppose we have a default button style.
/// This could be inserted at the outermost position in the environment. A specific style
/// can be then inserted, overriding the other style, for all widgets contained in the scope
/// of the environment. When leaving the scope, the outermost value can be accessed again.
/// It is not possible to access the value from the outer environment in the inner environment.
///
/// ## Mutability and immutability
/// The environment contains both immutable and mutable values. These are stored, inserted and
/// retrieved separately. This has an effect on scoping, since a value can be inserted with the
/// same key in both the immutable values and mutable values.
///
/// ## Lookup
/// The environment has very fast lookup, since the keys for the environments are generated at
/// compile time, and the hasher for the hashmap storage are just an identity operation. Since
/// the keys hashes are generated at compile time, and are generated with using a good hashing
/// algorithm, collisions should never occur.
pub struct Environment<'a> {
    data: HashMap<TypeId, &'a dyn AnyDebug, BuildTypeIdHasher>,
    data_mut: HashMap<TypeId, &'a mut dyn AnyDebug, BuildTypeIdHasher>,
    _marker: PhantomData<*const ()>, // Ensures the type is not Send and not Sync
}


impl<'a> Environment<'a> {
    /// Create a new empty environment.
    /// The environment can only be used in the thread it was created.
    pub fn new() -> Self {
        Environment {
            data: HashMap::with_hasher(BuildTypeIdHasher::default()),
            data_mut: HashMap::with_hasher(BuildTypeIdHasher::default()),
            _marker: Default::default(),
        }
    }
}

impl<'a> Environment<'a> {
    /// Get a value from the environment by the *key* type. The *key* must implement `EnvironmentKey`.
    /// The value is retrieved from the immutable part of the environment. Only values inserted
    /// in the immutable part of the environment can be retrieved by this method.
    ///
    /// If the value does not exist in the immutable part of the environment, `None` will be returned.
    pub fn get<K: EnvironmentKey + ?Sized>(&self) -> Option<&K::Value> {
        let id = TypeId::of::<K>();

        Option::map(self.data.get(&id), |value| {
            value.downcast_ref()
        }).flatten()
    }

    /// Get a mutable value from the environment by the *key* type. The *key* must implement `EnvironmentKey`.
    /// The mutable value is retrieved from the mutable part of the environment. Only values
    /// inserted into the mutable part of the environment can be retrieved by this method.
    ///
    /// If the value does not exist in the mutable part of the environment, `None` will be returned.
    pub fn get_mut<K: EnvironmentKey + ?Sized>(&mut self) -> Option<&mut K::Value> {
        let id = TypeId::of::<K>();

        Option::map(self.data_mut.get_mut(&id), |value| {
            value.downcast_mut()
        }).flatten()
    }

    pub fn value<K: EnvironmentKeyable>(&self, key: &K) -> Option<K::Output> {
        key.get(self)
    }
}

impl<'a> Environment<'a> {
    #[allow(unsafe_code)]
    pub fn with<'b, K: EnvironmentKey + ?Sized>(&mut self, v: &'b K::Value, f: impl FnOnce(&mut Environment)) where 'a: 'b {
        let id = TypeId::of::<K>();

        // If any existing value existed in the data with the key, remove and store it.
        let old = self.data.remove(&id);

        // SAFETY: The transmuted map has a shorter lifetime. The map is
        // only used and accessed in the scope of the closure. We insert the
        // reference only for the duration of the closure, and ensure it is
        // removed again, before returning from the function, ensuring the
        // reference to the value v does is not in the map after the closure,
        // and thus making sure it does not escape.
        let transmuted: &'b mut Environment<'b> = unsafe { transmute(self) };

        transmuted.data.insert(id, v);

        f(transmuted);

        if let Some(old) = old {
            transmuted.data.insert(id, old);
        } else {
            transmuted.data.remove(&id);
        }
    }

    #[allow(unsafe_code)]
    pub fn with_mut<'b, K: EnvironmentKey + ?Sized>(&mut self, v: &'b mut K::Value, f: impl FnOnce(&mut Environment)) where 'a: 'b {
        let id = TypeId::of::<K>();

        // If any existing value existed in the data with the key, remove and store it.
        let old = self.data_mut.remove(&id);

        // SAFETY: The transmuted map has a shorter lifetime. The map is
        // only used and accessed in the scope of the closure. We insert the
        // reference only for the duration of the closure, and ensure it is
        // removed again, before returning from the function, ensuring the
        // reference to the value v does is not in the map after the closure,
        // and thus making sure it does not escape.
        let transmuted: &'b mut Environment<'b> = unsafe { transmute(self) };

        transmuted.data_mut.insert(id, v);

        f(transmuted);

        if let Some(old) = old {
            transmuted.data_mut.insert(id, old);
        } else {
            transmuted.data_mut.remove(&id);
        }
    }
}

type BuildTypeIdHasher = BuildHasherDefault<TypeIdHasher>;

#[derive(Default, Clone, Copy, Debug)]
struct TypeIdHasher(u64);

impl Hasher for TypeIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        unreachable!("The hasher is only used with TypeId and it uses write_u64")
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

#[cfg(test)]
mod tests {
    use crate::environment::environment::{EnvironmentKey, Environment};

    #[derive(Copy, Clone, Debug)]
    struct TestKey;
    impl EnvironmentKey for TestKey { type Value = u64; }

    #[test]
    fn simple() {
        let mut map = Environment::new();

        map.with::<TestKey>(&42, |inner| {
            let get = inner.get::<TestKey>();
            println!("{:?}", get);
        });

        let get = map.get::<TestKey>();
        println!("{:?}", get);
    }
}