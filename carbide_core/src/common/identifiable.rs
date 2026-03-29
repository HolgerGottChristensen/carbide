use std::fmt::Debug;
use std::hash::Hash;
use carbide::state::StateContract;

pub trait Identifiable {
    type Id: Hash + Eq + Clone + Debug;
    fn id(&self) -> Self::Id;
}

macro_rules! impl_identifiable {
    ($($t:ty),*) => {
        $(
            impl Identifiable for $t {
                type Id = $t;

                fn id(&self) -> Self::Id {
                    *self
                }
            }
        )*
    };
}

impl_identifiable!(
    (),
    u8, i8,
    u16, i16,
    u32, i32,
    u64, i64,
    u128, i128,
    usize, isize
);