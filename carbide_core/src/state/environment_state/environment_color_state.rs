use crate::state::environment_color::EnvironmentColor;
use crate::Color;
use crate::prelude::{GlobalState, State, Environment};
use crate::state::state_key::StateKey;

#[derive(Clone)]
pub struct EnvironmentColorState {
    key: StateKey,
    value: Color,
}

impl EnvironmentColorState {
    pub fn new(key: EnvironmentColor) -> Self {
        EnvironmentColorState {
            key: StateKey::Color(key),
            value: Color::Rgba(0.0,0.0,0.0,1.0)
        }
    }
}

impl<GS: GlobalState> State<Color, GS> for EnvironmentColorState{
    fn get_value_mut(&mut self, env: &mut Environment<GS>, _: &mut GS) -> &mut Color {
        if let Some(color) = env.get_color(&self.key) {
            self.value = color;
        }

        &mut self.value
    }

    fn get_value(&mut self, env: &Environment<GS>, _: &GS) -> &Color {
        if let Some(color) = env.get_color(&self.key) {
            self.value = color;
        }

        &self.value
    }

    fn get_latest_value(&self) -> &Color {
        &self.value
    }

    fn get_latest_value_mut(&mut self) -> &mut Color {
        &mut self.value
    }

    fn get_key(&self) -> Option<&StateKey> {
        None
    }

    fn update_dependent_states(&mut self, _: &Environment<GS>) {}

    fn insert_dependent_states(&self, _: &mut Environment<GS>) {}
}