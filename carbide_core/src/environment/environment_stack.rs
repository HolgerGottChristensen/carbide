use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{BuildHasherDefault, Hasher};
use std::mem::transmute;

pub trait Key: Any + Debug + Clone + 'static {
    type Value: Any + Debug + Clone + 'static;
}

pub trait Keyable: Debug + Clone + 'static {
    type Output: Any + Debug + Clone + 'static;

    fn get(&self, stack: &TypeMap) -> Self::Output;
    fn with(&self, value: &Self::Output, stack: &mut TypeMap, f: impl FnOnce(&mut TypeMap));

    fn with_all(values: &[(Self, Self::Output)], stack: &mut TypeMap, f: impl FnOnce(&mut TypeMap)) where Self: Sized {
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

trait AnyDebug: Any + Debug {
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
    fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }

    #[inline(always)]
    fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}

pub struct TypeMap<'a> {
    data: HashMap<TypeId, &'a dyn AnyDebug, BuildTypeIdHasher>,
    data_mut: HashMap<TypeId, &'a mut dyn AnyDebug, BuildTypeIdHasher>,
}

impl<'a> TypeMap<'a> {
    pub fn new() -> Self {
        TypeMap {
            data: HashMap::with_hasher(BuildTypeIdHasher::default()),
            data_mut: HashMap::with_hasher(BuildTypeIdHasher::default()),
        }
    }
}

impl<'a> TypeMap<'a> {
    pub fn get<K: Key>(&self) -> Option<&K::Value> {
        let id = TypeId::of::<K>();

        Option::map(self.data.get(&id), |value| {
            value.downcast_ref()
        }).flatten()
    }

    pub fn value<K: Keyable>(&self, key: K) -> K::Output {
        key.get(self)
    }

    #[allow(unsafe_code)]
    pub fn with<'b, K: Key>(&mut self, v: &'b K::Value, f: impl FnOnce(&mut TypeMap)) where 'a: 'b {
        let id = TypeId::of::<K>();

        // If any existing value existed in the data with the key, remove and store it.
        let old = self.data.remove(&id);

        // SAFETY: The transmuted map has a shorter lifetime. The map is
        // only used and accessed in the scope of the closure. We insert the
        // reference only for the duration of the closure, and ensure it is
        // removed again, before returning from the function, ensuring the
        // reference to the value v does is not in the map after the closure,
        // and thus making sure it does not escape.
        let transmuted: &'b mut TypeMap<'b> = unsafe { transmute(self) };

        transmuted.data.insert(id, v);

        f(transmuted);

        if let Some(old) = old {
            transmuted.data.insert(id, old);
        } else {
            transmuted.data.remove(&id);
        }
    }
}

impl<'a> TypeMap<'a> {
    pub fn get_mut<K: Key>(&mut self) -> Option<&mut K::Value> {
        let id = TypeId::of::<K>();

        Option::map(self.data_mut.get_mut(&id), |value| {
            value.downcast_mut()
        }).flatten()
    }

    #[allow(unsafe_code)]
    pub fn with_mut<'b, K: Key>(&mut self, v: &'b mut K::Value, f: impl FnOnce(&mut TypeMap)) where 'a: 'b {
        let id = TypeId::of::<K>();

        // If any existing value existed in the data with the key, remove and store it.
        let old = self.data_mut.remove(&id);

        // SAFETY: The transmuted map has a shorter lifetime. The map is
        // only used and accessed in the scope of the closure. We insert the
        // reference only for the duration of the closure, and ensure it is
        // removed again, before returning from the function, ensuring the
        // reference to the value v does is not in the map after the closure,
        // and thus making sure it does not escape.
        let transmuted: &'b mut TypeMap<'b> = unsafe { transmute(self) };

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
    use crate::environment::environment_stack::{Key, TypeMap};

    #[derive(Copy, Clone, Debug)]
    struct TestKey;
    impl Key for TestKey { type Value = u64; }

    #[test]
    fn simple() {
        let mut map = TypeMap::new();

        map.with::<TestKey>(&42, |inner| {
            let get = inner.get::<TestKey>();
            println!("{:?}", get);
        });

        let get = map.get::<TestKey>();
        println!("{:?}", get);
    }
}