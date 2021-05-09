use crate::prelude::{Environment, EnvironmentFontSize, GlobalState, State};
use crate::state::state_key::StateKey;

#[derive(Clone)]
pub struct EnvironmentFontSizeState {
    key: StateKey,
    value: u32,
}

impl EnvironmentFontSizeState {
    pub fn new(key: EnvironmentFontSize) -> Self {
        EnvironmentFontSizeState {
            key: StateKey::FontSize(key),
            value: 20
        }
    }
}

impl<GS: GlobalState> State<u32, GS> for EnvironmentFontSizeState{
    fn get_value_mut(&mut self, env: &mut Environment<GS>, _: &mut GS) -> &mut u32 {
        if let Some(size) = env.get_font_size(&self.key) {
            self.value = size;
        }

        &mut self.value
    }

    fn get_value(&mut self, env: &Environment<GS>, _: &GS) -> &u32 {
        if let Some(size) = env.get_font_size(&self.key) {
            self.value = size;
        }

        &self.value
    }

    fn get_latest_value(&self) -> &u32 {
        &self.value
    }

    fn get_latest_value_mut(&mut self) -> &mut u32 {
        &mut self.value
    }

    fn get_key(&self) -> Option<&StateKey> {
        None
    }

    fn update_dependent_states(&mut self, _: &Environment<GS>) {}

    fn insert_dependent_states(&self, _: &mut Environment<GS>) {}
}