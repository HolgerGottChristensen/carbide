use crate::environment::Environment;
use crate::state::{NewStateSync, ReadState, ReadWidgetState, RState, ValueRef, WidgetState};

pub trait StateAnd<Rhs> {
    fn and(self, other: Rhs) -> RState<bool>;
}

macro_rules! and {
    ($($typ1: ty, $typ2: ty;)*) => {
        $(
        impl StateAnd<$typ2> for $typ1 {
            fn and(self, other: $typ2) -> RState<bool> {
                AndState::new(self.clone(), other.clone())
            }
        }
        )*
    };
}

and!(
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
pub struct AndState {
    left: RState<bool>,
    right: RState<bool>,
}

impl AndState {
    pub fn new(left: impl Into<RState<bool>>, right: impl Into<RState<bool>>) -> RState<bool> {
        ReadWidgetState::new(Box::new(AndState {
            left: left.into(),
            right: right.into()
        }))
    }
}

/// Implement NewStateSync for the RMap
impl NewStateSync for AndState {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut updated = false;

        updated |= self.left.sync(env);

        if *self.left.value() {
            updated |= self.right.sync(env);
        }

        updated
    }
}

impl ReadState<bool> for AndState {
    fn value(&self) -> ValueRef<bool> {
        let res = *self.left.value() && *self.right.value();
        ValueRef::Owned(res)
    }
}

impl core::fmt::Debug for AndState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AndState")
            .field("left", &*self.left.value())
            .field("right", &*self.right.value())
            .finish()
    }
}