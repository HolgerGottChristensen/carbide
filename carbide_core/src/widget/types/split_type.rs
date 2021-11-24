use crate::state::F64State;

#[derive(Debug, Clone)]
pub enum SplitType {
    Start(F64State),
    Percent(F64State),
    End(F64State),
}
