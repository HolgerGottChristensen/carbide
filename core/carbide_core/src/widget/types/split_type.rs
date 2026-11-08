use crate::state::State;

#[derive(Debug, Clone)]
pub enum SplitType<T> where T: State<T=f64> + Clone {
    Start(T),
    Percent(T),
    End(T),
}
