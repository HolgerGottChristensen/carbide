
pub trait GlobalState: 'static + Clone + std::fmt::Debug {}

impl<T> GlobalState for T where T: 'static + Clone + std::fmt::Debug {}