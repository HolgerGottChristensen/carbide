use crate::environment::Environment;
use crate::state::{Map2, NewStateSync, ReadState, ReadWidgetState, RState, StateContract, TState, ValueRef, WidgetState};

pub trait StateOr<Rhs> {
    fn or(self, other: Rhs) -> RState<bool>;
}

macro_rules! or {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl StateOr<$typ2> for $typ1 {
            fn or(self, other: $typ2) -> RState<bool> {
                OrState::new(self.clone(), other.clone())
            }
        }
        )*
    };
}

or!(
    WidgetState<bool>, WidgetState<bool>;
    WidgetState<bool>, &WidgetState<bool>;
    WidgetState<bool>, ReadWidgetState<bool>;
    WidgetState<bool>, &ReadWidgetState<bool>;

    &WidgetState<bool>, WidgetState<bool>;
    &WidgetState<bool>, &WidgetState<bool>;
    &WidgetState<bool>, ReadWidgetState<bool>;
    &WidgetState<bool>, &ReadWidgetState<bool>;

    ReadWidgetState<bool>, WidgetState<bool>;
    ReadWidgetState<bool>, &WidgetState<bool>;
    ReadWidgetState<bool>, ReadWidgetState<bool>;
    ReadWidgetState<bool>, &ReadWidgetState<bool>;

    &ReadWidgetState<bool>, WidgetState<bool>;
    &ReadWidgetState<bool>, &WidgetState<bool>;
    &ReadWidgetState<bool>, ReadWidgetState<bool>;
    &ReadWidgetState<bool>, &ReadWidgetState<bool>;
);

#[derive(Clone)]
#[allow(unused_parens)]
pub struct OrState {
    left: RState<bool>,
    right: RState<bool>,
}

impl OrState {
    pub fn new(left: impl Into<RState<bool>>, right: impl Into<RState<bool>>) -> RState<bool> {
        ReadWidgetState::new(Box::new(OrState {
            left: left.into(),
            right: right.into()
        }))
    }
}

/// Implement NewStateSync for the RMap
impl NewStateSync for OrState {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut updated = false;

        updated |= self.left.sync(env);

        if !*self.left.value() {
            updated |= self.right.sync(env);
        }

        updated
    }
}

impl ReadState<bool> for OrState {
    fn value(&self) -> ValueRef<bool> {
        let res = *self.left.value() || *self.right.value();
        ValueRef::Owned(res)
    }
}

impl core::fmt::Debug for OrState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrState")
            .field("left", &*self.left.value())
            .field("right", &*self.right.value())
            .finish()
    }
}