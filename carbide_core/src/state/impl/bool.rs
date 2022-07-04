use crate::state::{Map3, RState, StateContract, WidgetState};

impl WidgetState<bool> {
    pub fn choice<T: StateContract>(
        &self,
        s1: impl Into<RState<T>>,
        s2: impl Into<RState<T>>,
    ) -> RState<T> {
        Map3::read_map(self.clone(), s1, s2, |boolean: &bool, s1: &T, s2: &T| {
            if *boolean {
                s1.clone()
            } else {
                s2.clone()
            }
        })
    }
}
