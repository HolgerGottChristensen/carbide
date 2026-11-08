use std::fmt::Debug;
use carbide::environment::Shared;
use crate::environment::Environment;
use crate::state::{EnvMap1, Map1};
use carbide_derive::StateValue;
use crate::environment::EnvironmentKey;

/// The way in which text should wrap around the width.
#[derive(Copy, Clone, Debug, PartialEq, StateValue, Eq, Hash)]
#[derive(Default)]
pub enum Wrap {
    /// Wrap at the first character that exceeds the width.
    Character,
    /// Wrap at the first word that exceeds the width.
    #[default]
    Whitespace,
    /// No wrapping
    None,
}


#[derive(Debug)]
pub(crate) struct TextWrapKey;

impl EnvironmentKey for TextWrapKey {
    type Value = Wrap;
    type Access = Shared;
}

pub type WrapState = EnvMap1<fn(&mut Environment, &i32) -> Wrap, i32, Wrap, i32>;

pub fn wrap_state() -> WrapState {
    Map1::read_map_env(0, |env, _| {
        // Look up enabled in the environment, or default to true of nothing is specified
        
        env.get::<TextWrapKey>().cloned().unwrap_or_default()
    })
}