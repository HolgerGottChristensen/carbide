use crate::state::TState;

#[derive(Debug, Clone)]
pub enum SplitType {
    Start(TState<f64>),
    Percent(TState<f64>),
    End(TState<f64>),
}
