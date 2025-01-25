use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hasher};
use std::marker::PhantomData;
use std::mem::transmute;
use crate::misc::any_debug::AnyDebug;

pub trait EnvironmentKey: Any + Debug + 'static {
    type Value: Any + Debug + 'static;
}

pub trait EnvironmentKeyable: Debug + 'static {
    type Output: Any + Debug + 'static;

    fn get(&self, stack: &Environment) -> Option<Self::Output>;
    fn with(&self, value: &Self::Output, stack: &mut Environment, f: impl FnOnce(&mut Environment));

    fn with_all(values: &[(Self, Self::Output)], stack: &mut Environment, f: impl FnOnce(&mut Environment)) where Self: Sized {
        match values {
            [(slef, first), rest @ ..] => {
                slef.with(first, stack, |inner| {
                    Self::with_all(rest, inner, f)
                })
            },
            [(slef, first)] => {
                slef.with(first, stack, f)
            }
            [] => {
                f(stack)
            }
        }
    }
}

pub struct Environment<'a> {
    data: HashMap<TypeId, &'a dyn AnyDebug, BuildTypeIdHasher>,
    data_mut: HashMap<TypeId, &'a mut dyn AnyDebug, BuildTypeIdHasher>,
    _marker: PhantomData<*const ()>, // Ensures the type is not Send and not Sync
}


impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Environment {
            data: HashMap::with_hasher(BuildTypeIdHasher::default()),
            data_mut: HashMap::with_hasher(BuildTypeIdHasher::default()),
            _marker: Default::default(),
        }
    }
}

impl<'a> Environment<'a> {
    pub fn get<K: EnvironmentKey + ?Sized>(&self) -> Option<&K::Value> {
        let id = TypeId::of::<K>();

        Option::map(self.data.get(&id), |value| {
            value.downcast_ref()
        }).flatten()
    }

    pub fn value<K: EnvironmentKeyable>(&self, key: &K) -> Option<K::Output> {
        key.get(self)
    }

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
}

impl<'a> Environment<'a> {
    pub fn get_mut<K: EnvironmentKey + ?Sized>(&mut self) -> Option<&mut K::Value> {
        let id = TypeId::of::<K>();

        Option::map(self.data_mut.get_mut(&id), |value| {
            value.downcast_mut()
        }).flatten()
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

pub type BuildTypeIdHasher = BuildHasherDefault<TypeIdHasher>;

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