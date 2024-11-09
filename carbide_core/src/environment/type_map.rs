use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hasher};
use std::mem::transmute;

pub trait Key: Any + Debug + Clone + 'static {
    type Value: Any + Debug + Clone + 'static;
}

pub trait Keyable {
    type Output: Any + Debug + Clone + 'static;

    fn get(&self, map: &TypeMap) -> Self::Output;
}

pub struct TypeMap<'a> {
    data: HashMap<TypeId, &'a dyn Any, BuildTypeIdHasher>
}

impl<'a> TypeMap<'a> {
    pub fn new() -> Self {
        TypeMap {
            data: HashMap::with_hasher(BuildTypeIdHasher::default()),
        }
    }

    pub fn get<K: Key>(&self) -> Option<&'a K::Value> {
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
    use crate::environment::type_map::{Key, TypeMap};

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