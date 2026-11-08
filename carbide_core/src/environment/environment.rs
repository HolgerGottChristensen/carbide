use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{BuildHasherDefault, Hasher};
use std::marker::PhantomData;
use std::mem::transmute;
use crate::common::any_debug::AnyDebug;
use crate::environment::{EnvironmentKey, EnvironmentValue, Exclusive};
use crate::environment::environment_key::{extract_value, wrap_value};

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
/// The environment contains both immutable and mutable values. When storing a mutable value
/// you are allowed to read it both as immutable and mutable. But if you insert an immutable
/// value and you try to receive it mutably, the framework will return an error.
///
/// ## Lookup
/// The environment has very fast lookup, since the keys for the environments are generated at
/// compile time, and the hasher for the hashmap storage are just an identity operation. Since
/// the keys hashes are generated at compile time, and are generated with using a good hashing
/// algorithm, collisions should never occur.
pub struct Environment<'a> {
    data: HashMap<TypeId, InternalEnvironmentValue<'a>, BuildTypeIdHasher>,
    _marker: PhantomData<*const ()>, // Ensures the type is not Send and not Sync
}

#[doc(hidden)]
#[derive(Debug)]
pub(super) enum InternalEnvironmentValue<'a> {
    Value(&'static str, &'a dyn AnyDebug),
    ValueMut(&'static str, &'a mut dyn AnyDebug)
}

impl<'a> InternalEnvironmentValue<'a> {
    fn as_any(&self) -> &dyn AnyDebug {
        match self {
            InternalEnvironmentValue::Value(_, v) => *v,
            InternalEnvironmentValue::ValueMut(_, v) => &**v,
        }
    }
}

impl Default for Environment<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Environment<'a> {
    /// Create a new empty environment.
    /// The environment can only be used in the thread it was created.
    pub fn new() -> Self {
        Environment {
            data: HashMap::with_hasher(BuildTypeIdHasher::default()),
            _marker: Default::default(),
        }
    }
}

impl<'a> Environment<'a> {
    /// Get a value from the environment by the *key* type. The *key* must implement `EnvironmentKey`.
    ///
    /// If the value does not exist as part of the environment, `None` will be returned.
    /// This method can be used to to get both shared and exclusive values as &Value
    pub fn get<K: EnvironmentKey + ?Sized>(&self) -> Option<&K::Value> {
        self.data
            .get(&TypeId::of::<K>())?
            .as_any()
            .downcast_ref()
    }

    /// Get a mutable value from the environment by the *key* type. The *key* must implement `EnvironmentKey`.
    /// Only mutable values inserted into the environment can be retrieved by this method.
    ///
    /// If the value does not exist in the environment or was not added as mutable, `None` will be returned.
    ///
    /// WARNING: It is unsafe to mem swap the reference returned by this function. This is because the
    /// value you are extracting and swapping with have a lifetime that has been artificially shortened
    /// due to the scoping rules of the environment. If you swap with some value, that actually has
    /// a short lifetime, it will be invalid when the environment is used in the outer scopes.
    pub fn get_mut<K: EnvironmentKey<Access = Exclusive> + ?Sized>(&mut self) -> Option<&mut K::Value> {
        match self.data.get_mut(&TypeId::of::<K>())? {
            InternalEnvironmentValue::ValueMut(_, v) => v.downcast_mut(),
            InternalEnvironmentValue::Value(_, _) => None,
        }
    }

    pub fn get_many<K: GetMany>(&mut self) -> K::Output<'_> {
        K::get(self)
    }

    pub fn value<K: EnvironmentKeyable>(&self, key: &K) -> Option<K::Output> {
        key.get(self)
    }
}

/// Restores the previous value for `id` (or removes the entry) when dropped,
/// including during unwinding.
struct RestoreGuard<'b> {
    env: &'b mut Environment<'b>,
    id: TypeId,
    old: Option<InternalEnvironmentValue<'b>>,
}

impl<'b> Drop for RestoreGuard<'b> {
    fn drop(&mut self) {
        match self.old.take() {
            Some(old) => {
                self.env.data.insert(self.id, old);
            }
            None => {
                self.env.data.remove(&self.id);
            }
        }
    }
}

impl<'a> Environment<'a> {
    /// `v` is `&K::Value` for shared keys and `&mut K::Value` for exclusive keys.
    #[allow(unsafe_code)]
    pub fn with<'b, K: EnvironmentKey + ?Sized>(&mut self, v: EnvironmentValue<'b, K>, f: impl FnOnce(&mut Environment)) where 'a: 'b {
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

        transmuted.data.insert(id, wrap_value::<K>(v));

        let guard = RestoreGuard { env: transmuted, id, old };
        f(&mut *guard.env);
        // `guard` drops here (or during unwinding) and restores the map.
    }
}

impl Environment<'static> {
    // This is only safe because its static, and no other lifetime lives longer than static.
    // If it was not static, and we insert in an env with 'b,
    // and it is not removed before going back to 'a, it will be an invalid reference.
    pub fn insert<K: EnvironmentKey + ?Sized>(&mut self, v: EnvironmentValue<'static, K>) {
        let id = TypeId::of::<K>();

        self.data.insert(id, wrap_value::<K>(v));
    }
}

impl<'a> Environment<'a> {

    /// Take a value from the environment, and call the provided function with the value and the remaining environment.
    ///
    /// This is useful in two scenarios:
    /// 1. If the environment contains a type with a method that needs to be called with an environment.
    ///    This could for example be a factory method that constructs a type, but needs other parts of the
    ///    environment for the construction. Alternatively you can use get_many and extract the exact types
    ///    you depend upon, and narrow the constructor of your type.
    /// 2. If you need to remove some values from the environment before letting it be used by a descendent.
    ///    This is additional to possible shadowing, but instead of providing a shadowing value, the environment value
    ///    with the key is empty.
    pub fn extract<K: EnvironmentKey + ?Sized>(&mut self, f: impl FnOnce(Option<&K::Value>, &mut Environment)) {
        let id = TypeId::of::<K>();
        let value = self.data.remove(&id);

        let mapped = value.as_ref().and_then(|v| v.as_any().downcast_ref());
        f(mapped, self);

        // TODO: Add drop guard to handle panic unwinds
        if let Some(val) = value {
            self.data.insert(id, val);
        }
    }

    pub fn extract_mut<K: EnvironmentKey<Access = Exclusive> + ?Sized>(
        &mut self,
        f: impl FnOnce(Option<&mut K::Value>, &mut Environment),
    ) {
        let id = TypeId::of::<K>();
        let mut value = self.data.remove(&id);

        let mapped = value.as_mut().and_then(|v| extract_value::<K>(v));
        f(mapped, self);

        // TODO: Add drop guard to handle panic unwinds
        if let Some(val) = value {
            self.data.insert(id, val);
        }
    }
}

impl<'a> Debug for Environment<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct(type_name::<Environment<'a>>());

        for val in self.data.values() {
            match val {
                InternalEnvironmentValue::Value(name, val) => {
                    debug.field(&format!("'{name}'"), *val);
                }
                InternalEnvironmentValue::ValueMut(name, val) => {
                    debug.field(&format!("mut '{name}'"), *val);
                }
            }
        }

        debug.finish()
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

pub trait GetMany {
    type Output<'a>;

    fn get<'a>(env: &'a mut Environment) -> Self::Output<'a>;
}

impl<K1: EnvironmentKey> GetMany for K1 {
    type Output<'a> = Result<EnvironmentValue<'a, K1>, ()>;

    fn get<'a>(env: &'a mut Environment) -> Self::Output<'a> {
        let t1 = TypeId::of::<K1>();
        let [res] = env.data.get_disjoint_mut([&t1]);
        Option::map(res, extract_value::<K1>).flatten().ok_or(())
    }
}

// TODO: It is currently not possible to assert K1::Key and K2::Key are distinct types at compile time #143800 (const_cmp)
macro_rules! impl_get_many {
    ($(($($K:ident),+)),+ $(,)?) => {
        $(
            #[allow(non_snake_case)]
            impl<$($K: EnvironmentKey),+> GetMany for ($($K,)+) {
                type Output<'a> = Result<
                    ($(EnvironmentValue<'a, $K>,)+),
                    ($(Option<EnvironmentValue<'a, $K>>,)+),
                >;

                fn get<'a>(env: &'a mut Environment) -> Self::Output<'a> {
                    // Lowercase-free trick: reuse the type idents as local variable names.
                    // Locals and types are in separate namespaces, so this compiles.
                    $(let $K = TypeId::of::<$K>();)+

                    let [$($K),+] = env.data.get_disjoint_mut([$(&$K),+]);

                    // Nothing is moved unless the first arm matches, so the
                    // fallback arm can hand the untouched options back as `Err`.
                    match ($(Option::map($K, extract_value::<$K>).flatten(),)+) {
                        ($(Some($K),)+) => Ok(($($K,)+)),
                        partial => Err(partial),
                    }
                }
            }
        )+
    };
}

impl_get_many! {
    (K1),
    (K1, K2),
    (K1, K2, K3),
    (K1, K2, K3, K4),
    (K1, K2, K3, K4, K5),
    (K1, K2, K3, K4, K5, K6),
    (K1, K2, K3, K4, K5, K6, K7),
    (K1, K2, K3, K4, K5, K6, K7, K8),
    (K1, K2, K3, K4, K5, K6, K7, K8, K9),
    (K1, K2, K3, K4, K5, K6, K7, K8, K9, K10),
    (K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11),
    (K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12),
}

#[cfg(test)]
mod tests {
    use crate::environment::environment::{EnvironmentKey, Environment};
    use crate::environment::{Exclusive, Shared};

    #[derive(Copy, Clone, Debug)]
    struct TestKey;

    impl EnvironmentKey for TestKey { type Value = u64; type Access = Shared; }

    #[derive(Copy, Clone, Debug)]
    struct TestKey2;

    impl EnvironmentKey for TestKey2 { type Value = u64; type Access = Exclusive; }

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

    #[test]
    fn many() {
        let mut map = Environment::new();

        map.with::<TestKey>(&42, |inner| {
            inner.with::<TestKey2>(&mut 69, |inner| {
                let f = inner.get_many::<(TestKey, TestKey2)>();

                println!("{:?}", f)
            });

            let f = inner.get_many::<(TestKey, TestKey2)>();

            println!("{:?}", f)
        });
    }
}