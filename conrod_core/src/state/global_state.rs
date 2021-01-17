
// The global state needs to implement clone because the widgets do, and for them to be clone
// All the generic types need to implement it as well. The global state should never in practise
// be cloned, because that would most likely be very expensive.
pub trait GlobalState: 'static + Clone + std::fmt::Debug {}

impl<T> GlobalState for T where T: 'static + Clone + std::fmt::Debug {}