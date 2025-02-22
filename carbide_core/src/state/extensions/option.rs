use carbide::state::Map1;
use crate::state::{IntoReadState, IntoState, Map2, ReadState, State, StateContract};

pub trait OptionExtension<T: StateContract>: ReadState<T=Option<T>> {
    fn is_some(&self) -> impl ReadState<T=bool>;
    fn is_none(&self) -> impl ReadState<T=bool>;
    fn contains<U: StateContract + PartialEq<T>>(&self, x: impl ReadState<T=U>) -> impl ReadState<T=bool>;

    /// Will return a state of result that is Ok with the value if the original is Some and
    /// Err with the err state if the original is None.
    ///
    /// When setting the resulting state with Ok it will set the original state to Some with the
    /// value. When setting the resulting state with Err you will set the error state and the
    /// original state will be None.
    fn ok_or<E: StateContract>(&self, err: impl IntoState<E>) -> impl State<T=Result<T, E>> where Self: State<T=Option<T>>;
}

impl<T: StateContract, S: ReadState<T=Option<T>>> OptionExtension<T> for S {
    fn is_some(&self) -> impl ReadState<T=bool> {
        Map1::read_map(self.clone(), |x| x.is_some())
    }

    fn is_none(&self) -> impl ReadState<T=bool> {
        Map1::read_map(self.clone(), |x| x.is_none())
    }

    fn contains<U: StateContract + PartialEq<T>>(&self, x: impl IntoReadState<U>) -> impl ReadState<T=bool> {
        Map2::read_map(self.clone(), x.into_read_state(), |a, b| {
            match a {
                None => false,
                Some(a) => b == a
            }
        })
    }

    fn ok_or<E: StateContract>(&self, err: impl IntoState<E>) -> impl State<T=Result<T, E>> where Self: State<T=Option<T>> {
        Map2::map(self.clone(), err.into_state(), |a: &Option<T>, b: &E| {
            a.clone().ok_or_else(|| b.clone())
        }, |new: Result<T, E>, mut a, mut b| {
            match new {
                Ok(o) => *a = Some(o),
                Err(e) => *b = e,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{LocalState, State};
    use crate::state::extensions::option::OptionExtension;
    use crate::state::read_state::ReadState;

    #[test]
    fn f() {
        let mut l = LocalState::new(Some(42));

        let contains = l.contains(42);

        assert!(*contains.value());

        *l.value_mut() = Some(43);

        assert!(!*contains.value())
    }
}