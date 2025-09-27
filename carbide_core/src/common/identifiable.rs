use carbide::state::StateContract;

pub trait Identifiable<I: StateContract + PartialEq> {
    fn id(&self) -> I;
}

impl<T: StateContract + PartialEq> Identifiable<T> for T {
    fn id(&self) -> T {
        self.clone()
    }
}