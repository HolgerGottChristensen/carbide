use std::hash::Hash;
use carbide::state::StateContract;

pub trait Identifiable {
    type Id: Hash + Eq;
    fn id(&self) -> Self::Id;
}
