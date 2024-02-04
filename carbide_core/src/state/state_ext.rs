
use carbide_core::state::ReadState;
use crate::state::{AnyReadState, AnyState, IgnoreWritesState, LoggingState, Map1, RMap1, State, StateContract, TransitionState, TState};


pub trait StateExt<T>: Into<TState<T>> + Clone
where
    T: StateContract,
{
    // /// Example: size.mapped(|t: &f64| { format!("{:.2}", t) })
    // fn mapped<TO: StateContract + Default + 'static, M: Map<T, TO> + Clone>(
    //     &self,
    //     map: M,
    // ) -> TState<TO> {
    //     MapOwnedState::<T, TO>::new(self.clone(), move |s: &T, _: &_, _: &_| map(s)).into()
    // }
}

impl<T: StateContract, U> StateExt<T> for U where U: Into<TState<T>> + Clone {}


pub trait StateExtNew<T>: State<T=T> + Sized + Clone + 'static where T: StateContract {
    fn as_dyn(&self) -> Box<dyn AnyState<T=T>> {
        Box::new(self.clone())
    }

    fn log_changes(&self) -> LoggingState<T, Self> {
        LoggingState::new(self.clone())
    }
}

impl<T: StateContract, S> StateExtNew<T> for S where S: State<T=T> + Sized + Clone + 'static {}


pub trait ReadStateExtNew<T>: ReadState<T=T> + Sized + Clone + 'static where T: StateContract {
    fn as_dyn_read(&self) -> Box<dyn AnyReadState<T=T>> {
        Box::new(self.clone())
    }

    fn ignore_writes(&self) -> IgnoreWritesState<T, Self> {
        IgnoreWritesState::new(self.clone())
    }

    /// This map a state to another state. The resulting state is read-only.
    /// If you need a TState, use [Map1::map()] instead
    ///
    /// Example: size.map(|t: &f64| { format!("{:.2}", t) })
    ///
    /// This will return a RState<String> that will stay updated with the size
    fn map<TO: StateContract, MAP: Fn(&T) -> TO + Clone + 'static>(&self, map: MAP) -> RMap1<MAP, T, TO, Self> {
        Map1::read_map(self.clone(), map)
    }
}

impl<T: StateContract, S> ReadStateExtNew<T> for S where S: ReadState<T=T> + Sized + Clone + 'static {}